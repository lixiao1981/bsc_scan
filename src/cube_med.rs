use std::path::Path;

use eyre::{Context as _, Result};
use alloy_primitives::{Address, B256, U256};
use reth_chainspec::ChainSpecBuilder;
use reth_node_ethereum::node::EthereumNode;
use reth_provider::providers::ReadOnlyConfig;
use reth_provider::{
    BlockBodyIndicesProvider, BlockNumReader, HeaderProvider, ReceiptProvider, StateProviderFactory,
    TransactionsProvider,
};
use alloy_consensus::transaction::Transaction; // bring to()/gas/nonce/value APIs

/// 演示 _provider 的常见查询：区块头、交易、回执、体索引、状态等
pub fn demo_common(datadir: impl AsRef<Path>, block_number: u64, tx_hash: Option<B256>) -> Result<()> {
    let datadir = datadir.as_ref();
    let spec = ChainSpecBuilder::mainnet().build();
    let factory = EthereumNode::provider_factory_builder()
        .open_read_only(spec.into(), ReadOnlyConfig::from_datadir(datadir))
        .context("open_read_only provider factory")?;
    let provider = factory.provider().context("get read-only provider")?;
    // 1) 基本高度/区块头
    let latest = provider.best_block_number().context("best_block_number")?;
    tracing::info!(latest, "latest block number");

    if let Some(header) = provider.header_by_number(block_number).context("header_by_number")? {
        tracing::info!(block = header.number, timestamp = %header.timestamp, gas_used = %header.gas_used, "header fetched");
    } else {
        tracing::warn!(block_number, "header not found");
    }

    // 2) 区块体索引（定位交易范围）
    if let Some(indices) = provider.block_body_indices(block_number).context("block_body_indices")? {
        tracing::info!(first_tx = indices.first_tx_num(), tx_count = indices.tx_count(), "body indices");
    } else {
        tracing::warn!(block_number, "body indices not found");
    }

    // 3) 交易列表
    match provider.transactions_by_block(block_number.into()) {
        Ok(Some(txs)) => {
            tracing::info!(count = txs.len(), "transactions in block");
            if let Some(first) = txs.first() {
                tracing::debug!(hash = %first.hash(), to = ?first.to(), "first tx");
            }
        }
        Ok(None) => tracing::info!("no transactions in block"),
        Err(e) => tracing::warn!(error = %e, "transactions_by_block error"),
    }

    // 4) 回执列表
    match provider.receipts_by_block(block_number.into()) {
        Ok(Some(rcpts)) => {
            tracing::info!(count = rcpts.len(), "receipts in block");
            if let Some(r) = rcpts.first() {
                tracing::debug!(success = r.success, cumulative_gas_used = r.cumulative_gas_used, logs = r.logs.len(), "first receipt");
            }
        }
        Ok(None) => tracing::info!("no receipts in block"),
        Err(e) => tracing::warn!(error = %e, "receipts_by_block error"),
    }

    // 5) 按交易哈希查回执
    if let Some(h) = tx_hash {
        match provider.transaction_id(h) {
            Ok(Some(tx_num)) => match provider.receipt(tx_num) {
                Ok(Some(rcpt)) => tracing::info!(success = rcpt.success, gas = rcpt.cumulative_gas_used, "receipt by hash"),
                Ok(None) => tracing::warn!("receipt not found by tx number"),
                Err(e) => tracing::warn!(error = %e, "receipt() error"),
            },
            Ok(None) => tracing::warn!("transaction_id not found for hash"),
            Err(e) => tracing::warn!(error = %e, "transaction_id error"),
        }
    }

    // 6) 状态访问（通过 factory.latest() 获取 StateProvider）
    let state = factory.latest().context("factory.latest() state provider")?;
    let addr: Address = Address::ZERO;
    match state.basic_account(&addr) {
        Ok(Some(acc)) => tracing::info!(balance = %acc.balance, nonce = acc.nonce, "basic account for ZERO"),
        Ok(None) => tracing::info!("no basic account for ZERO"),
        Err(e) => tracing::warn!(error = %e, "basic_account error"),
    }

    // 读取一个存储槽（slot 0）
    let slot0: B256 = U256::from(0).into();
    match state.storage(addr, slot0) {
        Ok(Some(val)) => tracing::debug!(?val, "storage slot0 for ZERO"),
        Ok(None) => tracing::debug!("no storage at slot0 for ZERO"),
        Err(e) => tracing::warn!(error = %e, "storage error"),
    }

    Ok(())
}
