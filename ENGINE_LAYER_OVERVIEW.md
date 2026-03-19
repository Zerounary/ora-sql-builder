# 引擎层概述

## 目标定位

`engine` 层是整个项目最底层、最通用的 SQL 生成核心。
它不理解“业务单据”“权限策略”“元数据目录”这些上层概念，只负责一件事：

- 将结构化的 SQL 构建输入转换为稳定、可预测、可跨方言切换的 SQL 输出。

从职责边界上看，`engine` 更像一个“SQL 语法装配器 + 方言渲染器”，而不是运行时执行器，也不是业务模型层。

它必须长期保持以下特征：

- **纯粹**：不直接依赖数据库连接、连接池、`sqlx` 或执行环境。
- **稳定**：相同输入在相同方言下必须得到相同 SQL 与相同参数顺序。
- **低耦合**：不感知 `metadata` 层中的业务术语，只接收通用 SQL 构造信息。
- **可复用**：既能被 `metadata_driver` 调用，也能被 examples、tests 或独立工具直接使用。

## 在整体架构中的位置

当前项目可以粗略分为三层：

1. `metadata` 层负责描述业务语义、请求模型、权限与元数据结构。
2. `engine` 层负责把结构化 SQL 意图渲染成真正的 SQL 文本与参数列表。
3. `execution` 层负责把 SQL 发送到具体数据库并处理结果。

因此，`engine` 层位于中间偏底部的位置：

- 它向上承接 `metadata_driver`、示例代码、直接构建器调用。
- 它向下只输出 `BuiltQuery`，不关心后续由谁执行。

## 当前目录结构

该层位于 `src/engine/` 下，当前已经按职责做了模块化拆分。

- **`mod.rs`**
  该目录的统一导出入口，对外暴露稳定 API。

- **`builders.rs`**
  聚焦 DML 相关构建器，例如 `SelectBuilder`、`InsertBuilder`、`UpdateBuilder`、`DeleteBuilder`。
  这里定义的是“要构造什么 SQL”，而不是“如何执行 SQL”。

- **`ddl.rs`**
  聚焦模式变更相关的 DDL 构建，例如 `CreateTableBuilder`、`AlterTableBuilder`、`DropTableBuilder`，以及列定义、外键定义等结构。

- **`dialect.rs`**
  定义 SQL 方言抽象与具体实现，包括 `PostgresDialect`、`MySqlDialect`、`OracleDialect`、`SqlServerDialect`、`SqliteDialect`。
  这里负责处理不同数据库在占位符、分页、DDL 语法上的差异。

- **`facade.rs`**
  `MetaSqlEngine` 所在位置。它是引擎层的对外门面，负责把构建器对象与方言组合起来，输出最终的 `BuiltQuery`。

- **`query.rs`**
  放置查询构建过程中的共享数据结构，例如 `BuiltQuery`、`Predicate`、`Relation`、`JoinType`、`Pagination`、`TableRef`。

- **`tests.rs`**
  引擎层自身的测试入口，覆盖多方言 SQL 生成、边界行为、分页、DDL 与参数顺序稳定性。

## 引擎层负责什么

以下职责应明确属于 `engine` 层：

- **SQL 语句装配**
  例如 SELECT 的字段列表、JOIN、WHERE、GROUP BY、HAVING、ORDER BY、LIMIT/OFFSET 等结构化拼装。

- **参数占位符编排**
  不同数据库对占位符的要求不同，例如 `$1`、`?`、`:1` 等，这些都应由方言层统一控制。

- **DML 与 DDL 输出统一建模**
  无论是查询、更新、删除，还是建表、删表、改表，都应尽量通过统一的 builder/definition 模型表达。

- **边界条件安全降级**
  例如空 `IN` 列表、空条件、方言分页回退等情况，应产生安全而非无效的 SQL。

- **原始 SQL 片段的受控接入**
  如 `sysdate`、`get_sequenceno(...)` 这类表达式不能到处拼接，必须通过显式 raw API 引入。

## 引擎层不负责什么

以下职责不应下沉到 `engine` 层：

- **业务权限判断**
  例如字段是否可见、哪些字段允许写入、租户过滤规则等，应由 `metadata` 层决定。

