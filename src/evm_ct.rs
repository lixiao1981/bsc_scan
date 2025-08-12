use std::sync::Arc;

// bring signer recovery trait into scope for `recover_signer()`
use alloy_consensus::transaction::SignerRecoverable;
use eyre::{eyre, Result};
use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};
use reth_ethereum::{
    chainspec::ChainSpecBuilder,
    evm::{
        primitives::ConfigureEvm,
        EthEvmConfig,
        revm::{database::StateProviderDatabase, db::CacheDB},
    },
    node::EthereumNode,
    provider::{providers::ReadOnlyConfig, BlockReader},
};
// traits needed for evm transact and db commit
use alloy_evm::Evm;
use reth::revm::DatabaseCommit;
use reth_primitives::{Recovered};
use reth::rpc::types::BlockHashOrNumber;


pub fn evm_ct_test(block: u64, datadir: String) -> Result<()> {
    // ProviderFactory
    let spec = Arc::new(ChainSpecBuilder::mainnet().build());
    let factory = EthereumNode::provider_factory_builder()
        .open_read_only(spec.clone(), ReadOnlyConfig::from_datadir(&datadir))?;
    let provider = factory.provider()?;

    // 读取区块 & 状态
    let block = provider
        .block(BlockHashOrNumber::Number(block))?
        .ok_or_else(|| eyre!("block {} not found", block))?;
    let state_provider = provider.history_by_block_hash(block.header.parent_hash)?;
    let mut db = CacheDB::new(StateProviderDatabase::new(state_provider.as_ref()));

    // EVM 环境
    let evm_config = EthEvmConfig::new(spec);
    let mut evm_env = evm_config.evm_env(&block.header);
    evm_env.cfg_env.disable_block_gas_limit = true;

    // 追踪器
    let mut inspector = TracingInspector::new(TracingInspectorConfig::default());

    // 执行区块内每笔交易
    for tx in &block.body.transactions {
        let recovered_tx =
            Recovered::new_unchecked(tx.clone(), tx.recover_signer().unwrap());
        let tx_env = evm_config.tx_env(&recovered_tx);

        let mut evm =
            evm_config.evm_with_env_and_inspector(&mut db, evm_env.clone(), &mut inspector);
        let result = evm.transact(tx_env)?;

        // 输出合约创建事件
        for node in inspector
            .traces()
            .nodes()
            .iter()
            .filter(|n| n.trace.kind.is_any_create())
        {
            let t = &node.trace;
            println!(
                "new contract: addr={:?}, creator={:?}, init_code_len={}",
                t.address,
                t.caller,
                t.data.len()
            );
        }

        inspector.traces_mut().clear();
        db.commit(result.state);
    }

    Ok(())
}
