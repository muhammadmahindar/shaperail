---
title: Multi-tenancy
parent: Guides
nav_order: 14
---

Shaperail supports automatic row-level multi-tenancy. Add a single top-level
key to a resource file and the framework scopes every query, cache entry, and
rate limit to the authenticated user's tenant.

## Enabling multi-tenancy on a resource

Add `tenant_key` at the top level of the resource YAML. The value is the name
of a `uuid` field in the schema that identifies the tenant:

```yaml
resource: projects
version: 1
tenant_key: org_id

schema:
  id:         { type: uuid, primary: true, generated: true }
  org_id:     { type: uuid, ref: organizations.id, required: true }
  name:       { type: string, min: 1, max: 200, required: true }
  status:     { type: enum, values: [active, archived], default: active }
  created_at: { type: timestamp, generated: true }
  updated_at: { type: timestamp, generated: true }

endpoints:
  list:
    method: GET
    path: /projects
    auth: [member, admin]
    filters: [status]
    pagination: cursor

  create:
    method: POST
    path: /projects
    auth: [member, admin]
    input: [name, status]

  update:
    method: PATCH
    path: /projects/:id
    auth: [member, admin]
    input: [name, status]

  delete:
    method: DELETE
    path: /projects/:id
    auth: [admin]
```

That is the only change needed. The framework handles the rest.

## How it works

### JWT tenant_id claim

The tenant ID is extracted from the JWT `tenant_id` claim. Include it when
issuing tokens for your users:

```json
{
  "sub": "user-123",
  "role": "member",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Use `JwtConfig::encode_access_with_tenant()` to encode tokens with a tenant
claim:

```rust
let token = jwt_config.encode_access_with_tenant(
    "user-123",
    "member",
    Some("550e8400-e29b-41d4-a716-446655440000"),
)?;
```

### Automatic query scoping

When `tenant_key` is set and the user has a `tenant_id` claim:

| Operation | What happens |
| --- | --- |
| **List** | `WHERE org_id = $tenant_id` is added to every query automatically |
| **Get** | Record is fetched, then verified to belong to the user's tenant. Returns 404 if it does not match. |
| **Create** | `org_id` is auto-injected into the input data from the JWT claim (if not already provided) |
| **Update** | Record is pre-fetched and tenant is verified before the write proceeds |
| **Delete** | Record is pre-fetched and tenant is verified before deletion |

A user in tenant A will never see, modify, or delete records belonging to
tenant B. Attempts return 404 (not 403) to avoid leaking information about
other tenants' data.

### super_admin bypass

Users with the role `super_admin` bypass all tenant filtering. They can:

- List records across all tenants
- Read, update, and delete any tenant's records

This is useful for platform admin dashboards, support tools, and data
migrations.

### Cache isolation

Cache keys include the tenant ID, so cached responses are never shared across
tenants:

```
shaperail:projects:list:<hash>:org-abc:member
shaperail:projects:list:<hash>:org-xyz:member   # separate entry
```

When a user with no `tenant_id` claim makes a request, the tenant segment is
`_` (underscore placeholder).

### Rate limit isolation

Rate limit keys are scoped per tenant so each tenant gets its own independent
rate limit budget:

```
shaperail:ratelimit:t:org-abc:user:user-123
shaperail:ratelimit:t:org-xyz:user:user-456   # independent counter
```

### Controller access

The `tenant_id` is available in controller functions via `ctx.tenant_id`:

```rust
pub async fn check_project_limit(ctx: &mut ControllerContext) -> Result<(), ShaperailError> {
    if let Some(tenant_id) = &ctx.tenant_id {
        // Custom logic using the tenant ID
        tracing::info!(tenant = %tenant_id, "Checking project limit");
    }
    Ok(())
}
```

## Validation

`shaperail validate` checks that:

- The `tenant_key` field exists in the resource schema
- The field has `type: uuid`

Invalid configurations produce clear error messages:

```
resource 'projects': tenant_key 'org_id' not found in schema
resource 'projects': tenant_key 'org_name' must reference a uuid field, found string
```

## Mixing tenant and non-tenant resources

Not every resource needs multi-tenancy. Only resources with `tenant_key` get
automatic scoping. Resources without it work exactly as before.

A typical SaaS pattern:

```yaml
# resources/organizations.yaml — no tenant_key (orgs ARE the tenants)
resource: organizations
version: 1
schema:
  id:   { type: uuid, primary: true, generated: true }
  name: { type: string, required: true }

# resources/projects.yaml — tenant-scoped
resource: projects
version: 1
tenant_key: org_id
schema:
  id:     { type: uuid, primary: true, generated: true }
  org_id: { type: uuid, ref: organizations.id, required: true }
  name:   { type: string, required: true }

# resources/tasks.yaml — also tenant-scoped
resource: tasks
version: 1
tenant_key: org_id
schema:
  id:         { type: uuid, primary: true, generated: true }
  org_id:     { type: uuid, ref: organizations.id, required: true }
  project_id: { type: uuid, ref: projects.id, required: true }
  title:      { type: string, required: true }
```

## What multi-tenancy does NOT do

- **Schema-per-tenant** -- Shaperail uses row-level isolation (shared tables
  with a tenant column), not separate schemas or databases per tenant.
- **Auto-create tenants** -- You must create the tenant (e.g., organization)
  records separately. The framework only filters by the declared key.
- **Auto-fill tenant_id in JWT** -- Your auth service must include `tenant_id`
  in the JWT claims. Shaperail reads it but does not generate it.
