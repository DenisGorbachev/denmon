use clap::Parser;
use denmon::{Cli, Outcome};

#[tokio::main]
async fn main() -> Outcome {
    let args = Cli::parse();
    args.run().await
}
