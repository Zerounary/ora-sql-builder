# 性能测试方法

## 1. 目的

本文档用于统一 `ora-sql-builder` 的性能测试口径，重点观察三层开销：

- `engine`：SQL 构建与方言渲染
- `metadata`：request / plan / SQL 翻译
- `execution`：真实数据库执行与结果解码

统一关注两类指标：

- **时间开销**：总耗时、平均耗时、分阶段耗时
- **空间开销**：结构体静态大小、进程工作集内存

## 2. 基本原则

- 使用固定业务样本，不随意改测试数据。
- 先测单层，再测端到端。
- 时间与空间分开记录。
- 每轮重构后用同一套命令重复采样，形成可比较基线。

## 3. 当前可复用入口

### 3.1 engine 层

已有测试：

- 文件：`src/engine/tests.rs`
- 用例：`concurrent_builder_performance_report`

作用：

- 输出 `select / insert / update / delete / ddl_create` 的并发构建耗时
- 输出 `SelectBuilder`、`Predicate`、`BuiltQuery` 等结构体大小

运行命令：

```powershell
cargo test concurrent_builder_performance_report -- --ignored --nocapture
```

### 3.2 metadata 层

可复用样本：

- `src/metadata_demo.rs`
- `tests/metadata_stability.rs`

固定场景：

- `order_query_request()`
- `order_export_request()`
- `direct_order_insert_request(...)`
- `imported_order_insert_request(...)`
- `order_update_request(...)`
- `order_delete_request(...)`

当前建议先用稳定性测试统计总耗时：

```powershell
Measure-Command { cargo test metadata_stability -- --nocapture }
```

### 3.3 execution 层

可复用真实执行入口：

- `tests/metadata_stability.rs`
- 用例：`sqlite_execution_flow_covers_query_filter_relation_create_import_update_delete_export`

运行命令：

```powershell
Measure-Command { cargo test sqlite_execution_flow_covers_query_filter_relation_create_import_update_delete_export -- --nocapture }
```

## 4. 推荐统计指标

### 4.1 时间指标

- `total_ms`
- `avg_ns_per_op`
- `longest_thread_ms`
- 分阶段耗时：
  - request 构造
  - plan 构造
  - SQL 构建
  - 执行

### 4.2 空间指标

- `size_of::<T>()`
- `WorkingSet64`
- `PeakWorkingSet64`

## 5. engine 层怎么测

建议记录：

- `select`
- `insert`
- `update`
- `delete`
- `ddl_create`

结果表：

```markdown
| layer  | case       | total_ops | total_elapsed_ms | longest_thread_ms | avg_ns_per_op |
|--------|------------|-----------|------------------|-------------------|---------------|
| engine | select     |           |                  |                   |               |
| engine | insert     |           |                  |                   |               |
| engine | update     |           |                  |                   |               |
| engine | delete     |           |                  |                   |               |
| engine | ddl_create |           |                  |                   |               |
```

## 6. metadata 层怎么测

建议分三段：

### 6.1 request / plan 构造

建议后续补忽略测试，统计：

- `MetadataField`
- `MetadataQueryRequest`
- `QueryPlan`
- `WritePlan`
- `DeletePlan`

同时输出：

- `size_of::<MetadataField>()`
- `size_of::<MetadataQueryRequest>()`
- `size_of::<QueryPlan>()`
- `size_of::<WritePlan>()`

### 6.2 metadata -> SQL 翻译

核心口径：

- `MetadataSqlDriver::new(request).build(dialect)`

建议至少采样：

- query
- export
- insert
- import
- update
- delete

### 6.3 metadata 端到端

固定使用 `metadata_demo.rs` 场景，统计：

- request
- plan
- SQL build
- SQLite execute

## 7. execution 层怎么测

建议拆分记录：

- schema 创建
- seed 数据准备
- direct insert
- import insert
- query
- update
- export
- delete

如果暂时不加新 benchmark，可先统计总耗时。

## 8. Windows 下的内存采集

### 8.1 静态大小

通过测试输出：

- `size_of::<SelectBuilder>()`
- `size_of::<BuiltQuery>()`
- `size_of::<MetadataQueryRequest>()`
- `size_of::<QueryPlan>()`

### 8.2 运行中工作集

```powershell
Get-Process cargo | Select-Object Id, ProcessName, WorkingSet64, PeakWorkingSet64
```

建议记录：

- 测试开始前工作集
- 运行中峰值工作集
- 测试结束后工作集

## 9. 推荐结果表

### 9.1 时间表

```markdown
| layer     | scenario      | total_ms | avg_ns_per_op | remarks |
|-----------|---------------|----------|---------------|---------|
| engine    | select        |          |               |         |
| engine    | insert        |          |               |         |
| metadata  | query_build   |          |               |         |
| metadata  | update_build  |          |               |         |
| execution | sqlite_query  |          |               |         |
| execution | sqlite_write  |          |               |         |
| e2e       | metadata_flow |          |               |         |
```

### 9.2 空间表

```markdown
| layer    | item                 | static_size_bytes | working_set_before | peak_working_set | working_set_after |
|----------|----------------------|-------------------|--------------------|------------------|-------------------|
| engine   | SelectBuilder        |                   |                    |                  |                   |
| engine   | BuiltQuery           |                   |                    |                  |                   |
| metadata | MetadataQueryRequest |                   |                    |                  |                   |
| metadata | QueryPlan            |                   |                    |                  |                   |
```

## 10. 推荐执行顺序

1. 跑 `engine` 并发构建性能测试。
2. 跑 `metadata_stability` 统计总耗时。
3. 跑 SQLite 真实执行场景统计总耗时。
4. 如果有退化，再下钻到具体 case。

## 11. 当前阶段结论

当前阶段最重要的不是得到绝对精确的 benchmark 数字，而是建立一套**固定场景、固定命令、固定表格**的性能回归方法。只要持续按这套方法记录，就能逐步形成项目自己的时间与空间基线。
