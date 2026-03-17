---
title: GraphQL
parent: Guides
nav_order: 12
---

When you enable GraphQL, Shaperail exposes generated GraphQL fields over a
single endpoint. The resource YAML is still the source of truth, but the
current GraphQL surface is narrower than the REST API.

## Enabling GraphQL

Add the `graphql` feature to your app's `Cargo.toml`:

```toml
shaperail-runtime = { version = "0.7.0", default-features = false, features = ["graphql"] }
```

Then enable the protocol in `shaperail.config.yaml`:

```yaml
project: my-app
protocols: [rest, graphql]
```

When both are present, the scaffolded server registers:

| URL | Purpose |
| --- | --- |
| `POST /graphql` | GraphQL endpoint. Send `{ "query": "...", "variables": { ... } }`. |
| `GET /graphql/playground` | GraphQL Playground for development. |

## Authentication

GraphQL reuses the same auth checks as REST for each generated field.

- JWT works out of the box through `Authorization: Bearer <token>`.
- API key auth is available only if you manually inject an `ApiKeyStore` into
  the Actix app. The scaffolded app does not do this for you.
- Owner checks run on generated `get`, `update`, and `delete` resolvers when
  the endpoint auth rule includes `owner`.

## Current schema shape

### List fields

For each resource with a `list` endpoint, GraphQL exposes:

```graphql
query {
  list_users(limit: 10, offset: 0) {
    id
    email
    name
    role
  }
}
```

Current limitation: list fields only accept `limit` and `offset`. REST filters,
search fields, sort fields, and cursor pagination are not exposed as GraphQL
arguments yet.

### Get-by-id fields

For each resource with a `get` endpoint, GraphQL exposes a singular field:

```graphql
query {
  user(id: "550e8400-e29b-41d4-a716-446655440000") {
    id
    email
    name
  }
}
```

### Mutations

For each resource with write endpoints, GraphQL exposes snake_case mutations:

```graphql
mutation {
  create_users(
    input: {
      email: "alice@example.com"
      name: "Alice"
      role: "member"
      org_id: "550e8400-e29b-41d4-a716-446655440000"
    }
  ) {
    id
    email
    name
  }
}
```

Generated mutation names are:

- `create_<resource>`
- `update_<resource>`
- `delete_<resource>`

Input object fields come from the endpoint's `input:` list. If no `input:` list
is present, the runtime falls back to all non-generated, non-primary fields.

## Relations

Relation fields are generated from the same `relations:` block used by REST:

- `belongs_to` becomes a single nested object
- `has_one` becomes a single nested object
- `has_many` becomes a nested list

That means a `user` query can request `organization` or `orders` if those
relations are declared on the resource.

## Configuration

Optional limits live under `graphql:` in `shaperail.config.yaml`:

```yaml
graphql:
  depth_limit: 10
  complexity_limit: 200
```

These values are parsed and passed into schema construction when GraphQL is
enabled.

## Summary

| Feature | Current status |
| --- | --- |
| Endpoint registration | Yes |
| Playground | Yes |
| List queries | Yes, but `limit` and `offset` only |
| Get-by-id queries | Yes |
| Nested relations | Yes |
| Create / update / delete mutations | Yes |
| JWT auth | Yes |
| API key auth | Manual `ApiKeyStore` wiring required |
| REST filters / search / sort in GraphQL args | Not yet |
