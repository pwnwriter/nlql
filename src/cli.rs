// command line interface

use crate::tui::DbInfo;
use crate::{Db, Server};
use clap::{Parser, Subcommand};
use miette::Result;

#[derive(Parser)]
#[command(name = "nlql", about = "Talk to your database in plain english")]
struct Cli {
    /// database connection url
    #[arg(long, short, env = "DATABASE_URL", global = true)]
    db: Option<String>,

    /// anthropic api key
    #[arg(long, short = 'k', env = "ANTHROPIC_API_KEY", global = true)]
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

    let db = cli
        .db
        .ok_or_else(|| miette::miette!("database url required (--db or DATABASE_URL)"))?;

    match cli.command {
        Some(Commands::Serve { port, host }) => Ok(Server::run(&db, &host, port).await?),

        None => {
            let db_conn = Db::connect(&db).await?;
            let schema = db_conn.schema().await?;

            // count tables from schema
            let tables = schema.matches("TABLE ").count();

            let db_info = DbInfo {
                dialect: db_conn.dialect_name().to_string(),
                host: db_conn.host().to_string(),
                database: db_conn.database().to_string(),
                tables,
                url: db.clone(),
            };

            Ok(crate::tui::run(db_conn, schema, db_info, cli.confirm, cli.api_key).await?)
        }
    }
}
