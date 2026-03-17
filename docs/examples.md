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
| **Multi-service workspace** | Two services (users-api, orders-api) showing workspace layout, dependency-ordered startup, and validated saga definitions for order creation. |
| **Multi-tenant SaaS** | Three resources (organizations, projects, tasks) with controllers: plan-based project limits, status transition enforcement, cross-resource validation, tenant-scoped uniqueness. Shows `tenant_key`, JWT tenant claims, `super_admin` bypass. |
| **Controller walkthrough** | See the [Controllers]({{ '/controllers/' | relative_url }}) guide for a complete multi-resource billing walkthrough with manual controller registration, workflow/state-machine controllers, audit logging, payment reconciliation, and testing guidance. |
| **WASM plugins** | Controller hooks written in TypeScript and Python compiled to WASM. Includes email validation and input normalization examples with the full plugin interface documented. |

Examples show real controller logic patterns in the documentation, but the
repository examples focus on resource/config layout and HTTP flows. If you wire
controllers into a running app today, manual controller registration is still
required.
