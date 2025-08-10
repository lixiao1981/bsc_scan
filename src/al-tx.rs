use alloy_primitives::{Address, B256, U256};
use alloy_consensus::transaction::Transaction; // bring nonce/gas_limit/to/value/input/gas_price APIs into scope
use eyre::Result;
use reth_primitives::TransactionSigned;

use crate::databases::BscDatabase;

/// 可分析的交易摘要结构
#[derive(Debug, Clone)]
pub struct AnalyzedTx {
	pub block_number: u64,
	pub index: u32,
	pub hash: B256,
	pub to: Option<Address>,
	pub value: U256,
	pub nonce: u64,
	pub gas_limit: u64,
	pub gas_price: Option<U256>,                // Legacy/EIP-2930
	pub max_fee_per_gas: Option<U256>,          // EIP-1559/4844
	pub max_priority_fee_per_gas: Option<U256>, // EIP-1559/4844
	pub input_size: usize,
	pub tx_type: &'static str,
}

/// 将 Vec<TransactionSigned> 转换为可分析的列表
pub fn analyze_txs(block_number: u64, txs: Vec<TransactionSigned>) -> Vec<AnalyzedTx> {
	txs.into_iter()
		.enumerate()
		.map(|(idx, tx)| {
			// 通用字段
			let hash = tx.hash();
			let nonce = tx.nonce();
			let gas_limit = tx.gas_limit();
			let to = tx.to();
			let value = tx.value();
			let input_size = tx.input().len();

			// 费用与类型因交易变体而异
			let (gas_price, max_fee_per_gas, max_priority_fee_per_gas, tx_type) = match &tx {
				TransactionSigned::Legacy(t) => (
					t.gas_price().map(U256::from),
					None,
					None,
					"Legacy",
				),
				TransactionSigned::Eip2930(t) => (
					t.gas_price().map(U256::from),
					None,
					None,
					"Eip2930",
				),
				TransactionSigned::Eip1559(t) => (
					None,
					Some(U256::from(t.max_fee_per_gas())),
					t.max_priority_fee_per_gas().map(U256::from),
					"Eip1559",
				),
				TransactionSigned::Eip4844(t) => (
					None,
					Some(U256::from(t.max_fee_per_gas())),
					t.max_priority_fee_per_gas().map(U256::from),
					"Eip4844",
				),
				#[allow(unused)]
				TransactionSigned::Eip7702(t) => (
					None,
					Some(U256::from(t.max_fee_per_gas())),
					t.max_priority_fee_per_gas().map(U256::from),
					"Eip7702",
				),
			};

			AnalyzedTx {
				block_number,
				index: idx as u32,
				hash: *hash,
				to,
				value,
				nonce,
				gas_limit,
				gas_price,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				input_size,
				tx_type,
			}
		})
		.collect()
}

/// 入口：读取区块交易并转换为 AnalyzedTx
pub fn analyze_block(db: &BscDatabase, block_number: u64) -> Result<Vec<AnalyzedTx>> {
	let txs = db.query_block_order_transactions(block_number)?;
	Ok(analyze_txs(block_number, txs))
}

