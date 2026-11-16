use crate::{models::Student, service::StudentService};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};

/// 注册学生查询接口，并将共享服务注入所有 Handler。
pub fn create_router(service: StudentService) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/students", get(get_students))
        .route("/students/:id", get(get_student_by_id))
        .with_state(service)
}

async fn health() -> &'static str {
    "OK"
}

/// 查询 `stu3` 全表。成功返回 JSON，数据库错误返回 500。
async fn get_students(
    State(service): State<StudentService>,
) -> Result<Json<Vec<Student>>, StatusCode> {
    service
        .get_students()
        .await
        .map(Json)
        .map_err(internal_server_error)
}

/// 根据路径中的 ID 查询学生；不存在时返回 404。
async fn get_student_by_id(
    State(service): State<StudentService>,
    Path(id): Path<i64>,
) -> Result<Json<Student>, StatusCode> {
    service
        .get_student_by_id(id)
        .await
        .map_err(internal_server_error)?
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json)
}

/// 服务端记录详细数据库错误，但只向客户端暴露通用的 500 状态码，
/// 避免连接信息或 SQL 细节泄漏到 HTTP 响应。
fn internal_server_error(error: sqlx::Error) -> StatusCode {
    eprintln!("数据库查询失败: {error}");
    StatusCode::INTERNAL_SERVER_ERROR
}
