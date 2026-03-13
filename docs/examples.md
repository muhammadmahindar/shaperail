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
| [**Blog API**]({{ '/blog-api-example/' | relative_url }}) | Two resources (posts, comments), public reads, protected writes, owner-based updates via `created_by`, relations, cursor and offset pagination, soft delete. Includes `resources/*.yaml`, `migrations/*.sql`, and `shaperail.config.yaml`. |

Source files for the Blog API example live in the repository under `examples/blog-api/`. Use them as a reference when building your own resources and migrations.
