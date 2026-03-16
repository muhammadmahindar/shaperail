---
title: Troubleshooting
parent: Reference
nav_order: 5
---

Common errors and how to fix them.

## YAML parsing errors

### Unknown field

```
unknown field `cache_ttl`, expected one of `method`, `path`, `auth`, ...
```

You used a field name that doesn't exist. The error lists all valid fields.
Common mistakes:

| Wrong | Correct |
| --- | --- |
| `cache_ttl: 60` | `cache: { ttl: 60 }` |
| `hooks: [validate]` | `controller: { before: validate }` |
| `type: str` | `type: string` |
| `method: get` | `method: GET` |

### Missing field

```
missing field `resource`
```

A required top-level key is missing. Every resource file must have `resource`,
`version`, and `schema`.

### Wrong type

```
invalid value: string "get", expected one of `GET`, `POST`, `PATCH`, `PUT`, `DELETE`
```

Enum values are case-sensitive. Use uppercase HTTP methods.

## Feature flags

### GraphQL / gRPC not working

If you added `protocols: [graphql]` to your config but the endpoint doesn't
appear, check your `Cargo.toml`:

```toml
# This won't work — graphql feature is not enabled:
shaperail-runtime = { version = "0.6.0", default-features = false }

# This will:
shaperail-runtime = { version = "0.6.0", default-features = false, features = ["graphql"] }
```

### WASM plugins silently ignored

If your resource declares `controller: { before: "wasm:plugins/hook.wasm" }`
but the hook doesn't run, enable the feature:

```toml
shaperail-runtime = { version = "0.6.0", default-features = false, features = ["wasm-plugins"] }
```

Without the feature, WASM prefixed controllers return an error at runtime.

## Available features

| Feature | What it enables |
| --- | --- |
| `graphql` | `POST /graphql` endpoint via async-graphql |
| `grpc` | gRPC server via tonic on a separate port |
| `wasm-plugins` | WASM controller hooks via wasmtime |
| `multi-db` | MongoDB backend for resources with `db:` key |
| `observability-otlp` | OpenTelemetry OTLP span export |

All features are enabled by default when you don't specify `default-features = false`.

## Connection errors

| Problem | Fix |
| --- | --- |
| Cannot connect to Postgres | Run `docker compose ps`, confirm the service is healthy |
| Cannot connect to Redis | Same — check `docker compose ps` and `.env` `REDIS_URL` |
| Port already in use | Change the port in `docker-compose.yml` and update `.env` |
| `shaperail migrate` fails | Install `sqlx-cli`: `cargo install sqlx-cli` |

## Generated code

Files in `generated/` are overwritten on every `shaperail generate` and
`shaperail serve`. Never edit them by hand. If you need custom logic, use
controllers (`resources/<name>.controller.rs`) or WASM plugins.
