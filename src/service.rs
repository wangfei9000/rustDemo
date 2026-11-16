use crate::{models::Student, storage::StudentStore};

/// 学生业务服务，隔离 HTTP 路由和 PostgreSQL 存储实现。
#[derive(Clone)]
pub struct StudentService {
    store: StudentStore,
}

impl StudentService {
    pub fn new(store: StudentStore) -> Self {
        Self { store }
    }

    /// 获取全部学生。数据库错误继续交给路由层转换为 HTTP 错误。
    pub async fn get_students(&self) -> Result<Vec<Student>, sqlx::Error> {
        self.store.find_all().await
    }

    /// 按 ID 获取学生；未找到时返回 `Ok(None)`。
    pub async fn get_student_by_id(&self, id: i64) -> Result<Option<Student>, sqlx::Error> {
        self.store.find_by_id(id).await
    }
}
