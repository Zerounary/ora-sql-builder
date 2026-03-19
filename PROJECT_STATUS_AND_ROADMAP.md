# ora-sql-builder 项目现状与后续路线

## 1. 项目定位

当前仓库已经从一个以 `portal_provider` 为核心的字符串式 SQL 生成实现，逐步演进为一个以 `engine` 为底层能力、以强类型元数据模型为上层抽象的 SQL 构建基础库。

现阶段项目的核心目标已经从“只生成一段可用 SQL”转向“为未来的数据层 + 跨库 CRUD 自动生成平台提供稳定、可测试、可扩展的底座”。

---

## 2. 当前完成情况

### 2.1 已完成的核心模块

#### 2.1.1 `sql` 模块

`src/sql.rs` 仍保留原有 `SQLProvider` / `SQLExplorer` / `SQLStatement` 体系，作为兼容旧逻辑的基础层。

当前状态：

- 已抽取 `populate_statement`，消除重复装配逻辑
- 已移除无效状态字段，简化结构
- 已补充逻辑连接符和 JOIN 渲染相关测试
- 仍主要服务于旧的 `portal_provider` 输出模式

#### 2.1.2 `portal_provider` 模块

`src/portal_provider.rs` 仍是旧元数据 SQL 生成逻辑的主要承载体。

当前状态：

- 已完成一定程度的结构化重构
- 已抽取重复的表名拼装、过滤、插入值处理、更新赋值处理等辅助逻辑
- 已补充过滤清洗、布尔过滤、数组过滤、between 过滤、分组排序等测试
- 仍存在历史设计包袱：弱类型字段较多、字符串约定较多、模块仍偏大

#### 2.1.3 `engine` 模块

`src/engine.rs` 已经完成拆分，当前结构为：

- `src/engine/dialect.rs`
- `src/engine/query.rs`
- `src/engine/builders.rs`
- `src/engine/facade.rs`

当前能力：

- 支持 MySQL / PostgreSQL / Oracle / SQL Server / SQLite 五类方言
- 支持 `SELECT / INSERT / UPDATE / DELETE`
- 支持参数化 SQL
- 支持 JOIN、分页、排序、分组
- 支持原始 SQL 表达式写入
- 支持更完整的过滤谓词：
  - `=`
  - `!=`
  - `>`
  - `>=`
  - `<`
  - `<=`
  - `LIKE`
  - `IN`
  - `BETWEEN`
  - `EXISTS`
  - `RAW`
  - `CUSTOM(SQL + params)`

#### 2.1.4 新元数据层 `metadata`

`src/metadata.rs` 已建立新的强类型元数据模型，目标是替代 `portal_provider` 中大量弱类型、字符串驱动的配置方式。

当前模型已经包含：

- `CapabilityMask`
- `FieldInputKind`
- `SortDirection`
- `LookupReference`
- `LinkStep`
- `LinkReference`
- `FieldSource`
- `MetadataField`
- `MetadataQueryOptions`
- `MetadataQueryRequest`

这套模型已经能够描述：

- 直接字段
- 公式字段
- 已限定表达式字段
- 链式关联字段
- 引用查询字段
- 排序与过滤输入
- 序列号字段
- 默认值与实际值

#### 2.1.5 新元数据驱动核心 `metadata_driver`

`src/metadata_driver.rs` 已拆分为：

- `src/metadata_driver/context.rs`
- `src/metadata_driver/driver.rs`
- `src/metadata_driver/filters.rs`
- `src/metadata_driver/helpers.rs`

当前能力：

- 使用 `engine` 作为底层构建引擎
- 支持 `SELECT / INSERT / UPDATE / DELETE`
- 支持 lookup 字段投影
- 支持链式 JOIN 生成
- 支持布尔 / 数组 / between / 分词 like 过滤
- 支持序列号、默认值、原始 SQL 表达式写入
- 支持分组与排序
- 支持参数化输出

该模块已经具备替代 `portal_provider` 的雏形能力，但目前仍处于“新核心已可用、尚未全面接管旧入口”的阶段。

---

## 3. 当前目录与职责划分

### 3.1 现有主要模块

- `src/sql.rs`
  - 旧的 SQLProvider 装配和 SQL 字符串输出层
- `src/portal_provider.rs`
  - 旧的元数据驱动 SQL 生成实现
- `src/engine/`
  - 新的底层 SQL 构建引擎
- `src/metadata.rs`
  - 新的强类型元数据模型
- `src/metadata_driver/`
  - 基于新元数据层和 engine 的新驱动核心

