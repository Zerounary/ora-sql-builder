# AI-Friendly System Build Guide

## Goal

This document is a working contract for AI-assisted development on top of the current `ora-sql-builder` architecture.

You can describe a target business system in simple language, and an AI should be able to use this document to:

- define the required metadata model
- create demo fixtures
- generate query/write/import/export capabilities
- extend runtime planning and execution paths where necessary
- add verification coverage

## How To Describe The System To AI

When asking AI to build a new system, provide the request in this structure.

### 1. Business Scope

State what kind of system you want.

Example:

- sales order system
- inventory movement system
- project timesheet system
- service ticket system

### 2. Core Business Objects

List the main business entities and their relationships.

Example:

- `store`
- `customer`
- `sale_order`
- `sale_order_line`

And describe relation shape.

Example:

- one store has many sale orders
- one customer has many sale orders
- one sale order has many sale order lines

### 3. Functional Capabilities

Specify the behavior you need.

Example:

- query with filters and sorting
- grouped statistics
- create new records
- update selected fields
- delete by id with tenant filter
- import from external rows
- export with default filters
- row-level permission control

### 4. Governance Rules

State policies clearly.

Example:

- tenant isolation by `tenant_id`
- only enabled rows are exportable
- import must reject rows missing `code`
- some fields are read-only

### 5. Runtime/Execution Expectations

Specify execution scope.

Example:

- preview SQL for all dialects
- real execution on SQLite stability test
- persistence into standard metadata tables

## What AI Should Produce

For a well-formed request, AI should usually produce the following artifacts.

### Metadata Model Artifacts

- catalog entities in terms of `MetaDatasource`, `MetaTable`, `MetaColumn`, `MetaRelation`, `MetaPolicy`, `MetaImportProfile`, `MetaExportProfile`
- request constructors in a fixture/demo module
- any necessary standard metadata persistence output through `metadata_mapping`

### Planning Artifacts

- `QueryPlan` / `WritePlan` / `DeletePlan` compatibility
- any required permission or relation handling extensions

### SQL Translation Artifacts

- metadata-driver support for new field or filter semantics
- engine-builder usage only through existing abstractions where possible

### Execution Artifacts

- preview verification
- SQLite real execution verification if runtime semantics changed

### Documentation Artifacts

- short explanation of new entities
- test coverage summary
- any constraints or assumptions

## AI Development Workflow

AI should follow this order.

1. **Model the business domain in metadata**
2. **Define runtime requests and demo fixtures**
3. **Check whether planning layer already supports the requested behavior**
4. **Extend metadata-driver only if translation support is missing**
5. **Use engine builders instead of raw SQL whenever possible**
6. **Add execution coverage if runtime behavior changed**
7. **Add stability tests**
8. **Document the result**

## Rules AI Should Respect

- Keep functionality unchanged unless the request explicitly introduces new behavior.
- Extend metadata contracts before extending runtime execution logic.
- Avoid putting business-specific hacks into `engine`.
- Avoid bypassing metadata with direct string SQL unless the architecture already requires it.
- Keep demo fixtures representative and reusable.
- Every meaningful capability change should be backed by tests.

## Recommended Request Template For You

Use this template when asking AI to build a new system:

```markdown
Build a [system name] system.

Business objects:
- [object A]
- [object B]
- [object C]

Relations:
- [relation 1]
- [relation 2]

Required functions:
- query
- filter
- relation lookup
- create
- update
- delete
- import
- export
- statistics

Governance rules:
- [rule 1]
- [rule 2]
- [rule 3]

Execution expectations:
- preview for all supported dialects
- SQLite execution stability test
- metadata persistence compatibility

Deliverables:
- metadata model
- demo fixtures
- tests
- documentation
```

## Example Minimal Prompt

```markdown
Build an inventory movement system.

Business objects:
- warehouse
- product
- stock_move
- stock_move_line

Relations:
- one warehouse has many stock moves
- one stock move has many stock move lines
- one product appears in many stock move lines

Required functions:
- query stock moves by warehouse, date, status
- create and update moves
- delete draft moves
- import move lines from spreadsheet rows
- export approved move summaries
- enforce tenant filter and enabled filter

Execution expectations:
- preview SQL for all supported dialects
- SQLite real execution test
- metadata persistence snapshot support
```

## Expected AI Output Pattern

If AI follows this guide, its result should normally include:

- new metadata tables/entities only when required
- demo request constructors similar to `metadata_demo.rs`
- planning compatibility with `metadata_plan`
- SQL translation compatibility with `metadata_driver`
- stability tests similar to `metadata_stability.rs`
- concise markdown documentation

## When AI Should Ask You Questions

AI should ask follow-up questions only if one of these is unclear:

- the main business objects are missing
- relation shape is ambiguous
- import/export rules are unspecified but required
- permission model is critical but undefined
- runtime verification scope is unclear

If those items are already provided, AI should prefer implementation over more questioning.
