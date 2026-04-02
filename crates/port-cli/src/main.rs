use clap::Parser;

fn main() -> anyhow::Result<()> {
    port::run(port::Cli::parse())
}