### 3.2 当前架构关系

当前项目处于“双轨并存”阶段：

- 旧轨：`portal_provider -> sql`
- 新轨：`metadata -> metadata_driver -> engine`

这意味着：

- 旧逻辑仍可继续工作
- 新逻辑已经可以承载未来演进
- 当前仓库已经具备渐进式迁移条件

---

## 4. 当前测试与示例情况

### 4.1 测试现状

当前项目已具备较好的单元测试基础，覆盖了：

- `engine` 方言与 CRUD 构建能力
- JOIN / 分页 / 参数顺序
- 原始 SQL 表达式写入
- 高级过滤谓词
- `sql` 旧装配层逻辑
- `portal_provider` 的关键历史行为
- `metadata_driver` 的新元数据驱动行为

当前验证状态：

- `cargo test` 通过
- 当前测试总数：**38 个**
- 结果：**38 passed, 0 failed**

### 4.2 examples 现状

当前已提供以下示例：

- `examples/mysql_crud.rs`
- `examples/postgres_crud.rs`
- `examples/oracle_crud.rs`
- `examples/sqlserver_crud.rs`
- `examples/sqlite_crud.rs`
- `examples/metadata_driver.rs`

其中数据库示例已展示：

- 查询
- 过滤
- JOIN
- 分页
- 新增
- 修改
- 删除
- 原始 SQL 表达式写入

过滤示例覆盖：

- `=`
- `!=`
- `>`
- `>=`
- `<`
- `<=`
- `LIKE`
- `IN`
- `BETWEEN`
- `EXISTS`

当前验证状态：

- `cargo check --examples` 通过

---

## 4.3 当前项目支持数据库范围

当前 `engine` 与 examples 已覆盖以下关系型数据库方言：

- MySQL
- PostgreSQL
- Oracle
- SQL Server
- SQLite

SQLite 的接入意义不仅是补齐一个方言，更重要的是为未来：

- 轻量级嵌入式部署
- 本地开发 / 演示环境
- 单机版元数据管理后台
- 自动化测试与快速集成验证

提供一个低成本运行基座。

---

## 5. 当前项目仍存在的不足

### 5.1 旧入口仍未完成迁移

虽然新元数据层和新驱动核心已经建立，但 `portal_provider` 仍是旧体系的重要入口，尚未完成完全迁移。

### 5.2 元数据模型还偏“查询构建视角”

当前 `metadata` 更偏向 SQL 构建所需的数据结构，还不是完整的平台级元数据模型。

缺失方向包括：

- 表级元数据
- 字段标准类型抽象
- 主键策略
- 约束模型
- 索引模型
- 关系模型
- 逻辑删除策略
- 多租户策略
- 行级权限 / 字段权限策略
- 数据源信息模型

### 5.3 缺少执行层

当前仓库已具备“生成 SQL”的能力，但还没有真正进入：

- 多数据源管理
- 连接池
- SQL 执行
- 事务管理
- 统一 CRUD 服务
- API 层调用入口

### 5.4 缺少平台级建模和搜索能力

距离“跨库 CRUD 自动生成平台”还缺少：

- 元数据表标准设计
- 元数据持久化
- 元数据搜索与管理
- 平台级 CRUD 自动生成服务
- 可视化与开放接口承接层

---

## 6. 后续设计目标：数据层 + 跨库 CRUD 自动生成平台

后续目标不是继续堆积局部 SQL 拼装能力，而是围绕“统一数据层平台”向上建设。

目标平台应具备以下能力：

- 元数据统一建模
- 跨库 SQL 自动生成
- 自动 CRUD 服务生成
- 动态关联查询
- 字段级规则控制
- 多数据源切换
- SQL 预览与执行分离
- 平台级权限与审计

---

## 7. 建议的未来总体架构

### 7.1 分层建议

建议将未来系统拆为以下层次：

#### 7.1.1 元数据模型层

负责定义平台级标准模型：

- 数据源
- 表
- 字段
- 关系
- 约束
- 索引
- 权限
- 策略

#### 7.1.2 元数据访问层

负责：

- 读取元数据
- 缓存元数据
- 搜索元数据
- 校验元数据完整性
- 构建运行时上下文

#### 7.1.3 查询规划层

负责把平台级元数据请求转成统一的“查询计划”：

- 投影计划
- 过滤计划
- JOIN 计划
- 分组计划
- 排序计划
- 写入计划
- 主键 / 序列 / 默认值策略计划

#### 7.1.4 SQL 构建层

