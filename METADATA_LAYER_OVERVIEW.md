# 元数据层概述

## 目标定位

`metadata` 层是项目的语义中心，负责用平台无关的方式描述系统结构、字段能力、关系、策略和运行时请求。

这层不直接执行 SQL，也不管理连接池，而是回答两个核心问题：

- 系统里有什么对象、字段、关系和治理规则。
- 当前请求希望对这些对象做什么。

如果说：

- `engine` 负责“SQL 怎么生成”；
- `execution` 负责“SQL 怎么执行”；

那么 `metadata` 负责的就是“业务语义如何被结构化表达”。

## 当前结构

元数据相关代码主要分布在以下模块：

- **`src/metadata/`**
  核心元数据模型定义，包含字段、过滤器、目录实体、标准 schema 等基础结构。

- **`src/metadata_driver/`**
  将 `MetadataQueryRequest` 转换为引擎层可消费的 SQL 构建输入和 `BuiltQuery`。

- **`src/metadata_plan/`**
  将请求整理为更靠近执行阶段的计划模型，如查询计划、写入计划、删除计划、权限计划和模式计划。

- **`src/metadata_mapping/`**
  将目录对象映射为标准元数据表的持久化快照，支撑 metadata 自身落库。

- **`src/metadata_demo.rs`**
  提供示例场景和稳定性验证样本，用于验证 metadata 设计的可用性与一致性。

## 元数据子域

### 目录域

目录域定义平台级元数据实体，例如：

- `MetaDatasource`
- `MetaTable`
- `MetaColumn`
- `MetaRelation`
- `MetaPolicy`
- `MetaImportProfile`
- `MetaExportProfile`
- `MetadataCatalog`

这些类型回答的是：**系统结构与治理模型是什么。**

### 请求域

请求域由 `MetadataQueryRequest`、`MetadataField`、`MetadataFilterExpr`、`MetadataQueryOptions` 等组成。

它回答的是：**调用者当前要查什么、写什么、过滤什么。**

### 模式域

`MetadataTableSchema`、`MetadataColumnSchema`、`MetadataForeignKeySchema` 和 `standard_metadata_tables()` 用于回答：**元数据本身应该如何标准化存储。**

### 运行时规划域

`metadata_plan` 负责把请求整理为更适合执行层消费的计划模型。

它回答的是：**这次操作涉及哪些字段、关系、权限和写入规则。**

### 持久化映射域

`metadata_mapping` 将目录对象转换为标准元数据持久化快照。

它回答的是：**目录对象如何映射到标准化 metadata 表集。**

## 核心设计约束

- **元数据是行为来源**
  可查询字段、可写字段、导入导出能力和治理规则应尽量由元数据推导，而不是散落在执行代码里。

- **请求必须保持结构化**
  不要过早把请求折叠成原始 SQL，否则后续计划、权限和兼容性都会变差。

- **目录与请求必须分层**
  目录描述系统静态结构；请求描述一次动态操作，二者不能混写。

- **持久化映射必须确定**
  相同目录应生成相同快照，方便回归测试和后续迁移。

## 典型数据流

1. 先用 catalog 模型描述系统结构。
2. 再用 request 模型描述本次操作。
3. `metadata_plan` 抽取运行时计划信息。
4. `metadata_driver` 把请求翻译成引擎层 SQL 输出。
5. 如需落库 metadata 本身，则通过 `metadata_mapping` 生成标准持久化快照。
6. 最终由 `execution` 选择是否执行。

## 这层负责什么

- 平台级元数据实体。
- 请求模型与过滤表达式。
- 标准 metadata 存储 schema。
- 目录到持久化快照的映射。
- 元数据驱动的 SQL 翻译入口。
- 示例数据与稳定性验证基线。

## 这层不负责什么

- 真实数据库连接与连接池。
- 事务与驱动安装。
- SQL 参数绑定与结果解码。
- 环境基础设施和部署逻辑。

## 扩展建议

新增业务能力时，建议按以下顺序推进：

1. 先判断这是目录建模问题、请求表达问题，还是计划/持久化问题。
2. 优先扩展 `metadata` 模型本身，再决定是否需要调整 `metadata_plan`。
3. 只有元数据契约稳定后，再修改 `metadata_driver`。
4. 同步补 `metadata_demo.rs` 和稳定性测试，避免设计漂移。

## 常见风险点

- 混淆 catalog 语义与 request 语义。
- 把业务规则直接写死到执行层，而不是通过 metadata 表达。
- 让示例数据结构反向污染通用模型设计。
- 让持久化映射依赖不稳定的字符串约定。

## 一句话总结

`metadata` 层不是 SQL 层的附属物，而是整个系统“可配置、可扩展、可由 AI 驱动生成”的核心抽象层。
