# Metadata 层测试方法

## 1. 文档目的

本文档用于说明当前 `ora-sql-builder` 项目中，`metadata` 层相关测试是如何组织、如何运行、覆盖什么范围，以及在持续重构过程中应如何使用这些测试保护系统外部行为稳定。

这份文档的核心目标不是追求“测试数量越多越好”，而是建立一套可复用、可回归、可随架构演进持续保真的测试方法。

## 2. 测试设计原则

当前 metadata 层测试遵循以下原则：

- **先保护外部行为，再允许内部重构**
  只要对外 SQL 预览行为和关键运行时行为保持稳定，内部模块可以继续拆分和演进。

- **优先覆盖真实业务场景，而不是零碎函数**
  metadata 的价值在于把业务能力结构化表达出来，因此测试应围绕查询、导入、更新、删除、导出等完整场景展开。

- **预览层与执行层分开验证**
  一部分测试验证“SQL 能否正确生成”；另一部分测试验证“SQL 是否真的可以在目标数据库执行”。

- **跨方言预览 + 单数据库真实执行**
  当前通过所有支持数据库的 SQL 预览，验证方言兼容性；通过 SQLite 内存执行，验证端到端运行路径。

## 3. 测试依赖的基础数据

### 3.1 支持数据源文件

当前支持数据源清单定义在：

- `examples/supported_datasources.json`

该文件为每种当前支持的数据库准备了一条示例数据源记录，包括：

- `mysql`
- `postgres`
- `oracle`
- `sqlserver`
- `sqlite`

这个文件的用途包括：

- 示例演示
- 跨方言 SQL 预览验证
- 支持数据库清单回归检查

其中，**只有 SQLite** 在自动化测试里被用于真实执行，其他数据库当前主要用于 SQL 预览稳定性验证。

### 3.2 演示业务模型

metadata 示例业务模型定义在：

- `src/metadata_demo.rs`

当前示例使用一个简化销售域，包含以下表：

- `demo_store`
- `demo_customer`
- `demo_sale_order`

这组三张表的设计目的，不是模拟完整 ERP，而是作为一套最小但足够完整的 metadata 业务样本，用于覆盖：

- 查询
- 过滤
- 关联查询
- 新增
- 导入
- 更新
- 删除
- 导出

## 4. 场景入口设计

为了避免测试代码到处手写 request，当前把可复用场景统一放在 `src/metadata_demo.rs` 中，形成稳定入口。

当前主要场景包括：

- `order_query_request()`
- `order_export_request()`
- `direct_order_insert_request(...)`
- `imported_order_insert_request(...)`
- `order_update_request(...)`
- `order_delete_request(...)`

这样设计的好处是：

- 场景定义集中，便于维护。
- 可以同时复用于单测、稳定性测试、示例与后续性能测试。
- 当 metadata 模型演进时，只需要在少数场景入口处调整，即可验证影响范围。

## 5. 当前测试文件与分层

当前 metadata 稳定性主测试文件位于：

- `tests/metadata_stability.rs`

它当前分为两层验证：

### 5.1 数据源清单回归验证

对应测试：

- `datasource_file_covers_all_supported_databases`

该测试用于确保 `supported_datasources.json` 中的数据源定义没有漏掉当前框架宣称支持的数据库类型。

它保护的不是 SQL 逻辑，而是“支持范围声明”和“示例基线”本身。

### 5.2 跨数据库 SQL 预览稳定性验证

对应测试：

- `metadata_preview_scenarios_cover_all_supported_dialects`

该层测试的目标是验证：

- 同一组 metadata 业务场景，是否能在所有受支持方言下成功生成 SQL。
- 生成的 SQL 是否保持非空、可构造。
- 如果存在参数，是否保持各数据库对应的占位符风格。

当前主要验证的业务场景有：

- query
- export
- insert
- import
- update
- delete

这一层不直接验证数据库执行结果，而是重点保护：

- `metadata_driver` 的翻译能力
- `engine` 的跨方言输出稳定性
- metadata 请求模型对多数据库输出的一致适配能力

### 5.3 SQLite 真实执行稳定性验证

对应测试：

- `sqlite_execution_flow_covers_query_filter_relation_create_import_update_delete_export`

该测试使用 SQLite 内存数据源，对完整链路进行真实执行验证。

它覆盖的关键步骤包括：

1. 通过 `SchemaPlanExecutor` 建立示例表结构。
2. 准备门店与客户基础数据。
3. 执行直接新增。
4. 执行导入式新增。
5. 执行查询并验证关联字段。
6. 执行更新。
7. 执行导出查询。
8. 执行删除。
9. 最终校验剩余数据状态。

这一层测试保护的是完整链路：

- `metadata_demo`
- `metadata_plan`
- `metadata_driver`
- `engine`
- `execution`

也就是说，它不是单一模块测试，而是 metadata 驱动路径的端到端稳定性回归测试。

## 6. 运行方式

### 6.1 运行全部测试

```powershell
cargo test
```

### 6.2 仅运行 metadata 稳定性测试

```powershell
cargo test metadata_stability -- --nocapture
```

### 6.3 仅检查 examples 可编译性

```powershell
cargo check --examples
```

## 7. 建议的日常使用方式

在不同开发阶段，建议采用不同测试组合：

- **改 metadata 模型定义时**
  至少运行 `cargo test metadata_stability -- --nocapture`

- **改 metadata_driver / metadata_plan / engine 交界逻辑时**
  运行：
  - `cargo test`
  - `cargo check --examples`

- **做大规模模块拆分或 API 重组时**
  运行全量测试，并重点关注 `metadata_stability.rs` 是否仍保持通过。

## 8. 推荐补充的测试维度

当前测试已经覆盖主链路，但后续仍建议逐步补以下维度：

- 更复杂的过滤 AST 组合场景。
- 更复杂的 grouped / having 场景。
- metadata catalog 持久化快照的一致性校验。
- import/export profile 对默认过滤和列映射的更细粒度验证。
- portal legacy 语义适配到 metadata 路径后的兼容测试。

## 9. 稳定性判断标准

在当前项目中，可以把以下结果视为 metadata 层“基本稳定”：

- 所有 metadata 场景在支持方言下都能成功生成 SQL。
- SQLite 端到端执行链路持续通过。
- examples 可以通过编译检查。
- metadata demo 场景修改后，不会引发大面积 SQL 行为漂移。

## 10. 本文档保护的演进边界

这套测试方法主要用于在以下区域持续重构时控制风险：

- metadata 实体定义
- metadata 请求模型
- metadata plan 结构
- metadata persistence mapping
- metadata 到 engine 的翻译路径
- execution 的 metadata 执行链路
- engine builder 的输出行为

只要这套测试持续为绿色，就说明 metadata 核心的外部行为仍然可控，项目就可以在较低风险下继续进行模块拆分和架构演进。