由 `engine` 继续承接，但应进一步增强为可被查询计划直接消费的构建层。

#### 7.1.5 执行服务层

负责：

- 多数据源路由
- 方言选择
- SQL 执行
- 事务管理
- 统一结果封装

#### 7.1.6 API / 平台层

负责：

- 通用 CRUD 接口
- 元数据管理接口
- 搜索接口
- SQL 预览接口
- 平台扩展接口

---

## 8. 未来元数据层的设计方向

### 8.1 元数据层应从“字段配置”提升为“平台标准模型”

当前 `MetadataField` 是一个良好的起点，但未来应扩展成：

- `MetaDatasource`
- `MetaTable`
- `MetaColumn`
- `MetaRelation`
- `MetaConstraint`
- `MetaIndex`
- `MetaPolicy`
- `MetaCrudProfile`

### 8.2 字段类型应标准化抽象

建议引入统一字段类型枚举，例如：

- `String`
- `Text`
- `Int`
- `Long`
- `Decimal`
- `Bool`
- `Date`
- `DateTime`
- `Json`
- `Binary`
- `Enum`
- `Reference`

运行时再映射到不同数据库真实类型。

### 8.3 关系建模应配置化

未来应把当前 `LinkReference` 升级为更标准的关系建模：

- 一对一
- 一对多
- 多对一
- 多对多
- 关联中间表
- JOIN 类型
- 级联策略

### 8.4 权限与策略应进入元数据层

建议未来将以下规则纳入元数据：

- 字段可见性
- 字段可编辑性
- 行级权限
- 数据范围过滤
- 默认过滤器
- 逻辑删除策略
- 审计字段自动填充策略

---

## 9. 未来 engine 的演进方向

### 9.1 继续强化谓词系统

未来建议继续增加：

- `NOT IN`
- `NOT EXISTS`
- `IS NOT NULL`
- `OR group`
- 嵌套条件树
- 聚合函数表达式对象化
- HAVING 构建能力

### 9.2 从 Builder 进化为 Query Plan 渲染器

目前 `engine` 更像“手工装配 Builder”；未来应能直接接收统一查询计划对象并渲染为 SQL。

### 9.3 支持预览与执行分离

建议未来所有 SQL 生成都统一输出：

- SQL 文本
- 参数列表
- 语义化计划摘要
- 可审计结构

---

## 10. 未来 metadata_driver 的演进方向

### 10.1 从“构建器入口”升级为“查询规划器”

当前 `MetadataSqlDriver` 已具备生成 SQL 的能力，但未来应升级为：

- 接收平台级元数据
- 生成统一查询计划
- 调用 `engine` 渲染
- 与执行层衔接

### 10.2 与旧 `portal_provider` 的关系

建议未来演进策略：

- 短期：保持并行
- 中期：增加适配层，把旧字段结构映射到新元数据模型
- 长期：以 `metadata + metadata_driver + engine` 替代 `portal_provider`

### 10.3 增强过滤模型

当前新驱动已支持一批基础过滤能力，未来应继续引入显式的过滤对象模型，例如：

- `Eq`
- `Ne`
- `Gt`
- `Gte`
- `Lt`
- `Lte`
- `Like`
- `In`
- `Between`
- `Exists`
- `IsNull`
- `IsNotNull`
- `And`
- `Or`
- `Not`

从“运行时判断 Value 结构”演进为“显式过滤 AST”。

---

## 11. 未来数据层平台的开发阶段建议

### 阶段一：底座稳定化

目标：把当前仓库中的 SQL / Metadata / DDL 基础能力收束为一个稳定、可验证、可继续迁移的平台底座。

当前判断：阶段一已完成大半，现已从“搭骨架”进入“收口与统一抽象”阶段。

本阶段已完成：

- `engine` 已完成模块化拆分，并支持 MySQL / PostgreSQL / Oracle / SQL Server / SQLite
- `engine` 已支持 `SELECT / INSERT / UPDATE / DELETE` 与元数据驱动 DDL Builder
- `engine` 已补齐 `!=`、`>`、`>=`、`<`、`<=`、`LIKE`、`IN`、`BETWEEN`、`EXISTS` 等核心过滤能力
- `metadata` 已建立强类型字段模型，`metadata_driver` 已拆分为 `context / driver / filters / helpers`
- 已补齐多数据库 CRUD examples、SQLite example、元数据治理 example
- 已完成阶段性验证，当前测试与 examples 编译均通过

本阶段剩余工作：

