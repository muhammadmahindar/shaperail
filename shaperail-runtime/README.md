# shaperail-runtime

The Actix-web runtime for [Shaperail](https://shaperail.io). It powers the
generated app and also exposes the lower-level primitives used for jobs,
events, WebSockets, GraphQL, gRPC, and storage.

## Modules

| Module | Purpose |
|--------|---------|
| `db` | PostgreSQL connection pool, query generation, migrations, filtering, sorting, pagination, search |
| `handlers` | Actix-web route registration, CRUD handlers, response envelopes, field selection, relation loading |
| `auth` | JWT middleware, RBAC enforcement, token issuance, plus API key and rate-limiter primitives |
| `cache` | Redis connection pool, response caching, automatic invalidation |
| `jobs` | Redis-backed job queue, worker, retry with backoff, dead letter queue |
| `ws` | WebSocket session and room primitives |
| `storage` | File storage backends (local, S3, GCS, Azure), upload handling, image processing, signed URLs |
| `events` | Event emitter, webhook signing helpers, inbound webhook verification helpers |
| `observability` | Structured logging, Prometheus metrics, OpenTelemetry tracing, health checks |

## Usage

This crate is used by generated Shaperail applications. You typically do not
import it directly, but it is also the place to hook in manual worker, webhook,
API key, WebSocket, or workspace-related integrations that the scaffold does
not wire automatically yet.

```toml
[dependencies]
shaperail-runtime = "0.7.0"
```

## License

Dual-licensed under [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE).
