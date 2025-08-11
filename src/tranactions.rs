use std::path::Path;

use eyre::{bail, Result};
use reth_db::static_file::TransactionMask;
use reth_ethereum_primitives::EthPrimitives;
use reth_primitives::TransactionSigned;
use alloy_consensus::transaction::Transaction; // for to()/value()/nonce() on TransactionSigned
use reth_provider::providers::StaticFileProvider;
use reth_static_file_types::StaticFileSegment;
use crate::databases::BscDatabase;
use reth_provider::BlockBodyIndicesProvider;

/// 从 static_files 的 Transactions 段精准读取某区块的所有交易并打印摘要
/// 注意：传入的是数据目录路径（包含 static_files 子目录）
pub fn test_transactions(path: impl AsRef<Path>, block_number: u64) -> Result<()> {
    let p = path.as_ref();
    // 兼容传入数据目录或 static_files 目录
    let db_path;
    let static_dir;
    let candidate = p.join("static_files");
    if candidate.is_dir() {
        db_path = p;
        static_dir = candidate;
    } else {
        db_path = p;
        static_dir = p.to_path_buf();
    }

    // 1) 使用 provider 查询该区块的交易范围（起始 tx 编号与 tx 数量）
    let db = BscDatabase::new(db_path)?;
    let provider = db.provider_factory.provider()?;
    let indices = match provider.block_body_indices(block_number)? {
        Some(idx) => idx,
        None => bail!("No BlockBodyIndices found for block {block_number}"),
    };

    let start_tx: u64 = indices.first_tx_num();
    let tx_count: u64 = indices.tx_count();
    let end_tx = start_tx + tx_count;

    // 2) 打开包含该区块的 Transactions 段
    let sf = StaticFileProvider::<EthPrimitives>::read_only(static_dir, true)?;
    let tx_jar = sf.get_segment_provider_from_block(
        StaticFileSegment::Transactions,
        block_number,
        None,
    )?;
    let mut tx_cur = tx_jar.cursor()?;

    // 4) 遍历该区块内的所有交易，仅在范围内读取
    let mut printed = 0u64;
    for tx_num in start_tx..end_tx {
        if let Some(tx) = tx_cur.get_one::<TransactionMask<TransactionSigned>>(tx_num.into())? {
            println!(
                "block={} tx_num={} hash={:#x} to={}",
                block_number,
                tx_num,
                tx.hash(),
                tx.to()
                    .map(|a| format!("{a:#x}"))
                    .unwrap_or_else(|| "create".to_string())
            );
            printed += 1;
        } else {
            eprintln!("warn: missing tx at tx_num {} (block {})", tx_num, block_number);
        }
    }

    if printed == 0 {
        bail!(
            "No transactions printed for block {block_number} (tx range {}..{})",
            start_tx,
            end_tx
        );
    }

    Ok(())
}
