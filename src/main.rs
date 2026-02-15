// nlql - Talk to your database in plain english

use nlql::cli;

// Core entry point: ᢉ𐭩: where magic happens
fn main() -> miette::Result<()> {
    // enable pretty error output
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .context_lines(2)
                .build(),
        )
    }))?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { cli::run().await })
}
