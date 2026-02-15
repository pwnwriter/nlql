// claude integration - turns plain english into sql

use crate::Error;
use serde::{Deserialize, Serialize};

pub struct Claude {
    client: reqwest::Client,
    api_key: String,
}

// what we send to claude
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

// what claude sends back
#[derive(Deserialize)]
struct Response {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

impl Claude {
    pub fn new() -> Result<Self, Error> {
        // check common env var names for the api key
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("CLAUDE_API_KEY"))
            .or_else(|_| std::env::var("CLAUDE_KEY"))
            .map_err(|_| Error::MissingApiKey)?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }

    pub async fn generate_sql(&self, prompt: &str, schema: &str) -> Result<String, Error> {
        // tell claude what we need and give it the schema
        let system = format!(
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
        );

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
            let error = response.text().await?;
            return Err(Error::Claude(error));
        }

        let response: Response = response.json().await?;
        let sql = response
            .content
            .first()
            .map(|c| c.text.trim().to_string())
            .unwrap_or_default();

        // claude sometimes wraps sql in markdown code blocks
        let sql = sql
            .trim_start_matches("```sql")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string();

        Ok(sql)
    }
}
