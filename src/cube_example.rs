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
use alloy_consensus::transaction::Transaction; // to(), nonce(), value(), etc.

/// cube_example: 展示常见 factory/_provider 用法
///
/// 传入包含 db 与 static_files 的数据目录；其余参数用于示例查询。
pub fn demo_factory(datadir: impl AsRef<Path>, block_number: u64, tx_hash: Option<B256>) -> Result<()> {
    let datadir = datadir.as_ref();

    // 打开只读 ProviderFactory
    let spec = ChainSpecBuilder::mainnet().build();
    let factory = EthereumNode::provider_factory_builder()
        .open_read_only(spec.clone().into(), ReadOnlyConfig::from_datadir(datadir))
        .context("open_read_only provider factory")?;

    // 5) ChainSpecProvider: 返回链配置信息
    // 注：不同版本 API 名称略有差异，常见为 factory.chain_spec()/chain_spec()
    // 这里示例从我们手上已有的 spec 获取链 ID
    let chain_id = spec.chain().id();
    tracing::info!(chain_id, "chain id from spec");

    // 获取只读 provider
    let provider = factory.provider().context("get read-only provider")?;

    // 9) BlockNumReader: 读取链信息与区块高度
    let latest = provider.best_block_number().context("best_block_number")?;
    tracing::info!(latest, "latest block number");

    // 7/8) HeaderProvider + 根据高度查找（常见组合：header_by_number / header_by_hash）
    if let Some(header) = provider.header_by_number(block_number).context("header_by_number")? {
        tracing::info!(block = header.number, timestamp = %header.timestamp, gas_used = %header.gas_used, "header fetched");
    } else {
        tracing::warn!(block_number, "header not found");
    }

    // 13) BlockBodyIndicesProvider: 获取区块体索引（first_tx_num / tx_count）
    if let Some(indices) = provider.block_body_indices(block_number).context("block_body_indices")? {
        tracing::info!(first_tx = indices.first_tx_num(), tx_count = indices.tx_count(), "body indices");
    } else {
        tracing::warn!(block_number, "body indices not found");
    }

    // 11) TransactionsProvider: 读取区块内交易
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

    // 12) ReceiptProvider: 读取回执
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

    // 11/12 组合：交易哈希 -> TxNumber -> 回执
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

    // 16) HashedPostStateProvider / 3) StateCommitmentProvider：
    // 这些一般用于计算状态根或读取哈希化状态，API 可能在不同 crate/trait 中，版本差异较大。
    // 在本示例中，展示 StateProviderFactory 提供的最新状态读取（更稳定通用）。
    let state = factory.latest().context("factory.latest() state provider")?;
    let addr: Address = Address::ZERO;
    match state.basic_account(&addr) {
        Ok(Some(acc)) => tracing::info!(balance = %acc.balance, nonce = acc.nonce, "basic account for ZERO"),
        Ok(None) => tracing::info!("no basic account for ZERO"),
        Err(e) => tracing::warn!(error = %e, "basic_account error"),
    }

    let slot0: B256 = U256::from(0).into();
    match state.storage(addr, slot0) {
        Ok(Some(val)) => tracing::debug!(?val, "storage slot0 for ZERO"),
        Ok(None) => tracing::debug!("no storage at slot0 for ZERO"),
        Err(e) => tracing::warn!(error = %e, "storage error"),
    }

    // 其余条目按功能说明（不同版本 API 可能在 factory 或 provider 上）：
    // 1) NodePrimitivesProvider: factory.primitives()（如有）
    // 2) DatabaseProviderFactory: factory.provider()/provider_rw()（统一生成读/写 DatabaseProvider）
    // 4) StaticFileProviderFactory: factory.static_file_provider()（如有）；否则 StaticFileProvider::read_only(datadir/static_files)
    // 6) HeaderSyncGapProvider: factory.local_tip_header(num)（如有）
    // 8) BlockHashReader: factory/provider.block_hash(height)（按版本）
    // 10) BlockReader: factory/provider.block(id)
    // 14) StageCheckpointReader: factory.get_stage_checkpoint(stage)（如有）
    // 15) PruneCheckpointReader: factory.get_prune_checkpoint(segment)（如有）

    Ok(())
}
