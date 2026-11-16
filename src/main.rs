mod models;
mod routes;
mod service;
mod storage;

use service::StudentService;
use storage::StudentStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 数据库密码等敏感信息从环境变量读取，不写死在源代码中。
    // 示例：
    // export DATABASE_URL='postgres://postgres:<密码>@121.40.244.102:5432/wf'
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "缺少 DATABASE_URL 环境变量，请先配置 PostgreSQL 连接地址")?;

    // 程序启动时创建连接池，后续所有请求复用池中的连接。
    let store = StudentStore::connect(&database_url).await?;
    let service = StudentService::new(store);
    let app = routes::create_router(service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;

    println!("🚀 Rust REST 服务已启动: http://127.0.0.1:8081");
    println!("   GET  /health          - 健康检查");
    println!("   GET  /students        - 从 PostgreSQL 获取全部学生");
    println!("   GET  /students/:id    - 根据 ID 获取学生");

    axum::serve(listener, app).await?;
    Ok(())
}
