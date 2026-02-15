use clap::{Parser, Subcommand};
use nlql::{Claude, Db, Error, Output, Safety, Server};

#[derive(Parser)]
#[command(name = "nlql", about = "Natural Language to SQL using Claude AI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate and execute SQL from natural language
    Query {
        /// The natural language query
        prompt: String,

        /// Database URL
        #[arg(long, short, env = "DATABASE_URL")]
        db: String,

        /// Just show SQL, don't execute
        #[arg(long)]
        dry_run: bool,

        /// Output format: pretty, raw, or sql-only
        #[arg(long, short, default_value = "pretty")]
        output: OutputFormat,

        /// Skip safety check
        #[arg(long)]
        no_check: bool,

        /// Allow dangerous queries (DROP, DELETE without WHERE, etc.)
        #[arg(long)]
        run_dangerous: bool,
    },

    /// Start HTTP server
    Serve {
        /// Database URL
        #[arg(long, short, env = "DATABASE_URL")]
        db: String,

        /// Port to listen on
        #[arg(long, short, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Show database schema
    Schema {
        /// Database URL
        #[arg(long, short, env = "DATABASE_URL")]
        db: String,
    },
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Pretty,
    Raw,
    SqlOnly,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query {
            prompt,
            db,
            dry_run,
            output,
            no_check,
            run_dangerous,
        } => {
            // Connect to database
            let db = Db::connect(&db).await?;

            // Get schema for context
            let schema = db.schema().await?;

            // Generate SQL using Claude
            let claude = Claude::new()?;
            let sql = claude.generate_sql(&prompt, &schema).await?;

            // Safety check
            if !no_check {
                let safety = Safety::check(&sql);
                if safety.is_dangerous && !run_dangerous {
                    eprintln!("Dangerous query detected: {}", safety.reason);
                    eprintln!("SQL: {sql}");
                    eprintln!("\nUse --allow-dangerous to run anyway.");
                    return Ok(());
                }
                if let Some(warning) = safety.warning {
                    eprintln!("Warning: {warning}");
                }
            }

            // Output SQL only
            if dry_run || matches!(output, OutputFormat::SqlOnly) {
                println!("{sql}");
                return Ok(());
            }

            // Execute and display results
            let rows = db.execute(&sql).await?;
            match output {
                OutputFormat::Pretty => Output::pretty(&sql, &rows),
                OutputFormat::Raw => Output::raw(&rows),
                OutputFormat::SqlOnly => unreachable!(),
            }
        }

        Commands::Serve { db, port, host } => {
            Server::run(&db, &host, port).await?;
        }

        Commands::Schema { db } => {
            let db = Db::connect(&db).await?;
            let schema = db.schema().await?;
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
    }

    Ok(())
}
