pub use reth_provider::{HeaderProvider, TransactionsProvider, StateProvider, ReceiptProvider};

pub mod databases;
pub mod cli;
pub mod error;
pub mod mdbx;
pub mod al;
pub mod receipts;
/// 初始化 tracing（可传入日志级别；否则读取环境变量，默认 info）
pub fn init_tracing(level: Option<&str>) {
	use tracing_subscriber::{EnvFilter, fmt, prelude::*};

	let env_filter = match level {
		Some(lvl) => EnvFilter::new(lvl),
		None => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
	};

	let fmt_layer = fmt::layer().with_target(true).with_ansi(true);

	tracing_subscriber::registry()
		.with(env_filter)
		.with(fmt_layer)
		.init();
}