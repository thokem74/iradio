use anyhow::Result;
use clap::Parser;
use iradio::app::run;

#[derive(Debug, Parser)]
#[command(name = "iradio", version, about = "Interactive internet radio TUI")]
struct Cli {}

fn main() -> Result<()> {
    let _ = Cli::parse();
    run()
}
