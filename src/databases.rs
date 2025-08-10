use alloy_primitives::{Address, U256, B256};
use eyre::{Context, Result};
use std::sync::Arc;
use std::path::Path;

use reth_node_ethereum::{
    node::EthereumNode,
};
use reth_chainspec::ChainSpecBuilder;
use reth_provider::{
    providers::ReadOnlyConfig,
    HeaderProvider, ReceiptProvider, StateProvider, TransactionsProvider,
    ProviderFactory, BlockNumReader, BlockBodyIndicesProvider,
};
use reth_node_api::NodeTypesWithDBAdapter;
use reth_db::DatabaseEnv;
use reth_primitives::{Header, TransactionSigned};



/// Comprehensive BSC database testing utility
pub struct BscDatabase {
    // Use the actual type returned by the factory builder
    pub provider_factory: ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>,
    pub latest_block: u64,
    pub earliest_available_block: u64,
}

/// 根据区块号查询得到的区块数据
pub struct BlockData {
    pub header: Header,
    pub tx_count: usize,
}

impl BscDatabase {
    /// Create a new BSC database test instance
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        tracing::info!(path = %db_path.as_ref().display(), "Opening BSC database");
        
        // Create chain spec (use mainnet for now, can be customized for BSC)
        let spec = ChainSpecBuilder::mainnet().build();
        
        // Create provider factory using the EthereumNode
        let provider_factory = EthereumNode::provider_factory_builder()
            .open_read_only(spec.into(), ReadOnlyConfig::from_datadir(db_path))?;
        
    tracing::info!("Database opened successfully");
        
        // Detect node type and available data
        let ( latest_block, earliest_available_block) = 
            Self::detect_node_characteristics(&provider_factory)?;
        
