// command line interface

use crate::tui::DbInfo;
use crate::{Db, Provider, Server};
use clap::{Parser, Subcommand};
use miette::Result;

#[derive(Parser)]
#[command(name = "nlql", about = "Talk to your database in plain english")]
struct Cli {
    /// database connection url
    #[arg(long, short, env = "DATABASE_URL", global = true)]
    db: Option<String>,

    /// ai provider (claude, openai)
    #[arg(long, short = 'p', default_value = "claude", global = true)]
    provider: Provider,

    /// api key for the ai provider
    #[arg(long, short = 'k', global = true)]
    api_key: Option<String>,

    /// ask for confirmation before running sql
    #[arg(long, short)]
    confirm: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// start as http server
    Serve {
        /// port number
        #[arg(long, short, default_value = "3000")]
        port: u16,

        /// host to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Serve { port, host }) => {
            // serve mode requires --db
            let db = cli
                .db
                .ok_or_else(|| miette::miette!("database url required (--db or DATABASE_URL)"))?;
            Ok(Server::run(&db, &host, port).await?)
        }

        None => {
            // TUI mode - check if we have a database URL
            match cli.db {
                Some(db) => {
                    // normal mode: connect and run TUI
                    let db_conn = Db::connect(&db).await?;
                    let schema = db_conn.schema().await?;

                    let tables = schema.matches("TABLE ").count();

                    let db_info = DbInfo {
                        dialect: db_conn.dialect_name().to_string(),
                        host: db_conn.host().to_string(),
                        database: db_conn.database().to_string(),
                        tables,
                        url: db.clone(),
                    };

                    Ok(crate::tui::run(
                        Some(db_conn),
                        Some(schema),
                        Some(db_info),
                        cli.confirm,
                        cli.provider,
                        cli.api_key,
                    )
                    .await?)
                }
                None => {
                    // setup mode: launch TUI with interactive setup
                    Ok(
                        crate::tui::run(None, None, None, cli.confirm, cli.provider, cli.api_key)
                            .await?,
                    )
                }
            }
        }
    }
}