- 将过滤模型从基于 `Value` 的运行时推断，升级为显式过滤 AST
- 为 `metadata_driver` 增加更完整的聚合、HAVING、复杂条件树与关系组合能力
- 建立 `portal_provider -> metadata` 的适配桥，继续降低旧入口的重要性
- 将 DDL Builder 与未来标准元数据表结构进一步对齐，避免形成第二套独立 schema 描述方式

阶段一验收标准建议调整为：

- `engine` 在 DML + DDL 两类场景下都具备稳定的多方言构建能力
- `metadata_driver` 能覆盖当前主流查询、写入、权限过滤、导入导出示例场景
- `portal_provider` 不再承担新增能力演进，只保留兼容职责
- 测试、examples、文档三者对外表达一致，能够作为阶段二建模工作的稳定基座

### 阶段二：统一元数据平台模型

目标：把当前已经出现的“查询元数据模型”“治理元数据模型”“DDL schema 模型”进一步统一为平台级标准元数据体系。

当前判断：阶段二已具备启动条件，并且已经有一部分前置成果，不再是从零开始。

本阶段已具备的基础：

- `metadata` 已具备查询驱动所需的字段模型、权限 mask、lookup、link、显式过滤 AST
- `metadata` 中已经新增标准 schema 描述能力，包括 `MetadataColumnType`、`MetadataColumnSchema`、`MetadataForeignKeySchema`、`MetadataTableSchema`
- 已提供 `standard_metadata_tables()`，说明标准元数据表结构已经有了第一版代码化表达
- `engine` DDL Builder 已可直接承接这些标准 schema 对象生成建表语句

本阶段的核心工作应调整为：

- 将当前分散的查询模型、治理模型、DDL schema 模型进一步统一，避免未来出现多套平行元数据描述
- 补齐平台级实体：数据源、表、字段、关系、约束、索引、权限、策略、导入模板、导出模板
- 建立“配置模型”与“运行时模型”的边界，明确哪些结构用于存储，哪些结构用于执行期规划
- 抽象多数据源配置模型，并让标准元数据表结构与多数据源生命周期管理相衔接
- 定义平台内部统一 `QueryPlan / WritePlan / DeletePlan / SchemaPlan`

建议本阶段进一步拆成三条主线：

#### 主线一：标准元数据实体统一

- 统一 `MetaDatasource`
- 统一 `MetaTable`
- 统一 `MetaColumn`
- 统一 `MetaRelation`
- 统一 `MetaPolicy`
- 统一 `MetaImportProfile`
- 统一 `MetaExportProfile`

#### 主线二：运行时规划模型统一

- 定义 `QueryPlan`
- 定义 `WritePlan`
- 定义 `DeletePlan`
- 定义 `SchemaPlan`
- 定义 `PermissionPlan`

#### 主线三：标准元数据持久化对齐

- 让 `standard_metadata_tables()` 覆盖更完整的平台表集合
- 增加元数据模型与标准元数据表之间的映射层
- 让 DDL 生成、元数据管理和未来执行层使用同一套 schema 描述来源

阶段二验收标准建议补充为：

- 仓库中只保留一套权威的标准元数据平台模型，不再出现查询模型、治理模型、DDL 模型长期分裂的情况
- 标准元数据表结构能够完整表达平台核心实体，并可直接生成 DDL
- `QueryPlan / WritePlan / SchemaPlan` 的职责边界明确，可以稳定承接阶段三执行层建设
- 文档、示例、代码中的元数据术语与结构保持统一，对外表达一致

### 阶段三：执行层建设

目标：从“生成 SQL”走向“生成并执行 CRUD”。

建议工作：

- 引入 `sqlx`
- 建立 DatasourceManager
- 增加事务管理
- 建立 CRUD 执行服务

### 阶段四：平台接口化

目标：对上层提供统一服务接口。

建议工作：

- 元数据管理 API
- 元数据搜索 API
- SQL 预览 API
- 通用 CRUD API
- 审计与日志接口

### 阶段五：低代码 / 平台化扩展

目标：变成真正的通用平台。

建议工作：

- 动态列表页 / 表单页支持
- 导入导出
- 权限体系
- 可视化配置
- 插件化扩展能力

---

## 12. 当前建议的近期优先级

### 高优先级

- 将 `portal_provider` 逐步适配到新元数据层
- 抽象显式过滤对象模型
- 设计平台级元数据表结构
- 为新元数据驱动补充更多复杂关系测试

### 中优先级

