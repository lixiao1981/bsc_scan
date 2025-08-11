use clap::{Parser, Subcommand};

/// bsc_scan 命令行
#[derive(Debug, Parser)]
#[command(name = "bsc_scan", version, about = "BSC DB utilities", author = "")]
pub struct Cli {
    /// 日志等级（info|debug|warn|error），默认从 RUST_LOG 读取
    #[arg(long, value_name = "LEVEL", global = true)]
    pub log: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 根据区块号查询区块头与交易数量
    Header {
    /// 数据目录路径（包含 reth/bsc 数据库）
    #[arg(long, value_name = "PATH")]
    db_path: String,
        /// 区块号
        #[arg(value_name = "BLOCK_NUMBER")]
        block: u64,
    },
    /// 输出该区块内所有交易的 to（原始 Address，合约创建为 None）
    Tos {
        /// 数据目录路径（包含 reth/bsc 数据库）
        #[arg(long, value_name = "PATH")]
        db_path: String,
        /// 区块号
        #[arg(value_name = "BLOCK_NUMBER")]
        block: u64,
    },
    /// 从 static_files 中测试读取 receipts 段并打印摘要
    ReceiptsTest {
        /// static_files 目录（通常是 <db_path>/static_files）
        #[arg(long, value_name = "PATH")]
        static_dir: String,
        /// 任意位于该段内的块号（用于定位段文件）
        #[arg(value_name = "BLOCK_NUMBER")]
        block: u64,
    },
}