- **数据源与连接池管理**
  引擎层不应该知道数据库连接从哪里来，也不应该接触 `DatasourceManager`。

- **结果读取与数据解码**
  行数据如何反序列化、如何转成 JSON、如何做类型兼容，是 `execution` 层的职责。

- **业务语义建模**
  例如“门店”“零售单”“导入模板”“导出规则”这些概念不应直接出现于引擎层 API。

## 核心设计约束

为了让上层持续稳定依赖 `engine`，以下约束需要长期保持。

- **方言差异必须集中处理**
  所有数据库差异都应尽量聚焦在 `dialect.rs` 或与方言直接相关的渲染流程中，避免上层拼接方言分支。

- **参数顺序必须完全可预测**
  Builder 添加值的顺序、谓词顺序、JOIN 条件顺序，都直接影响参数顺序。这个顺序一旦变化，调用侧测试也会跟着波动。

- **Builder API 应该表达语义，而不是表达字符串细节**
  当你想新增某种 SQL 能力时，优先扩展 `Predicate`、`Relation`、`TableRef`、Builder 方法，而不是在上层直接塞字符串。

- **Raw SQL 必须是例外，不应成为主路径**
  Raw 接口存在是为了兼容数据库函数、序列或特殊表达式，但不能把整层退化成“字符串模板系统”。

## 典型数据流

一个典型的调用链如下：

1. 上层模块准备结构化输入，例如 `SelectBuilder` 或 `InsertBuilder`。
2. `MetaSqlEngine` 选择某个具体方言进行渲染。
3. 方言负责输出数据库兼容的 SQL 文本。
4. 渲染结果被封装为 `BuiltQuery`。
5. 上层模块决定是仅预览 SQL，还是交给 `execution` 层执行。

结合当前项目，最常见的实际链路是：

- `metadata_driver` 根据 `MetadataQueryRequest` 构建 builder。
- `engine` 把 builder 渲染成 SQL。
- `execution` 在运行时把 `BuiltQuery` 绑定并发送到数据库。

## 典型扩展方式

如果你要在引擎层新增能力，建议按下面顺序推进：

1. **先判断是“建模问题”还是“渲染问题”**
   如果缺的是表达能力，应先加 builder/query model；如果缺的是某个数据库的输出样式，再动方言渲染。

2. **优先扩展已有 builder，而不是新建平行 builder**
   例如新增 HAVING、EXISTS、复杂 JOIN 语义时，优先看能否在现有 builder 体系里自然表达。

3. **跨方言行为必须一起验证**
   只在某一个数据库上“能跑”不够。只要影响占位符、分页、DDL 或函数表达，就应补跨方言断言。

4. **补上边界条件测试**
   新功能除了 happy path，还应覆盖空值、空列表、原始表达式、分页叠加、排序缺失等情况。

## 适合在这里加测试的场景

以下变化最适合直接在 `engine/tests.rs` 中加测试：

- 新增一种谓词或逻辑组合方式。
- 修改不同数据库的分页行为。
- 调整参数顺序生成逻辑。
- 新增 DDL 输出能力。
- 修复空 `IN`、空排序、空分组等边界问题。

## 常见风险点

- **把元数据语义硬塞进 builder**
  一旦 builder 开始理解“字段权限”“元数据目录”，层次边界就会被打穿。

- **跨方言行为不一致**
  某个改动可能只在 PostgreSQL 看起来正确，但在 SQL Server、Oracle、SQLite 上失效。

- **参数顺序被隐式改变**
  这类问题最隐蔽，SQL 文本可能看起来没问题，但执行时参数会错位。

- **Raw 接口被滥用**
  如果上层越来越多依赖 raw SQL，说明 builder 抽象可能需要补能力，而不是继续放任字符串拼接。

## 演进建议

从当前代码形态看，`engine` 层已经具备比较清晰的模块边界，后续建议持续遵守以下原则：

- 新能力先抽象，再渲染。
- 方言差异集中，不向上泄漏。
- 保持 `BuiltQuery` 作为统一输出。
- 每次改动都用测试守住 SQL 形状和参数顺序。

如果这层保持稳定，上层 `metadata` 和下层 `execution` 都可以更轻松地演进，而不会相互牵连。
