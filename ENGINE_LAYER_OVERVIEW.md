# Engine Layer Overview

## Purpose

The `engine` layer is the low-level SQL construction core of the project.
Its responsibility is to provide stable, dialect-aware builders that can assemble SQL for query, write, delete, and DDL operations without depending on higher-level metadata concepts.

It should remain:

- **Pure**: no database connections, no `sqlx`, no runtime execution.
- **Reusable**: callable by metadata-driven modules and also by direct examples/tests.
- **Deterministic**: given the same builder inputs and dialect, it must generate the same SQL and parameter order.

## Current Structure

The layer now lives under `src/engine/`.

- **`mod.rs`**
  Re-export entry for the engine layer.

- **`builders.rs`**
  Query/write builders such as `SelectBuilder`, `InsertBuilder`, `UpdateBuilder`, `DeleteBuilder`.

- **`ddl.rs`**
  DDL-focused builders and models such as `CreateTableBuilder`, `AlterTableBuilder`, `DropTableBuilder`, `ColumnDefinition`, `ForeignKeyDefinition`.

- **`dialect.rs`**
  SQL dialect abstraction and concrete dialect implementations such as `PostgresDialect`, `MySqlDialect`, `OracleDialect`, `SqlServerDialect`, `SqliteDialect`.

- **`facade.rs`**
  `MetaSqlEngine`, the main façade for rendering builders into `BuiltQuery`.

- **`query.rs`**
  Shared query-side models such as `BuiltQuery`, `Predicate`, `Relation`, `JoinType`, `Pagination`, `TableRef`.

- **`tests.rs`**
  Engine-only stability tests and performance-oriented smoke coverage.

## Core Invariants

- **Builder output must be dialect stable**
  Placeholder style, pagination shape, and DDL syntax must be controlled by the dialect implementation.

- **Parameter order must be predictable**
  Every builder must append parameters in the exact logical order reflected by generated SQL.

- **Raw SQL support is explicit**
  Raw expressions such as `sysdate` or sequence calls are allowed only through dedicated raw APIs.

- **Empty collections must be safe**
  Cases like empty `IN` lists must degrade into safe SQL instead of producing invalid SQL.

## Typical Data Flow

A common flow is:

1. Higher layer creates builder objects.
2. `MetaSqlEngine` renders them using a chosen dialect.
3. A `BuiltQuery` is returned.
4. Higher layers decide whether to preview or execute it.

Example usage patterns:

- metadata layer creates `SelectBuilder` or `InsertBuilder` indirectly through metadata translation.
- execution layer receives `BuiltQuery` and binds parameters for actual database execution.

## What Belongs Here

- SQL syntax assembly.
- Placeholder generation.
- DDL rendering.
- Join/predicate/pagination composition.
- Safe SQL fallbacks for edge cases.

## What Does Not Belong Here

- Metadata permissions.
- Datasource registration.
- Runtime execution logic.
- Business table semantics.
- Import/export policy decisions.

## Extension Guidance

When adding new engine capability:

1. Add or extend builder-side data structures.
2. Keep dialect-specific rendering in `dialect.rs` or dialect render paths.
3. Add coverage in `engine/tests.rs` for all affected dialect shapes.
4. Prefer extending existing builder vocabulary before introducing ad hoc string assembly upstream.

## Safe Evolution Rules

- Do not push metadata-specific branching into the engine.
- Do not couple builders to `serde_json::Value` unless the abstraction really needs generic data.
- Add new builder APIs only when the behavior can be explained at SQL-construction level.
- If a change affects placeholder ordering or pagination output, add cross-dialect assertions immediately.
