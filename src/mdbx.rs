use std::path::Path;
use std::sync::Arc;
use reth_db::{
    open_db_read_only,
    mdbx::DatabaseArguments,
    tables,                // access table definitions like PlainAccountState
    cursor::DbCursorRO,
    transaction::DbTx,     // RO transaction trait
    Database,              // brings tx() into scope for DatabaseEnv
};

pub fn mdbxinit() -> eyre::Result<()> {
    // 1. 打开 MDBX 数据库目录（注意传入的是目录，而非 mdbx.dat 文件本身）
    let db = Arc::new(open_db_read_only(
        Path::new("/ethereumdata/bsc/db"),
        DatabaseArguments::default(),
    )?);

    // 2. 启动只读事务
    let tx = db.tx()?;

    // 3. 读取表：以账户表为例（PlainAccountState）
    let mut cursor = tx.cursor_read::<tables::PlainAccountState>()?;

    // 4. 遍历表内记录
    while let Some((addr, account)) = cursor.next()? {
        tracing::info!(
            address = %addr,
            nonce = account.nonce,
            balance = %account.balance,
            "Account"
        );
    }

    // 若只需查询特定账户：
    // let addr = Address::from_slice(&[0u8; 20]);
    // if let Some(account) = tx.get::<Accounts>(&addr)? {
    //     println!("特定地址余额: {}", account.balance);
    // }

    Ok(())
}
