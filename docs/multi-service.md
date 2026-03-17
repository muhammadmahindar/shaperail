---
title: Multi-service workspaces
parent: Guides
nav_order: 13
---

# Multi-service workspaces

Shaperail can parse a workspace definition and start multiple services from one
root, but the current workspace support is intentionally lighter than the full
vision described in older docs.

## What `shaperail serve --workspace` does today

From a workspace root, the CLI currently:

1. loads `shaperail.workspace.yaml`
2. validates service directories and resource files
3. loads saga files if present
4. computes dependency order from `depends_on`
5. spawns `cargo run` once per service with `SHAPERAIL_PORT` set

It does **not** currently:

- register services in Redis automatically
- start service heartbeats
- merge `shared:` config into each service's runtime bootstrap
- generate or wire typed inter-service clients
- orchestrate saga execution

## Workspace layout

```text
my-platform/
├── shaperail.workspace.yaml
├── sagas/
│   └── create_order.saga.yaml
└── services/
    ├── users-api/
    │   ├── shaperail.config.yaml
    │   ├── resources/
    │   └── src/main.rs
    └── orders-api/
        ├── shaperail.config.yaml
        ├── resources/
        └── src/main.rs
```

## `shaperail.workspace.yaml`

```yaml
workspace: my-platform

services:
  users-api:
    path: services/users-api
    port: 3001
  orders-api:
    path: services/orders-api
    port: 3002
    depends_on: [users-api]

shared:
  cache:
    type: redis
    url: redis://localhost:6379
  auth:
    provider: jwt
    secret_env: JWT_SECRET
    expiry: 24h
```

### Fields

| Field | Type | Description |
| --- | --- | --- |
| `workspace` | string | Workspace name. |
| `services` | map | Named services to start. |
| `shared` | object | Shared metadata parsed by the workspace schema. |

### Service fields

| Field | Type | Description |
| --- | --- | --- |
| `path` | string | Relative path to the service directory. |
| `port` | integer | HTTP port passed through `SHAPERAIL_PORT`. |
| `depends_on` | list | Startup ordering dependencies. |

Current limitation: `shared:` is parsed and validated, but it is not merged
into each spawned service automatically by `serve --workspace`.

## Starting the workspace

```bash
cd my-platform
shaperail serve --workspace
```

Services start in dependency order and each runs as its own child process.

## Service registry and typed clients

The runtime contains a Redis-backed `ServiceRegistry`, and the codebase also
contains client/codegen primitives for future multi-service work. Those pieces
are not wired into `serve --workspace` yet.

So today:

- the workspace runner is a coordinated process launcher
- registry/discovery is still a manual integration task
- typed inter-service clients are not emitted by the normal workspace flow

## Sagas

Saga YAML files are still parsed and validated.

Example:

```yaml
saga: create_order
version: 1
steps:
  - name: reserve_inventory
    service: inventory-api
    action: POST /v1/reservations
    compensate: DELETE /v1/reservations/:id
    timeout_secs: 5
```

Current limitation: `serve --workspace` only loads and reports saga files. It
does not run a saga orchestrator or execute those steps automatically.

## What this is useful for today

- validating a multi-service repo layout
- starting several Shaperail services together in dependency order
- keeping ports and service directories coordinated from one root

If you need registry heartbeats, discovery, typed clients, or real saga
execution, plan on wiring those pieces manually for now.
