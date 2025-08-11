use std::path::Path;

use eyre::Result;
use reth_chainspec::ChainSpecBuilder;
use reth_node_ethereum::node::EthereumNode;
use reth_provider::providers::ReadOnlyConfig;

/// 初始化 MDBX 与 static_files，并构建 ProviderFactory 与只读 Provider。
/// datadir 需包含:
/// - <datadir>/db
/// - <datadir>/static_files
pub fn init_stack(datadir: impl AsRef<Path>) -> Result<()> {
    let datadir = datadir.as_ref();

    // 1) 构建链配置（以太坊主网；BSC 可替换）
    let spec = ChainSpecBuilder::mainnet().build();

    // 2) 通过节点构建器只读打开数据目录（自动挂载 db 与 static_files）
    let factory = EthereumNode::provider_factory_builder()
        .open_read_only(spec.into(), ReadOnlyConfig::from_datadir(datadir))?;

    // 3) 获取只读 Provider
    let _provider = factory.provider()?; // 若需使用，绑定到变量并继续调用 API

    tracing::info!("ProviderFactory initialized and read-only provider acquired");
    Ok(())
}

