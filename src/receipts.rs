use std::path::PathBuf;
use reth_provider::providers::StaticFileProvider;
use reth_static_file_types::StaticFileSegment;
use reth_db::static_file::ReceiptMask;
use reth_ethereum_primitives::{EthPrimitives, Receipt};   // Reth 内置的 Receipt 类型

/// 从指定 static_files 目录中，读取包含给定块号的 receipts 段并打印摘要
pub fn test_receipts(static_dir: impl Into<PathBuf>, block_in_segment: u64) -> eyre::Result<()> {
    // 1) 打开静态文件提供者（压缩：true）
    let sf_provider = StaticFileProvider::<EthPrimitives>::read_only(static_dir.into(), true)?;

    // 2) 获取包含该块号的 receipts 段 jar
    let jar = sf_provider.get_segment_provider_from_block(
        StaticFileSegment::Receipts,
        block_in_segment,
        None,
    )?;

    // 3) 遍历段内所有收据
    let mut cursor = jar.cursor()?;
    let mut tx_num = cursor.jar().user_header().start().unwrap_or_default();
    while let Some(receipt) = cursor.get_one::<ReceiptMask<Receipt>>(tx_num.into())? {
        println!(
            "tx_num={} tx_type={:?} success={} cumulative_gas_used={} logs={}",
            tx_num,
            receipt.tx_type,
            receipt.success,
            receipt.cumulative_gas_used,
            receipt.logs.len(),
        );
        tx_num += 1;
    }
    Ok(())
}

