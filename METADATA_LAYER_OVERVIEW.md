# Metadata Layer Overview

## Purpose

The `metadata` layer describes the business system in a platform-neutral way.
It is the semantic center of the project: tables, columns, relations, policies, import/export profiles, and runtime request models are all defined here or derived from here.

This layer is responsible for turning business intent into structured metadata, not into executed SQL.

## Current Structure

The metadata-related code is mainly distributed across the following modules:

- **`src/metadata/`**
  Core metadata model definitions.

- **`src/metadata_driver/`**
  Translates `MetadataQueryRequest` into engine-level SQL builders and final `BuiltQuery` output.

- **`src/metadata_plan/`**
  Builds runtime planning models such as permission plan, query plan, write plan, delete plan, and schema plan.

- **`src/metadata_mapping/`**
  Converts platform metadata catalog objects into persistence snapshots for standard metadata tables.

- **`src/metadata_demo.rs`**
  Demo metadata fixtures and scenario constructors used by examples and stability tests.

## Metadata Subdomains

### Catalog Domain

The catalog domain defines platform-level metadata entities such as:

- `MetaDatasource`
- `MetaTable`
- `MetaColumn`
- `MetaRelation`
- `MetaPolicy`
- `MetaImportProfile`
- `MetaExportProfile`
- `MetadataCatalog`

These types answer the question:

**What is the system structure and governance model?**

### Request Domain

Request-side metadata models such as `MetadataQueryRequest`, `MetadataField`, `MetadataFilterExpr`, and `MetadataQueryOptions` answer:

**What is the caller trying to do right now?**

### Schema Domain

`MetadataTableSchema`, `MetadataColumnSchema`, `MetadataForeignKeySchema`, and `standard_metadata_tables()` answer:

**How should metadata itself be stored and provisioned?**

### Runtime Planning Domain

`metadata_plan` translates requests into execution-oriented planning models while still staying metadata-centric.

It answers:

**Given a request, what fields, permissions, relations, assignments, and filters should downstream layers honor?**

### Persistence Mapping Domain

`metadata_mapping` translates catalog objects into standard metadata persistence rows.

It answers:

**How do catalog objects map into the standardized metadata table set?**

## Core Invariants

- **Metadata is the source of truth for system behavior**
  Queryable fields, writable fields, import/export capabilities, and governance rules should be derivable from metadata.

- **Runtime requests stay structured**
  Do not collapse metadata requests into raw SQL too early.

- **Catalog and request models are separate concerns**
  Catalog describes the system; request describes an operation on the system.

- **Persistence mapping should be deterministic**
  The same catalog should always produce the same persistence snapshot.

## Typical Data Flow

1. A business scenario is modeled as catalog entities and request metadata.
2. `metadata_plan` extracts runtime planning information.
3. `metadata_driver` turns request metadata into engine SQL output.
4. `metadata_mapping` persists catalog metadata into standard metadata tables when needed.
5. `execution` optionally executes the planned output.

## What Belongs Here

- Platform metadata entities.
- Request and filter AST models.
- Standard metadata storage schema.
- Catalog-to-persistence mapping.
- Metadata-driven SQL translation.
- Demo metadata fixtures and stability coverage.

## What Does Not Belong Here

- SQL placeholder binding.
- Pool management.
- Transaction handling.
- Driver installation.
- Environment-specific infrastructure concerns.

## Extension Guidance

When introducing a new business capability:

1. Define whether it is a catalog concern, request concern, planning concern, or persistence concern.
2. Extend the metadata model first.
3. Add or adapt runtime planning structures if the capability affects execution semantics.
4. Extend `metadata_driver` only after the metadata contract is stable.
5. Extend stability fixtures in `metadata_demo.rs` and tests in `tests/metadata_stability.rs`.

## High-Risk Areas

- Mixing request semantics with persistence semantics.
- Encoding business rules directly into execution code instead of metadata.
- Letting one-off demo assumptions leak into generic metadata abstractions.
- Making persistence mapping depend on unstable string formatting rules.
