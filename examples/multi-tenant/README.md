# Multi-Tenant SaaS Example

A project management API demonstrating Shaperail's automatic multi-tenancy.
Organizations are tenants; projects and tasks are scoped by `org_id`.

## Resources

| Resource | Tenant-scoped | Description |
| --- | --- | --- |
| `organizations` | No | Tenant entities (the orgs themselves) |
| `projects` | Yes (`org_id`) | Projects within an organization |
| `tasks` | Yes (`org_id`) | Tasks within projects, also scoped to org |

## How it works

1. `projects.yaml` and `tasks.yaml` declare `tenant_key: org_id`
2. JWTs include a `tenant_id` claim matching the user's organization
3. All queries for projects and tasks are automatically filtered by `org_id`
4. Users with role `super_admin` can access all organizations' data

## Running

```bash
docker compose up -d
shaperail validate
shaperail migrate
shaperail seed
shaperail serve
```

## Testing tenant isolation

```bash
# Get a token for org-a member
TOKEN_A=$(curl -s localhost:3000/auth/token \
  -d '{"user_id":"user-1","role":"member","tenant_id":"ORG_A_UUID"}' \
  | jq -r .access_token)

# Get a token for org-b member
TOKEN_B=$(curl -s localhost:3000/auth/token \
  -d '{"user_id":"user-2","role":"member","tenant_id":"ORG_B_UUID"}' \
  | jq -r .access_token)

# User A only sees org A's projects
curl -H "Authorization: Bearer $TOKEN_A" localhost:3000/v1/projects

# User B only sees org B's projects
curl -H "Authorization: Bearer $TOKEN_B" localhost:3000/v1/projects

# super_admin sees everything
TOKEN_ADMIN=$(curl -s localhost:3000/auth/token \
  -d '{"user_id":"admin-1","role":"super_admin"}' \
  | jq -r .access_token)
curl -H "Authorization: Bearer $TOKEN_ADMIN" localhost:3000/v1/projects
```
