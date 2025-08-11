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
        Commands::Tos { db_path, block } => {
            let db = BscDatabase::new(db_path)?;
            let tos = bsc_scan::al::analyze_block_transactions_with_to(&db, block)?;
            for (i, to) in tos.iter().enumerate() {
                match to {
                    Some(addr) => println!("{}: {:#x}", i, addr),
                    None => println!("{}: None", i),
                }
            }
        }
        Commands::ReceiptsTest { static_dir, block } => {
            bsc_scan::receipts::test_receipts(static_dir, block)?;
        }
        Commands::TransactionsTest { static_dir, block } => {
            bsc_scan::tranactions::test_transactions(static_dir, block)?;
        }
        Commands::CubeDemo { db_path, block, tx } => {
            let tx_hash_opt = tx
                .as_deref()
                .and_then(|s| s.parse().ok());
            bsc_scan::cube_med::demo_common(db_path, block, tx_hash_opt)?;
        }
    }

    Ok(())
}
