use lazy_static::lazy_static;
use log::info;
use rusqlite::{Connection, Params, Result, Row};
use std::sync::Mutex;

// 使用 lazy_static! 宏创建一个全局静态变量 CONN，它是一个 Mutex 包裹的 Connection 对象
lazy_static! {
    pub static ref CONN: Mutex<Connection> = Mutex::new(init());
}

// 初始化数据库连接的函数
pub fn init() -> Connection {
    // 初始化连接
    info!("开始初始化本地数据库连接");
    // 打开名为 rusqlite.db 的数据库文件，如果打开失败会导致程序 panic，实际应用中应进行错误处理
    let conn = Connection::open("rusqlite.db").unwrap();
    info!("初始化本地数据库连接成功");

    {

        let _ = conn.execute("PRAGMA synchronous = OFF",(),);

        // 初始化数据，在一个代码块中执行一系列 SQL 语句创建表和索引
        let _ = conn.execute(
            "
                            CREATE TABLE MAP_DB_EXPIRE (
                                TIMEOUT BIGINT,
                                EXPIRE_KEY VARCHAR(1000),
                                STORAGE_TYPE VARCHAR(1000)
                            )",
            (),
        );
        let _ = conn.execute(
            "CREATE INDEX MAP_DB_EXPIRE_TIMEOUT_IDX ON MAP_DB_EXPIRE (TIMEOUT)",
            (),
        );
        let _ = conn.execute(
            "CREATE INDEX MAP_DB_EXPIRE_EXPIRE_KEY_IDX ON MAP_DB_EXPIRE (EXPIRE_KEY)",
            (),
        );
        let _ = conn.execute(
            "CREATE INDEX MAP_DB_EXPIRE_STORAGE_TYPE_IDX ON MAP_DB_EXPIRE (STORAGE_TYPE)",
            (),
        );
    }
    {
        // 初始化数据，在另一个代码块中执行 SQL 语句创建另一个表和索引
        let _ = conn.execute(
            r#"
                            CREATE TABLE DISK_CACHE (
                            KEY VARCHAR(1000) NOT NULL,
                            DATA TEXT,
                            PRIMARY KEY ("KEY")
                            );"#,
            (),
        );
        let _ = conn.execute("CREATE INDEX DISK_CACHE_KEY ON DISK_CACHE (KEY)", ());
    }

    // 返回初始化后的数据库连接
    conn
}

// 执行无参数的 SQL 语句的函数
pub fn execute<S: Into<String>>(sql: S) -> Result<usize> {
    // 获取全局的数据库连接锁
    let conn = CONN.lock().unwrap();
    let sql: String = sql.into();
    // 执行 SQL 语句并返回受影响的行数，如果执行失败会返回错误
    conn.execute(sql.as_str(), ())
}

// 执行带参数的 SQL 语句的函数
pub fn execute_by_params<S: Into<String>>(sql: S, params: impl Params) -> Result<usize> {
    // 获取全局的数据库连接锁
    let conn = CONN.lock().unwrap();
    let sql: String = sql.into();
    // 执行带参数的 SQL 语句并返回受影响的行数，如果执行失败会返回错误
    conn.execute(sql.as_str(), params)
}

// 执行带参数的 SQL 查询语句并获取单个结果的函数
pub fn select_one<S: Into<String>, F, T>(sql: S, params: impl Params, mut f: F) -> Result<Option<T>>
where
    F: FnMut(&Row<'_>) -> T,
{
    // 获取全局的数据库连接锁
    let conn = CONN.lock().unwrap();
    let sql: String = sql.into();
    let mut stmt = conn.prepare(sql.as_str())?;
    let mut rows = stmt.query(params)?;
    if let Some(row) = rows.next()? {
        // 如果有结果，调用传入的闭包函数 f 处理行数据并返回包含结果的 Option
        let t: T = f(row);
        Ok(Some(t))
    } else {
        // 如果没有结果，返回 None
        Ok(None)
    }
}

// 执行带参数的 SQL 查询语句并获取所有结果的函数
pub fn select_all<S: Into<String>, F, T>(sql: S, params: impl Params, mut f: F) -> Result<Vec<T>>
where
    F: FnMut(&Row<'_>) -> T,
{
    // 获取全局的数据库连接锁
    let conn = CONN.lock().unwrap();
    let sql: String = sql.into();
    let mut stmt = conn.prepare(sql.as_str())?;
    let mut rows = stmt.query(params)?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        // 遍历所有结果行，调用传入的闭包函数 f 处理行数据并将结果添加到向量中
        let t: T = f(row);
        results.push(t);
    }
    // 返回 返回包含所有结果的向量
    Ok(results)
}