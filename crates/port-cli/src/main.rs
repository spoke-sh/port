use clap::Parser;

fn main() -> anyhow::Result<()> {
    port_cli::run(port_cli::Cli::parse())
}
