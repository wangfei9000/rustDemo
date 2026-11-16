use crate::models::Student;
use sqlx::{PgPool, postgres::PgPoolOptions};

/// PostgreSQL 学生存储。
///
/// `PgPool` 内部已经使用 `Arc` 管理共享连接池，因此克隆 `StudentStore`
/// 的成本很低，也不会为每个请求重新建立数据库连接。
#[derive(Clone)]
pub struct StudentStore {
    pool: PgPool,
}

impl StudentStore {
    /// 创建最多包含 5 个连接的 PostgreSQL 连接池。
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// 执行查询并返回 `stu3` 表中的全部学生。
    ///
    /// `stu_name` 是 `char(10)` 类型，PostgreSQL 会使用空格补齐长度。
    /// 查询中的 `RTRIM` 用于去除这些尾部填充空格，并转换成 `text`。
    pub async fn find_all(&self) -> Result<Vec<Student>, sqlx::Error> {
        sqlx::query_as::<_, Student>(
            r#"
            SELECT
                id,
                RTRIM(stu_name)::text AS stu_name,
                stu_age
            FROM public.stu3
            ORDER BY id
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// 使用 SQL 参数绑定查询单个学生，避免把 ID 拼接进 SQL。
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Student>, sqlx::Error> {
        sqlx::query_as::<_, Student>(
            r#"
            SELECT
                id,
                RTRIM(stu_name)::text AS stu_name,
                stu_age
            FROM public.stu3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}
