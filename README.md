# Rust PostgreSQL REST 项目说明

## 1. 项目概述

这是一个使用 Rust、Axum、Tokio、Serde 和 SQLx 编写的只读学生查询服务。数据来自 PostgreSQL 数据库 `wf` 的 `public.stu3` 表，不再使用内存模拟数据。

程序提供：

- `GET /health`：健康检查；
- `GET /students`：查询 `stu3` 全部数据；
- `GET /students/:id`：根据主键查询单个学生。

## 2. 项目结构

```text
rust-demo/
├── Cargo.toml
├── Cargo.lock
├── RUST_CODE_GUIDE.md
└── src/
    ├── main.rs       # 读取配置、连接数据库、启动 HTTP 服务
    ├── models.rs     # PostgreSQL 行与 JSON 响应模型
    ├── routes.rs     # 路由、请求参数和 HTTP 响应
    ├── service.rs    # 学生业务服务
    └── storage.rs    # SQLx 连接池和 SQL 查询
```

依赖方向：

```text
HTTP 请求
   ↓
routes 路由层
   ↓
service 服务层
   ↓
storage PostgreSQL 存储层
   ↓
models 数据模型
```

## 3. 数据库表结构

程序读取：

```text
主机：121.40.244.102
端口：5432
数据库：wf
模式：public
表名：stu3
```

已确认的表结构：

| 字段 | PostgreSQL 类型 | 是否允许 NULL | Rust 类型 |
| --- | --- | --- | --- |
| `id` | `bigint` | 否 | `i64` |
| `stu_name` | `character(10)` | 是 | `Option<String>` |
| `stu_age` | `text` | 是 | `Option<String>` |

`id` 是主键。由于另外两个字段允许 `NULL`，Rust 模型不能直接使用 `String`，需要用 `Option<String>` 区分“有值”和“数据库空值”。

## 4. 各层说明

### 4.1 模型层

`models.rs` 中的 `Student` 同时承担 SQL 查询结果和 JSON 响应模型：

```rust
#[derive(Debug, Serialize, FromRow)]
pub struct Student {
    pub id: i64,
    pub stu_name: Option<String>,
    pub stu_age: Option<String>,
}
```

- `Serialize`：把学生转换为 JSON；
- `FromRow`：让 SQLx 按字段名把 PostgreSQL 查询结果映射为 `Student`。

### 4.2 存储层

`storage.rs` 使用 `PgPool` 管理 PostgreSQL 连接：

```rust
pub struct StudentStore {
    pool: PgPool,
}
```

连接池最多维护 5 个连接。`PgPool` 可以低成本克隆，多个请求共享同一个底层连接池，不会为每次请求重新连接数据库。

查询全部学生的核心 SQL：

```sql
SELECT
    id,
    RTRIM(stu_name)::text AS stu_name,
    stu_age
FROM public.stu3
ORDER BY id
```

这里没有直接使用字面上的 `SELECT *`，原因是 `stu_name` 为 `char(10)`，数据库会用空格补齐固定长度。`RTRIM` 会去掉尾部填充空格，API 返回 `"san"` 而不是 `"san       "`。查询字段仍覆盖 `stu3` 的全部三列。

按 ID 查询使用：

```sql
SELECT
    id,
    RTRIM(stu_name)::text AS stu_name,
    stu_age
FROM public.stu3
WHERE id = $1
```

`$1` 通过 SQLx `.bind(id)` 绑定，没有字符串拼接，可避免 SQL 注入。

### 4.3 服务层

`StudentService` 隔离路由和存储：

- `get_students()`：获取全部学生；
- `get_student_by_id(id)`：按 ID 获取学生。

服务层不依赖 Axum 的状态码。SQLx 错误向上传递，由 HTTP 路由层决定如何响应。

### 4.4 路由层

路由层使用 Axum Extractor：

- `State<StudentService>`：取得共享服务；
- `Path<i64>`：解析路径中的学生 ID。

响应规则：

| 场景 | HTTP 状态码 |
| --- | --- |
| 查询成功 | `200 OK` |
| ID 不存在 | `404 Not Found` |
| PostgreSQL 连接或查询失败 | `500 Internal Server Error` |
| 路径 ID 不是合法 `i64` | Axum 参数解析错误 |

数据库错误的详细内容只写入服务端标准错误输出，客户端只收到 500，避免泄露 SQL 或连接信息。

### 4.5 程序入口

`main.rs` 的执行过程：

1. 从 `DATABASE_URL` 环境变量读取连接地址；
2. 创建 PostgreSQL 连接池；
3. 创建 `StudentStore`；
4. 将存储注入 `StudentService`；
5. 将服务注入 Axum Router；
6. 监听 `127.0.0.1:8081`；
7. 启动 HTTP 服务。

数据库密码不会写死在 Rust 源码中。

## 5. 配置与启动

### 5.1 配置连接地址

在终端中设置环境变量：

```bash
export DATABASE_URL='postgres://postgres:<数据库密码>@121.40.244.102:5432/wf'
```

将 `<数据库密码>` 替换成实际密码。不要把包含真实密码的连接地址提交到 Git。

如果密码含有 `@`、`:`、`/`、`?`、`#` 等特殊字符，需要先进行 URL 编码。

### 5.2 启动服务

```bash
cargo run
```

成功后监听：

```text
http://127.0.0.1:8081
```

服务绑定 `127.0.0.1`，默认只能在运行程序的机器上访问。

## 6. 接口说明

### 6.1 健康检查

```bash
curl -i http://127.0.0.1:8081/health
```

响应体：

```text
OK
```

当前健康检查只确认 HTTP 服务正在运行，不会额外执行数据库查询。

### 6.2 查询全部学生

```bash
curl -i http://127.0.0.1:8081/students
```

响应示例：

```json
[
  {
    "id": 1,
    "stu_name": "san",
    "stu_age": "11"
  },
  {
    "id": 2,
    "stu_name": "wang",
    "stu_age": "12"
  },
  {
    "id": 3,
    "stu_name": "大哥",
    "stu_age": "9"
  }
]
```

### 6.3 根据 ID 查询

```bash
curl -i http://127.0.0.1:8081/students/1
```

找到时返回：

```json
{
  "id": 1,
  "stu_name": "san",
  "stu_age": "11"
}
```

访问不存在的 ID：

```bash
curl -i http://127.0.0.1:8081/students/999
```

返回 `404 Not Found`。

## 7. 常用验证命令

```bash
cargo fmt -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

直接验证数据库：

```bash
psql -h 121.40.244.102 -p 5432 -U postgres -d wf \
  -c 'SELECT * FROM public.stu3 ORDER BY id;'
```

`psql` 会交互式要求输入密码，避免把密码写进 shell 历史。

## 8. 当前边界与后续建议

1. 当前服务只有查询接口，不会修改 `stu3` 数据；
2. `/health` 未检测数据库状态，可增加 `/ready` 执行 `SELECT 1`；
3. 数据库连接参数只有 `DATABASE_URL`，可进一步支持连接池大小和超时配置；
4. 错误响应体为空，可增加统一的 JSON 错误模型；
5. 当前使用 `eprintln!`，生产环境建议改为 `tracing` 结构化日志；
6. 可以使用 mock 数据库或独立测试库增加存储层集成测试；
7. 如果查询量较大，`GET /students` 应增加分页参数；
8. 部署时应通过密钥管理系统注入数据库密码，不应保存在源码或镜像中。
