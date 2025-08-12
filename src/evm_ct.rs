use std::sync::Arc;
use reth_primitives::BlockHashOrNumber;
use eyre::{eyre, Result};
use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};
use reth_ethereum::{
    chainspec::ChainSpecBuilder,
    evm::{EthEvmConfig, primitives::ConfigureEvm, revm::{database::StateProviderDatabase, db::CacheDB}},
    node::EthereumNode,
    provider::{providers::ReadOnlyConfig, BlockReader},
};

pub fn evm_ct_test(block: u64, datadir: String) -> Result<()> {
    let spec = Arc::new(ChainSpecBuilder::mainnet().build());
    let factory = EthereumNode::provider_factory_builder()
        .open_read_only(spec.clone(), ReadOnlyConfig::from_datadir(&datadir))?;
    let provider = factory.provider()?;

    let block = provider
        .block(BlockHashOrNumber::Number(block))?
        .ok_or_else(|| eyre!("block {} not found", block))?;
    let state = provider.state_provider(block.header.state_root)?;
    let mut db = CacheDB::new(StateProviderDatabase::new(state));

    let evm_config = EthEvmConfig::new(spec);
    let mut evm_env = evm_config.evm_env(&block.header);
    evm_env.cfg_env.disable_block_gas_limit = true;

    let mut inspector = TracingInspector::new(TracingInspectorConfig::default_parity());
    for tx in &block.body.transactions {
        let tx_env = evm_config.tx_env(tx);
        let mut evm =
            evm_config.evm_with_env_and_inspector(&mut db, evm_env.clone(), &mut inspector);
        let result = evm.transact(tx_env)?;

        for node in inspector.traces().node.iter().filter_map(|n| n.trace.create()) {
            println!(
                "new contract: addr={:?}, creator={:?}, init_code_len={}",
                node.address,
                node.caller,
                node.init_code.len()
            );
        }

        inspector.traces_mut().node.clear();
        db.commit(result.state);
    }
    Ok(())
}