        Ok(Self {
            provider_factory,
            latest_block,
            earliest_available_block,
        })
    }
    
    /// Detect node type and available block range
    fn detect_node_characteristics(
        provider_factory: &ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>
    ) -> Result<(u64, u64)> {
        tracing::info!("Detecting node type and available data range...");
        
        let provider = provider_factory.provider()?;
        
        // Get latest block
        let latest_block = provider
            .best_block_number()
            .context("Failed to get latest block number")?;
        
    tracing::info!(latest_block, "Latest block");
        
        // Test early blocks to determine node type
        let test_blocks = vec![1, 100, 1000, 10000];
        let mut earliest_available = latest_block;
        
        for block_num in test_blocks {
            if block_num > latest_block {
                continue;
            }
            
            match provider.header_by_number(block_num) {
                Ok(Some(_)) => {
                    earliest_available = block_num;
                    tracing::debug!(block_num, "Block available");
                },
                Ok(None) => {
                    tracing::debug!(block_num, "Block not found");
                    break;
                },
                Err(e) => {
                    tracing::warn!(block_num, error = %e, "Block access error");
                    break;
                }
            }
        }
        
        // Binary search for exact earliest block
        if earliest_available > 1 {
            let actual_earliest = Self::binary_search_earliest_block(&provider, 1, earliest_available)?;
            earliest_available = actual_earliest;
        }
        
    let _available_blocks = latest_block - earliest_available + 1;
        
        
        Ok((latest_block, earliest_available))
    }
    
    /// Binary search to find the earliest available block
    fn binary_search_earliest_block(
        provider: &impl HeaderProvider,
        mut low: u64,
        mut high: u64
    ) -> Result<u64> {
        while low < high {
            let mid = (low + high) / 2;
            match provider.header_by_number(mid) {
                Ok(Some(_)) => high = mid,
                _ => low = mid + 1,
            }
        }
        Ok(low)
    }
    
    /// Run all available tests
    pub fn run_all_tests(&self) -> Result<()> {
    tracing::info!("Starting comprehensive BSC database tests...");
        
        
    tracing::debug!(separator = %"=".repeat(60));
        // Query latest header as a sanity check
        let _ = self.query_headers_with_blocknumber(self.latest_block)?;
        
    // tracing::debug!(separator = %"=".repeat(60));
    //     self.test_transactions()?;
        
    tracing::debug!(separator = %"=".repeat(60));
        self.test_state()?;
        
    tracing::debug!(separator = %"=".repeat(60));
        self.test_receipts()?;
        
    tracing::info!("All tests completed successfully");
        Ok(())
    }
    

    /// 根据区块号返回该区块的数据（头 + 交易数量）。不存在则返回 None。
    pub fn query_headers_with_blocknumber(&self, block_number: u64) -> Result<Option<BlockData>> {
        let provider = self.provider_factory.provider()?;

        if block_number < self.earliest_available_block {
            tracing::debug!(block_number, earliest = self.earliest_available_block, "Header not available before earliest block");
            return Ok(None);
        }

        let header_opt = provider
            .header_by_number(block_number)
            .context("Failed to get header by number")?;

        if let Some(header) = header_opt {
            // 获取交易数量（如果存在区块体）
            let tx_count = match provider.block_body_indices(block_number) {
                Ok(Some(body)) => body.tx_count() as usize,
                _ => 0,
            };

            tracing::info!(
                block = %header.number,
                hash = ?header.hash_slow(),
                parent = ?header.parent_hash,
                timestamp = %header.timestamp,
                tx_count,
                "Fetched block data",
            );

            Ok(Some(BlockData { header, tx_count }))
        } else {
            tracing::debug!(block_number, "Header not found");
            Ok(None)
        }
    }
    
    /// 查询指定区块号下的所有交易，若无区块体或无交易则返回空 Vec
    pub fn query_block_order_transactions(&self, block_number: u64) -> Result<Vec<TransactionSigned>> {
        let provider = self.provider_factory.provider()?;

        if block_number < self.earliest_available_block {
            tracing::debug!(block_number, earliest = self.earliest_available_block, "Transactions not available before earliest block");
            return Ok(Vec::new());
        }

        // 先检查是否存在区块体，可快速判定是否有交易
        match provider.block_body_indices(block_number) {
            Ok(Some(body)) => {
                let tx_count = body.tx_count();
                tracing::info!(block_number, tx_count, "Block body found, loading transactions");
            }
            Ok(None) => {
                tracing::debug!(block_number, "Block body not found");
                return Ok(Vec::new());
            }
            Err(e) => {
                tracing::warn!(block_number, error = %e, "Block body read error");
                return Err(e.into());
            }
        }

        match provider.transactions_by_block(block_number.into()) {
            Ok(Some(txs)) => {
                tracing::info!(block_number, count = txs.len(), "Fetched transactions for block");
                Ok(txs)
            }
            Ok(None) => {
                tracing::debug!(block_number, "No transactions found for block");
                Ok(Vec::new())
            }
            Err(e) => {
                tracing::warn!(block_number, error = %e, "Failed to load transactions for block");
                Err(e.into())
            }
        }
    }
    
    /// Test state provider functionality with system contracts
    pub fn test_state(&self) -> Result<()> {
    tracing::info!("Testing State Provider...");
        
        // Always use latest state to avoid pruning issues
        let state_provider = self.provider_factory.latest()
            .context("Failed to get latest state provider")?;
        
        // Test some known addresses
        let test_contracts = vec![
            ("Null Address", "0x0000000000000000000000000000000000000000"),
            ("BSC Validator Set", "0x0000000000000000000000000000000000001000"),
            ("BSC System Reward", "0x0000000000000000000000000000000000001002"),
        ];
        
    tracing::info!("Testing system contracts at latest state");
        
        for (name, address_str) in test_contracts {
            let address: Address = address_str.parse()
                .context("Failed to parse contract address")?;
            
            match state_provider.account_code(&address) {
                Ok(Some(code)) => {
                    tracing::debug!(contract = %name, code_len = code.len(), "Contract code present");
                    
                    // Check account info
                    if let Ok(Some(account)) = state_provider.basic_account(&address) {
                        tracing::debug!(balance = %account.balance, nonce = %account.nonce, "Account info");
                    }
                },
                Ok(None) => {
                    if address == Address::ZERO {
                        tracing::debug!(contract = %name, "No code (expected)");
                    } else {
                        tracing::debug!(contract = %name, "No code found");
                    }
                },
                Err(e) => {
                    tracing::warn!(contract = %name, error = %e, "Code access error");
                }
            }
        }
        
        // Test storage for a known contract
        let test_address: Address = "0x0000000000000000000000000000000000001000".parse()?;
    tracing::info!("Testing storage access for system contract");
        
        // Test common storage slots
        for slot in 0..3 {
            let storage_key: B256 = U256::from(slot).into();
            match state_provider.storage(test_address, storage_key) {
                Ok(Some(value)) => {
                    if !value.is_zero() {
                        tracing::debug!(slot, ?value, "Storage slot value");
                    } else {
                        tracing::debug!(slot, "Storage slot empty");
                    }
                },
                Ok(None) => {
                    tracing::debug!(slot, "Storage slot no data");
                },
                Err(e) => {
                    tracing::warn!(slot, error = %e, "Storage slot error");
                }
            }
        }
        
        tracing::info!("State provider test completed");
        Ok(())
    }
    
    /// Test receipt provider functionality
    pub fn test_receipts(&self) -> Result<()> {
    tracing::info!("Testing Receipt Provider...");
        
        let provider = self.provider_factory.provider()?;
        
        // Test receipts from safe blocks with transactions
        let test_blocks = self.get_safe_test_blocks(3);
        let mut total_gas_used = 0u64;
        let mut successful_txs = 0;
        let mut failed_txs = 0;
        
        for &block_num in &test_blocks {
            match provider.receipts_by_block(block_num.into()) {
                Ok(Some(receipts)) => {
                    tracing::info!(block_num, receipts = %receipts.len(), "Receipts in block");
                    
                    for (i, receipt) in receipts.iter().enumerate() {
                        total_gas_used += receipt.cumulative_gas_used;
                        
                        if receipt.success {
                            successful_txs += 1;
                        } else {
                            failed_txs += 1;
                        }
                        
                        // Show details for first few receipts
                        if i < 2 && !receipts.is_empty() {
                            tracing::debug!(index = i, gas_used = %receipt.cumulative_gas_used, success = receipt.success, logs = %receipt.logs.len(), "Receipt summary");
                        }
                    }
                },
                Ok(None) => {
                    tracing::debug!(block_num, "No receipts found");
                },
                Err(e) => {
                    tracing::warn!(block_num, error = %e, "Receipts access error");
                }
            }
        }
        
        tracing::info!(total_gas_used, successful_txs, failed_txs, "Receipt summary");
        
        if successful_txs + failed_txs > 0 {
            let success_rate = (successful_txs as f64 / (successful_txs + failed_txs) as f64) * 100.0;
            tracing::info!(success_rate, "Success rate");
        }
        
        tracing::info!("Receipt provider test completed");
        Ok(())
    }
    
    /// Get safe block numbers for testing based on available range
    fn get_safe_test_blocks(&self, count: usize) -> Vec<u64> {
        let mut blocks = Vec::new();
        
        // Always include latest block
        blocks.push(self.latest_block);
        
        if count > 1 {
            let available_range = self.latest_block - self.earliest_available_block + 1;
            let step = if available_range > count as u64 {
                available_range / count as u64
            } else {
                1
            };
            
            for i in 1..count {
                let block_num = self.latest_block.saturating_sub(i as u64 * step);
                if block_num >= self.earliest_available_block && !blocks.contains(&block_num) {
                    blocks.push(block_num);
                }
            }
        }
        
        blocks.sort();
        blocks
    }
    
    /// Test individual header by number
    pub fn test_header_by_number(&self, block_number: u64) -> Result<()> {
        let provider = self.provider_factory.provider()?;
        
        if block_number < self.earliest_available_block {
            tracing::debug!(block_number, earliest = self.earliest_available_block, "Block is before earliest available");
            return Ok(());
        }
        
        match provider.header_by_number(block_number) {
            Ok(Some(header)) => {
                tracing::debug!(block_number, hash = ?header.hash_slow(), timestamp = %header.timestamp, gas_used = %header.gas_used, "Block header found");
            },
            Ok(None) => {
                tracing::debug!(block_number, "Block not found");
            },
            Err(e) => {
                tracing::warn!(block_number, error = %e, "Header by number error");
            }
        }
        
        Ok(())
    }
    
    /// Test state at specific block (if available)
    pub fn test_state_at_block(&self, block_number: u64) -> Result<()> {
        if block_number < self.earliest_available_block {
            tracing::debug!(block_number, earliest = self.earliest_available_block, "State at block not available (before earliest)");
            return Ok(());
        }
        
        
        // For historical state access, we'd need to use different methods
        // This is a simplified version that just tests current capabilities
    tracing::info!("Historical state testing would require additional provider methods");
    tracing::info!("State test completed with available methods");
        
        Ok(())
    }
}

/// Helper function to run basic BSC database tests
pub fn run_bsc_database_tests<P: AsRef<Path>>(db_path: P) -> Result<()> {
    let db = BscDatabase::new(db_path)?;
    db.run_all_tests()
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_type_classification() {
        // These would need actual database paths to run
        // Keeping as example structure
    }
    
    #[test]
    fn test_safe_block_generation() {
        // Test the safe block number generation logic
        // This can be unit tested without database
    }
}
