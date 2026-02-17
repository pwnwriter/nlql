// ai providers - turns plain english into sql

use crate::Error;
use serde::{Deserialize, Serialize};

/// which ai provider to use
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Provider {
    #[default]
    Claude,
    #[value(alias = "chatgpt", alias = "gpt")]
    OpenAI,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Claude => write!(f, "claude"),
            Provider::OpenAI => write!(f, "openai"),
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" | "anthropic" => Ok(Provider::Claude),
            "openai" | "chatgpt" | "gpt" => Ok(Provider::OpenAI),
            _ => Err(format!("unknown provider: {s}")),
        }
    }
}

/// ai client that can use different providers
pub struct Ai {
    provider: Provider,
    client: reqwest::Client,
    api_key: String,
}

impl Ai {
    pub fn new(provider: Provider, api_key: Option<String>) -> Result<Self, Error> {
        let api_key = match provider {
            Provider::Claude => api_key
                .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                .or_else(|| std::env::var("CLAUDE_API_KEY").ok())
                .ok_or(Error::MissingApiKey {
                    provider: "claude",
                    env_var: "ANTHROPIC_API_KEY",
                })?,
            Provider::OpenAI => api_key
                .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                .ok_or(Error::MissingApiKey {
                    provider: "openai",
                    env_var: "OPENAI_API_KEY",
                })?,
        };

        Ok(Self {
            provider,
            client: reqwest::Client::new(),
            api_key,
        })
    }

    pub fn provider(&self) -> Provider {
        self.provider
    }

    pub async fn generate_sql(&self, prompt: &str, schema: &str) -> Result<String, Error> {
        match self.provider {
            Provider::Claude => self.call_claude(prompt, schema).await,
            Provider::OpenAI => self.call_openai(prompt, schema).await,
        }
    }

    async fn call_claude(&self, prompt: &str, schema: &str) -> Result<String, Error> {
        #[derive(Serialize)]
        struct Request {
            model: &'static str,
            max_tokens: u32,
            messages: Vec<Message>,
            system: String,
        }

        #[derive(Serialize)]
        struct Message {
            role: &'static str,
            content: String,
        }

        #[derive(Deserialize)]
        struct Response {
            content: Vec<Content>,
        }

        #[derive(Deserialize)]
        struct Content {
            text: String,
        }

        let system = self.system_prompt(schema);

        let request = Request {
            model: "claude-sonnet-4-20250514",
            max_tokens: 1024,
            system,
            messages: vec![Message {
                role: "user",
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await?;
            return Err(Error::Ai(format!("claude {status}: {error}")));
        }

        let response: Response = response.json().await?;
        let sql = response
            .content
            .first()
            .map(|c| c.text.trim().to_string())
            .unwrap_or_default();

        Ok(self.clean_sql(&sql))
    }

    async fn call_openai(&self, prompt: &str, schema: &str) -> Result<String, Error> {
        #[derive(Serialize)]
        struct Request {
            model: &'static str,
            messages: Vec<Message>,
            max_tokens: u32,
        }

        #[derive(Serialize)]
        struct Message {
            role: &'static str,
            content: String,
        }

        #[derive(Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: ResponseMessage,
        }

        #[derive(Deserialize)]
        struct ResponseMessage {
            content: String,
        }

        let system = self.system_prompt(schema);

        let request = Request {
            model: "gpt-4o",
            max_tokens: 1024,
            messages: vec![
                Message {
                    role: "system",
                    content: system,
                },
                Message {
                    role: "user",
                    content: prompt.to_string(),
                },
            ],
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await?;
            return Err(Error::Ai(format!("openai {status}: {error}")));
        }

        let response: Response = response.json().await?;
        let sql = response
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_default();

        Ok(self.clean_sql(&sql))
    }

    fn system_prompt(&self, schema: &str) -> String {
        format!(
            r#"You are a SQL query generator. Given a natural language request, generate a valid SQL query.

Database schema:
{schema}

Rules:
- Output ONLY the SQL query, no explanations or markdown
- Use proper SQL syntax for the database
- Be precise with table and column names from the schema
- For SELECT queries, be specific about columns when possible
- For PostgreSQL: cast timestamp/date columns to text (e.g., created_at::text)
- Add reasonable LIMIT if none specified (max 100 rows)"#
        )
    }

    fn clean_sql(&self, sql: &str) -> String {
        sql.trim_start_matches("```sql")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string()
    }
}
