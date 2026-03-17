---
title: Resource archetypes
parent: Reference
nav_order: 5
---

# Resource archetypes

When you scaffold a new resource with `shaperail resource create`, you choose an
archetype. Archetypes are starter templates that give you a working resource YAML
with sensible defaults for common patterns. There are five archetypes:

```bash
shaperail resource create <name> --archetype <archetype>
```

Available archetypes: `basic`, `user`, `content`, `tenant`, `lookup`.

## basic

The minimal starting point. Gives you an `id` and timestamps with CRUD
endpoints. Add your own fields.

**When to use:** Any resource that does not fit the other archetypes. Start here
and customize.

**Schema fields:**

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key, auto-generated |
| `created_at` | timestamp | Auto-generated |
| `updated_at` | timestamp | Auto-generated |

**Endpoints:**

| Action | Method | Auth | Notes |
| --- | --- | --- | --- |
| list | GET | public | Cursor pagination |
| get | GET | public | |
| create | POST | admin | Empty `input` array -- add your fields |
| update | PATCH | admin | Empty `input` array -- add your fields |
| delete | DELETE | admin | Hard delete |

**Example:**

```bash
shaperail resource create products --archetype basic
```

Then edit `resources/products.yaml` to add fields like `title`, `price`, and
`status`, and add those field names to the `input` arrays.

## user

A user/account resource with email, name, role, and organization membership.
Includes search, filtering, soft delete, and an index on email.

**When to use:** User accounts, team members, admin users, or any
person-representing resource that has email-based identity and role-based access.

**Schema fields:**

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key, auto-generated |
| `email` | string | Unique, required, email format, marked sensitive |
| `name` | string | Required, 1-200 chars |
| `role` | enum | Values: `admin`, `member`, `viewer`. Default: `member` |
| `org_id` | uuid | Required, references `organizations.id` |
| `created_at` | timestamp | Auto-generated |
| `updated_at` | timestamp | Auto-generated |

**Endpoints:**

| Action | Method | Auth | Notes |
| --- | --- | --- | --- |
| list | GET | member, admin | Filters: `role`, `org_id`. Search: `name`, `email`. Sort: `created_at`, `name`. Cursor pagination |
| get | GET | member, admin | |
| create | POST | admin | Input: `email`, `name`, `role`, `org_id` |
| update | PATCH | admin, owner | Input: `name`, `role` |
| delete | DELETE | admin | Soft delete |

**Relations:**

| Name | Type | Key |
| --- | --- | --- |
| `organization` | belongs_to | `org_id` -> `organizations.id` |

**Indexes:**

| Fields | Options |
| --- | --- |
| `email` | unique |
| `org_id`, `role` | composite |
| `created_at` | descending |

**Example:**

```bash
shaperail resource create members --archetype user
```

## content

A content/article resource with title, slug, body, status, and author. Includes
caching, search, soft delete, and a unique slug index.

**When to use:** Blog posts, articles, pages, documentation entries, or any
resource that has a title, body, publication status, and a URL-friendly slug.

**Schema fields:**

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key, auto-generated |
| `title` | string | Required, 1-500 chars |
| `slug` | string | Unique, required |
| `body` | string | Required |
| `status` | enum | Values: `draft`, `published`, `archived`. Default: `draft` |
| `author_id` | uuid | Required |
| `created_at` | timestamp | Auto-generated |
| `updated_at` | timestamp | Auto-generated |

**Endpoints:**

| Action | Method | Auth | Notes |
| --- | --- | --- | --- |
| list | GET | public | Filters: `status`, `author_id`. Search: `title`, `body`. Sort: `created_at`, `title`. Cursor pagination. Cache: 60s TTL, invalidated on create/update/delete |
| get | GET | public | Cache: 300s TTL |
| create | POST | admin, member | Input: `title`, `slug`, `body`, `status`, `author_id` |
| update | PATCH | admin, owner | Input: `title`, `slug`, `body`, `status` |
| delete | DELETE | admin | Soft delete |

**Indexes:**

| Fields | Options |
| --- | --- |
| `slug` | unique |
| `author_id`, `status` | composite |
| `created_at` | descending |

**Example:**

```bash
shaperail resource create articles --archetype content
```

## tenant

A tenant-scoped resource with automatic row-level isolation. The `tenant_key`
field at the top level of the YAML tells Shaperail to automatically filter all
queries by the tenant ID from the JWT.

**When to use:** Any resource in a multi-tenant application where rows must be
isolated by organization, company, or workspace. Projects, settings,
invitations, or any org-scoped data.

**Schema fields:**

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key, auto-generated |
| `org_id` | uuid | Required, references `organizations.id`. Used as `tenant_key` |
| `name` | string | Required, 1-200 chars |
| `created_at` | timestamp | Auto-generated |
| `updated_at` | timestamp | Auto-generated |

**Top-level key:**

```yaml
tenant_key: org_id
```

This causes Shaperail to:
1. Automatically add a `WHERE org_id = <jwt.tenant_id>` clause to all queries
2. Scope cache keys by tenant
3. Scope rate limits by tenant
4. Allow `super_admin` role to bypass tenant filtering

**Endpoints:**

| Action | Method | Auth | Notes |
| --- | --- | --- | --- |
| list | GET | member, admin | Filters: `org_id`. Search: `name`. Cursor pagination |
| get | GET | member, admin | |
| create | POST | admin | Input: `org_id`, `name` |
| update | PATCH | admin | Input: `name` |
| delete | DELETE | admin | Soft delete |

**Relations:**

| Name | Type | Key |
| --- | --- | --- |
| `organization` | belongs_to | `org_id` -> `organizations.id` |

**Indexes:**

| Fields | Options |
| --- | --- |
| `org_id` | single |
| `created_at` | descending |

**Example:**

```bash
shaperail resource create projects --archetype tenant
```

See the [Multi-tenancy guide]({{ '/multi-tenancy/' | relative_url }}) for more
on tenant isolation, JWT claims, and `super_admin` bypass.

## lookup

Simple reference data with a code and label. Heavily cached, uses offset
pagination, and has a unique index on the code field.

**When to use:** Countries, currencies, categories, statuses, tags, or any
small, slowly-changing reference table that other resources reference by code.

**Schema fields:**

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key, auto-generated |
| `code` | string | Unique, required, 1-50 chars |
| `label` | string | Required, 1-200 chars |

**Endpoints:**

| Action | Method | Auth | Notes |
| --- | --- | --- | --- |
| list | GET | public | Sort: `code`, `label`. Offset pagination. Cache: 3600s TTL |
| get | GET | public | Cache: 3600s TTL |
| create | POST | admin | Input: `code`, `label` |
| update | PATCH | admin | Input: `label` (code is immutable after creation) |
| delete | DELETE | admin | Hard delete |

**Indexes:**

| Fields | Options |
| --- | --- |
| `code` | unique |

**Example:**

```bash
shaperail resource create countries --archetype lookup
```

Note that lookup resources use offset pagination (not cursor) and have long
cache TTLs because the data changes infrequently.

## Choosing an archetype

| Scenario | Archetype |
| --- | --- |
| Generic starting point | `basic` |
| User accounts or people | `user` |
| Articles, posts, pages | `content` |
| Org-scoped / tenant-isolated data | `tenant` |
| Small reference tables | `lookup` |

Every archetype produces valid YAML that passes `shaperail validate` and
compiles on first run. After scaffolding, edit the resource file to add, remove,
or change fields and endpoints to match your requirements.
