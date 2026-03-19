# Metadata Layer Testing Method

## 1. Purpose

This document describes how the metadata layer stability tests are organized for the current `ora-sql-builder` project.

The goal is to keep the external behavior of the metadata core stable while the internal architecture continues to evolve.

## 2. Datasource Information File

The supported datasource information file is:

- `examples/supported_datasources.json`

It contains one demo datasource entry for each currently supported database type:

- `mysql`
- `postgres`
- `oracle`
- `sqlserver`
- `sqlite`

The file is intended for preview, testing, and example orchestration.

Only the SQLite datasource is used for real execution in automated tests.

## 3. Demo Business Table Set

The demo metadata table set is defined in:

- `src/metadata_demo.rs`

It models a small sales domain with three tables:

- `demo_store`
- `demo_customer`
- `demo_sale_order`

This table set is designed to demonstrate the following business capabilities:

- Query
- Filter
- Relation
- Create
- Import
- Update
- Delete
- Export

## 4. Scenario Coverage

The reusable metadata requests in `src/metadata_demo.rs` cover these scenarios:

- `order_query_request()`
- `order_export_request()`
- `direct_order_insert_request(...)`
- `imported_order_insert_request(...)`
- `order_update_request(...)`
- `order_delete_request(...)`

These requests are used to keep the scenario definition centralized and reusable.

## 5. Stability Tests

The stability tests are located in:

- `tests/metadata_stability.rs`

They are split into two layers.

### 5.1 Cross-database SQL preview stability

This layer verifies that the same business scenarios can be rendered for every supported datasource listed in `examples/supported_datasources.json`.

It focuses on:

- SQL generation availability
- Placeholder style stability per dialect
- Scenario coverage stability across query / create / import / update / delete / export

### 5.2 SQLite real execution stability

This layer uses the SQLite in-memory datasource and the stage-three execution layer to run the scenarios end to end.

It focuses on:

- Schema creation
- Seed data preparation
- Create
- Import
- Update
- Query
- Export
- Delete

## 6. How to Run

Run all tests:

```powershell
cargo test
```

Run only metadata stability tests:

```powershell
cargo test metadata_stability -- --nocapture
```

Check example compilation:

```powershell
cargo check --examples
```

## 7. Stability Principle

The stability tests are intended to protect the outward behavior of the metadata core while the following internal areas continue to evolve:

- metadata entities
- metadata plans
- metadata persistence mapping
- execution layer
- engine builders

As long as these tests remain green, the project can continue refactoring the core with significantly lower risk.
