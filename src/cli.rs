// command line interface - handles all user interaction

use crate::{Claude, Db, Error, Output, Safety, Server};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nlql", about = "talk to your database in plain english")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// ask your database a question
    Query {
        /// what you want to know
        prompt: String,

        /// database connection url
        #[arg(long, short, env = "DATABASE_URL")]
        db: String,

        /// just show the sql, don't run it
        #[arg(long)]
        dry_run: bool,

        /// how to show results: pretty, raw, or sql-only
        #[arg(long, short, default_value = "pretty")]
        output: OutputFormat,

        /// skip the safety check
        #[arg(long)]
        no_check: bool,

        /// allow dangerous stuff like DROP or DELETE without WHERE
        #[arg(long)]
        run_dangerous: bool,
    },

    /// start as http server
    Serve {
        /// database connection url
        #[arg(long, short, env = "DATABASE_URL")]
        db: String,

        /// port number
        #[arg(long, short, default_value = "3000")]
        port: u16,

        /// host to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// show what tables and columns exist
    Schema {
        /// database connection url
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

pub async fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query {
            prompt,
            db,
            dry_run,
            output,
            no_check,
            run_dangerous,
        } => query(&prompt, &db, dry_run, output, no_check, run_dangerous).await,

        Commands::Serve { db, port, host } => Server::run(&db, &host, port).await,

        Commands::Schema { db } => schema(&db).await,
    }
}

// ask claude to write sql, check it, run it
async fn query(
    prompt: &str,
    db_url: &str,
    dry_run: bool,
    output: OutputFormat,
    no_check: bool,
    run_dangerous: bool,
) -> Result<(), Error> {
    // connect and grab the schema so claude knows what tables exist
    let db = Db::connect(db_url).await?;
    let schema = db.schema().await?;

    // ask claude to write the sql
    let claude = Claude::new()?;
    let sql = claude.generate_sql(prompt, &schema).await?;

    // make sure it's not doing anything sketchy
    if !no_check {
        let safety = Safety::check(&sql);
        if safety.is_dangerous && !run_dangerous {
            eprintln!("that looks dangerous: {}", safety.reason);
            eprintln!("sql: {sql}");
            eprintln!("\nuse --run-dangerous if you really want to run it");
            return Ok(());
        }
        if let Some(warning) = safety.warning {
            eprintln!("heads up: {warning}");
        }
    }

    // just show sql if that's all they want
    if dry_run || matches!(output, OutputFormat::SqlOnly) {
        println!("{sql}");
        return Ok(());
    }

    // run it and show results
    let rows = db.execute(&sql).await?;
    match output {
        OutputFormat::Pretty => Output::pretty(&sql, &rows),
        OutputFormat::Raw => Output::raw(&rows),
        OutputFormat::SqlOnly => unreachable!(),
    }

    Ok(())
}

// dump the database schema as json
async fn schema(db_url: &str) -> Result<(), Error> {
    let db = Db::connect(db_url).await?;
    let schema = db.schema().await?;
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
