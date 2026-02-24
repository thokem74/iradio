use anyhow::Result;
use clap::Parser;
use iradio::app::run;

#[derive(Debug, Parser)]
#[command(name = "iradio", version, about = "Interactive internet radio TUI")]
struct Cli {
    #[arg(long, help = "Enable verbose debug logs")]
    debug: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli.debug)
}
