use bsc_scan::{cli::{Cli, Commands}, databases::BscDatabase};
use clap::Parser;
use eyre::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    bsc_scan::init_tracing(cli.log.as_deref());

    tracing::info!(version = %env!("CARGO_PKG_VERSION"), "Starting bsc_scan");

    match cli.command {
        Commands::Header { db_path, block } => {
            let db = BscDatabase::new(db_path)?;
            match db.query_headers_with_blocknumber(block)? {
                Some(data) => {
                    println!(
                        "Block #{:?}\nHash: {:?}\nParent: {:?}\nTimestamp: {}\nTx count: {}",
                        data.header.number,
                        data.header.hash_slow(),
                        data.header.parent_hash,
                        data.header.timestamp,
                        data.tx_count,
                    );
                }
                None => {
                    println!("Block #{} not found or not available", block);
                }
            }
        }
    }

    Ok(())
}