- 设计多数据源管理层
- 定义 CRUD 服务 Trait
- 定义 SQL 预览与执行分离接口
- 引入缓存和元数据加载优化

### 低优先级

- 上层 API
- 管理后台
- 低代码页面支撑
- 插件化机制

---

## 13. 结论

当前项目已经完成了关键的底座转型：

- 已从旧的单体式字符串拼 SQL 方案，演进出可扩展的 `engine`
- 已建立新的强类型元数据模型 `metadata`
- 已实现新的元数据 SQL 驱动核心 `metadata_driver`
- 已具备未来承载“数据层 + 跨库 CRUD 自动生成平台”的基础条件

下一阶段的关键，不再是单点补 SQL 细节，而是把当前这些可用能力统一收束成：

- 平台级元数据模型
- 查询 / 写入计划模型
- 多数据源执行层
- 通用 CRUD 服务层

如果后续继续沿这条路线演进，当前仓库完全可以逐步发展成一个企业级、跨数据库、元数据驱动的自动 CRUD 平台基础内核。

---

## 14. 企业级平台目标蓝图

为了真正把当前仓库演进成“企业级、可落地、跨关系型数据库兼容、元数据驱动的通用 CRUD 自动生成平台”，后续路线建议明确收束到一个完整的平台蓝图，而不是继续停留在局部 SQL 构建层面。

目标平台建议具备以下完整交付能力：

- 标准元数据层
- 元数据搜索与管理能力
- 跨库 SQL 自动生成引擎
- 通用 CRUD 执行服务
- 多数据源动态切换
- SQL 预览与执行分离
- 审计日志与权限控制
- 对外 API 与上层低代码扩展能力

这意味着未来平台应当从“库”逐步演进为“平台内核 + 服务层 + 接口层”的形态。

---

## 15. 建议的企业级模块拆分

未来建议将整体项目拆分为更清晰的模块或 crate 边界：

### 15.1 `meta-model`

负责平台级标准模型：

- 数据源模型
- 表模型
- 字段模型
- 关系模型
- 索引与约束模型
- 权限与策略模型
- API 入参与出参模型

### 15.2 `meta-repository`

负责元数据读写：

- 元数据存储
- 元数据缓存
- 搜索索引接入
- 元数据加载与版本控制

### 15.3 `meta-planner`

负责把平台请求转为统一计划对象：

- `QueryPlan`
- `WritePlan`
- `JoinPlan`
- `ProjectionPlan`
- `FilterPlan`
- `ValidationPlan`

### 15.4 `meta-engine`

负责方言无关 SQL 渲染：

- 方言适配
- QueryPlan 渲染
- 参数绑定输出
- SQL 预览输出

### 15.5 `meta-executor`

负责执行：

- 数据源路由
- 连接池
- 事务
- 执行结果映射
- 错误封装

### 15.6 `meta-service`

负责平台级能力编排：

- 元数据 CRUD
- 通用业务 CRUD
- 关系查询
- 批量导入导出
- SQL 预览

### 15.7 `meta-api`

负责对外提供：

- REST API
- OpenAPI 文档
- 身份认证与权限鉴权
- 平台管理接口

---

## 16. 标准元数据层数据库表设计方向

当前 `metadata.rs` 已经提供了运行时查询建模的雏形，但未来平台落地时需要一套可持久化、可管理、可搜索的标准元数据表结构。

建议最少包含以下核心表：

### 16.1 数据源配置

- `meta_datasource`

字段建议：

- 数据源编码
- 数据源名称
- 数据库类型
- 连接串
- 用户名
- 密码密文
- 是否启用
- 默认 schema
- 扩展配置

### 16.2 表定义

- `meta_table`

字段建议：

- 表编码
- 物理表名
- 显示名称
- 业务域
- 数据源标识
- 主键策略
- 是否逻辑删除
- 是否审计表
- 默认排序规则

### 16.3 字段定义

- `meta_column`

字段建议：

- 所属表
- 字段编码
- 物理列名
- 显示名称
- 标准字段类型
- 是否主键
- 是否可空
- 是否可查询
- 是否可编辑
- 是否可排序
- 默认值策略
- 引用关系配置
- 序列策略
- UI 扩展属性

### 16.4 关系定义

- `meta_relation`

字段建议：

- 左表
- 右表
- 关系类型
- JOIN 类型
- 关联键
- 中间表信息
- 级联策略

### 16.5 约束与索引

- `meta_constraint`
- `meta_index`

### 16.6 权限与策略

- `meta_policy`
- `meta_permission`

### 16.7 审计与运维

