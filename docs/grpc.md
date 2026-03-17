---
title: gRPC
parent: Guides
nav_order: 13
---

When you enable gRPC, Shaperail starts a Tonic server alongside the HTTP
server. The resource YAML still drives the service surface, but the current
runtime support is partial.

## Enabling gRPC

Add the `grpc` feature to your app's `Cargo.toml`:

```toml
shaperail-runtime = { version = "0.7.0", default-features = false, features = ["grpc"] }
```

Then enable the protocol in `shaperail.config.yaml`:

```yaml
project: my-app
protocols: [rest, grpc]
```

The scaffolded app will start the gRPC server when both the Cargo feature and
the `grpc` protocol are enabled.

## Configuration

```yaml
grpc:
  port: 50051
  reflection: true
```

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `port` | integer | `50051` | Port for the gRPC server. |
| `reflection` | boolean | `true` | Enable server reflection for tools like `grpcurl`. |

## Current implementation status

The runtime currently handles these RPC families:

- `List*`
- `Stream*`
- `Get*`
- `Create*`
- `Delete*`

Important limitation: `Update*` requests are currently returned as
`UNIMPLEMENTED` even when an `update` endpoint exists on the resource.

## Request behavior today

The gRPC runtime is currently simpler than the proto generator suggests:

- `List*` returns up to 100 rows from the table.
- `Stream*` streams rows from the table.
- request filters, search, cursor pagination, and sort values are not applied
  by the current runtime handlers.

Treat the current gRPC support as a generated CRUD transport, not a full mirror
of the REST query surface yet.

## Authentication

Current gRPC auth uses JWT only:

- send `authorization: Bearer <token>` as gRPC metadata
- protected RPCs return `UNAUTHENTICATED` or `PERMISSION_DENIED` when auth
  checks fail

API key auth is not wired into the gRPC server path today.

Example:

```bash
grpcurl -plaintext \
  -H "authorization: Bearer eyJ..." \
  -d '{"id":"550e8400-e29b-41d4-a716-446655440000"}' \
  localhost:50051 shaperail.v1.users.UserService/GetUser
```

## Health checks and reflection

The gRPC server includes:

- `grpc.health.v1.Health`
- optional server reflection when `reflection: true`

Examples:

```bash
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
grpcurl -plaintext localhost:50051 list
```

## Proto generation

There are two separate pieces here:

- the codegen library contains a proto generator
- the CLI `shaperail generate` command currently writes Rust modules only

So `.proto` generation exists in the codebase, but it is not currently emitted
to disk by the normal CLI workflow. If you need proto files today, you must
call the codegen layer yourself or rely on reflection.

## Summary

| Feature | Current status |
| --- | --- |
| Server startup from scaffolded app | Yes |
| Health service | Yes |
| Reflection | Yes |
| JWT auth via metadata | Yes |
| `List*` / `Stream*` / `Get*` / `Create*` / `Delete*` | Yes |
| `Update*` | Not implemented |
| Runtime application of filters/search/cursor/sort | Not implemented |
| CLI-written `.proto` files | Not implemented |
