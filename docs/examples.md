---
title: Examples
nav_order: 4
has_children: true
permalink: /examples/
---

# Examples

Complete examples that show how a real Shaperail application is structured.

## Available examples

| Example | Description |
| --- | --- |
| [**Blog API**]({{ '/blog-api-example/' | relative_url }}) | Two resources (posts, comments) with controllers: slug generation, edit rules, comment rate limiting, XSS prevention. Public reads, protected writes, owner-based updates, relations, cursor/offset pagination, soft delete. |
| **Multi-service workspace** | Two services (users-api, orders-api) with controllers: email normalization, password hashing, domain blocking, order state machine, cross-service validation, refund detection. Distributed saga for order creation. |
| **Multi-tenant SaaS** | Three resources (organizations, projects, tasks) with controllers: plan-based project limits, status transition enforcement, cross-resource validation, tenant-scoped uniqueness. Shows `tenant_key`, JWT tenant claims, `super_admin` bypass. |
| **Enterprise SaaS** | Full B2B billing example with customers, invoices, and payments. Controllers: invoice approval workflow (draft→sent→paid), payment validation with idempotency, audit trail with before/after snapshots, plan-based credit limits. |
| **WASM plugins** | Controller hooks written in TypeScript and Python compiled to WASM. Includes email validation and input normalization examples with the full plugin interface documented. |

Every example includes controller files (`*.controller.rs`) with real business
logic patterns. Source files live in the repository under `examples/`.