- `meta_operation_log`
- `meta_publish_log`
- `meta_runtime_snapshot`

---

## 17. 运行时核心对象设计方向

未来如果要做到企业级平台，建议把当前“直接从字段生成 SQL”的模式升级为“先建计划对象，再渲染，再执行”的模式。

建议引入以下运行时对象：

### 17.1 查询请求对象

- `CrudQueryRequest`
- `CrudInsertRequest`
- `CrudUpdateRequest`
- `CrudDeleteRequest`

### 17.2 计划对象

- `QueryPlan`
- `WritePlan`
- `DeletePlan`
- `FilterExpr`
- `JoinNode`
- `SortExpr`
- `GroupExpr`

### 17.3 执行上下文对象

- `ExecutionContext`
- `DatasourceContext`
- `TenantContext`
- `PermissionContext`

### 17.4 输出对象

- `SqlPreview`
- `ExecutionResult<T>`
- `PagedResult<T>`
- `ApiResponse<T>`

引入这些对象之后，平台将更容易实现：

- SQL 预览
- 审核后执行
- 计划缓存
- 日志追踪
- 安全校验

---

## 18. 未来功能交付清单建议

从企业级交付角度，建议后续版本按如下能力清单推进：

### 18.1 元数据管理能力

- 元数据新增 / 修改 / 删除
- 元数据版本管理
- 元数据发布
- 元数据差异比较

### 18.2 元数据搜索能力

- 按表名搜索
- 按字段名搜索
- 按业务域搜索
- 按数据源搜索
- 按关系搜索
- 按标签搜索

### 18.3 通用 CRUD 能力

- 单表查询
- 关联查询
- 新增
- 修改
- 删除
- 批量新增
- 批量修改
- 批量删除
- 逻辑删除

### 18.4 平台辅助能力

- SQL 预览
- 参数预览
- 操作日志
- 审计字段自动填充
- 唯一性校验
- 非空校验
- 引用完整性校验

### 18.5 上层扩展能力

- 后台管理页面接口
- 低代码列表页接口
- 动态表单接口
- 导入导出能力
- 对外开放 API

---

## 19. 建议的阶段化开发路线（升级版）

### 阶段 A：平台底座稳定化

- 完成 `portal_provider` 到新元数据层的迁移桥接
- 补齐 `engine` 的谓词树、HAVING、NOT IN、NOT EXISTS、IS NOT NULL 等能力
- 完善 `metadata_driver` 的过滤 AST 和关系能力
- 持续补强测试基线

### 阶段 B：标准元数据持久化

- 设计并实现标准元数据表
- 完成元数据模型与库表映射
- 增加元数据读取、缓存、搜索与校验能力

### 阶段 C：执行层建设

- 引入 `sqlx`
- 建立多数据源管理器
- 建立执行器、事务管理器、统一错误系统
- 打通 SQL 生成与执行闭环

### 阶段 D：平台服务层建设

- 通用 CRUD Service
- 元数据管理 Service
- SQL 预览 Service
- 审计与权限 Service

### 阶段 E：接口与产品化

- REST API
- OpenAPI
- 管理后台接口
- 低代码支撑接口

---

## 20. 当前最值得优先推进的事项

结合当前仓库实际状态，建议近期优先级调整为：

### 第一优先级

- 将旧 `portal_provider` 能力逐步迁移到 `metadata + metadata_driver + engine`
- 将过滤模型从“Value 推断”升级为显式 AST
- 设计标准元数据库表结构和 Rust 模型

### 第二优先级

- 建立 QueryPlan / WritePlan
- 建立多数据源执行层
- 设计 SQL 预览与执行分离接口

### 第三优先级

- 建立元数据搜索模块
- 构建平台级 CRUD Service
- 建立 OpenAPI 和平台接口

### 第四优先级

- 管理后台
- 低代码页面能力
- 插件化和扩展机制

---

## 21. 更新后的结论

当前项目已经不再只是一个 SQL Builder，而是一个正在成型的“元数据驱动数据层内核”。

在已经完成 `engine`、`metadata`、`metadata_driver` 以及多方言 examples 的基础上，下一阶段应该明确把目标锁定为：

- 标准元数据平台模型
- 统一的 QueryPlan / WritePlan
- 跨数据库执行层
- 企业级通用 CRUD 服务层
- 平台化接口与治理能力

只要后续持续按照这份路线图推进，当前仓库可以逐步演进为一套真正企业级、可落地、跨关系型数据库兼容、元数据驱动的通用 CRUD 自动生成平台内核。
