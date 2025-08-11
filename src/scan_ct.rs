use reth_db::{
    cursor::DbCursorRO,
    tables,
    transaction::DbTx,
    open_db_read_only,
    mdbx::DatabaseArguments,
    Database,
};
use std::{path::Path, sync::Arc};
use crate::databases::BscDatabase;
use alloy_consensus::transaction::Transaction; // bring to() into scope

fn scan_contract_creations_raw<T: DbTx>(tx: &T) -> eyre::Result<()> {
    let mut cursor = tx.cursor_read::<tables::Transactions>()?;
    while let Some((tx_num, t)) = cursor.next()? {
        if t.to().is_none() {
            // 这里处理合约创建交易（to 为 None）
            tracing::debug!(tx_num, "contract creation tx detected");
        }
    }
    Ok(())
}

/// 打开只读 MDBX 并扫描所有合约创建交易
pub fn scan_contract_creations(datadir: impl AsRef<Path>) -> eyre::Result<()> {
    let datadir = datadir.as_ref();
    let db_path = datadir.join("db");
    let db = Arc::new(open_db_read_only(&db_path, DatabaseArguments::default())?);
    let tx = db.tx()?;
    scan_contract_creations_raw(&tx)
}

/// 扫描指定区块内的所有交易，判定是否为合约创建（to == None）
pub fn scan_block_contract_creations(db: &BscDatabase, block_number: u64) -> eyre::Result<Vec<bool>> {
    let txs = db.query_block_order_transactions(block_number)?;
    let flags = txs.iter().map(|t| t.to().is_none()).collect();
    Ok(flags)
}

/// 打印版：按序输出 idx 与是否合约创建
pub fn print_block_contract_creations(db: &BscDatabase, block_number: u64) -> eyre::Result<()> {
    let flags = scan_block_contract_creations(db, block_number)?;
    for (i, is_create) in flags.iter().copied().enumerate() {
        println!("{}: {}", i, if is_create { "CREATE" } else { "CALL" });
    }
    Ok(())
}
