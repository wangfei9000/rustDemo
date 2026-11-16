use serde::Serialize;
use sqlx::FromRow;

/// `public.stu3` 表对应的学生响应模型。
///
/// PostgreSQL 的 `bigint` 映射为 Rust 的 `i64`。
/// `stu_name` 和 `stu_age` 在表中允许为空，因此使用 `Option<String>`。
#[derive(Debug, Serialize, FromRow)]
pub struct Student {
    pub id: i64,
    pub stu_name: Option<String>,
    pub stu_age: Option<String>,
}
