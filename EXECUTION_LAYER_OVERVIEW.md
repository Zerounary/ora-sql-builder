# Execution Layer Overview

## Purpose

The `execution` layer is the runtime bridge between metadata/engine planning output and real database operations.
It owns datasource registration, SQL preview, SQL execution, row materialization, and schema/catalog persistence execution.

It should remain a runtime adapter layer, not a business modeling layer.

## Current Structure

The execution layer now lives under `src/execution/`.

- **`mod.rs`**
  Re-export entry for execution APIs.

- **`context.rs`**
  Runtime context and options such as `ExecutionContext`, `ExecutionMode`, `ExecutionOptions`.

- **`datasource.rs`**
  Datasource registry and pool management via `DatasourceManager`.

- **`error.rs`**
  Shared execution-layer error types.

- **`results.rs`**
  Query/write/delete/schema/catalog execution result models.

- **`helpers.rs`**
  Runtime helper functions such as dialect selection, row binding, row decoding, schema-plan helpers.

- **`executors.rs`**
  Actual executors for query, write, delete, schema, and metadata catalog persistence.

- **`tests.rs`**
  SQLite-backed execution verification.

## Main Runtime Concepts

### DatasourceManager

Manages connection pools keyed by metadata datasource id.

Responsibilities:

- register datasource pools
- fetch pools by datasource id
- run basic health checks

### ExecutionContext

Bundles three things together:

- datasource manager
- target datasource metadata
- execution options

This object defines the runtime boundary for an execution call.

### Executors

The layer currently exposes:

- `QueryPlanExecutor`
- `WritePlanExecutor`
- `DeletePlanExecutor`
- `SchemaPlanExecutor`
- `MetadataCatalogExecutor`

Each executor supports preview behavior and real execution behavior.

## Core Invariants

- **Preview and execution must stay behaviorally aligned**
  Dry-run output should match the SQL that would be executed.

- **Datasource kind determines dialect binding**
  Execution should not guess dialect behavior outside the datasource metadata.

- **Execution layer should not reinvent planning**
  It consumes plans and metadata requests from upstream layers instead of re-deriving business semantics.

- **SQLite normalization remains explicit**
  Any runtime SQL normalization, such as `sysdate` replacement, should stay centralized and auditable.

## Typical Data Flow

1. Upstream builds a `QueryPlan`, `WritePlan`, `DeletePlan`, `SchemaPlan`, or `MetadataCatalog`.
2. Executor previews dialect-specific SQL.
3. If not in dry-run mode, execution binds parameters and runs through `sqlx`.
4. Results are converted into layer-specific result models.

## What Belongs Here

- Pool registration.
- Runtime SQL normalization.
- Parameter binding.
- Result decoding.
- Schema execution.
- Metadata catalog persistence execution.

## What Does Not Belong Here

- Modeling metadata entities.
- Deciding field visibility rules.
- Defining import/export semantics.
- Constructing high-level business requests.

## Extension Guidance

When extending execution behavior:

1. Decide whether the change belongs in executor orchestration, helper logic, or datasource management.
2. Keep all datasource-specific render decisions behind dialect resolution.
3. Prefer adding new result models over overloading existing ones with unrelated fields.
4. Add SQLite execution coverage when the change affects runtime binding or result decoding.
5. Add preview assertions when the change affects generated SQL.

## High-Risk Areas

- Divergence between preview SQL and actual executed SQL.
- Inconsistent row decoding across database types.
- Hidden datasource-kind assumptions outside `dialect_for`.
- Spreading transaction handling logic across too many places.
