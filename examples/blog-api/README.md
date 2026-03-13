# Blog API Example

This example shows the files a Shaperail user actually authors:

- `resources/*.yaml` — schema and endpoint declarations
- `resources/*.controller.rs` — business logic for controllers declared in YAML
- `migrations/*.sql` — database schema
- `shaperail.config.yaml` — project configuration
- `.env` — environment variables
- `docker-compose.yml` — local Postgres and Redis

## Quick Start

```bash
shaperail init blog-api
cd blog-api
```

Then copy these files into your app:

- `examples/blog-api/shaperail.config.yaml`
- `examples/blog-api/docker-compose.yml`
- `examples/blog-api/.env.example` as `.env`
- `examples/blog-api/resources/*.yaml`
- `examples/blog-api/resources/*.controller.rs`
- `examples/blog-api/migrations/*.sql`

After that:

```bash
docker compose up -d
shaperail serve
```

Open:

- `http://localhost:3000/docs` — interactive API docs
- `http://localhost:3000/openapi.json` — OpenAPI 3.1 spec

All API endpoints are versioned based on each resource's `version` field:

- `http://localhost:3000/v1/posts` — list posts
- `http://localhost:3000/v1/comments` — list comments

## What This Example Covers

- versioned API endpoints (`/v1/posts`, `/v1/comments`)
- public blog post reads
- protected post creation with a before-controller (`set_created_by`)
- owner-based post and comment updates through `created_by`
- post/comment relations
- cursor pagination on posts
- offset pagination on comments
- soft delete on posts
- single-database config (`database:`); for multi-DB use `databases:` in config and optional `db:` on resources (see [Configuration reference](https://shaperail.dev/configuration/#databases-multi-database))

## Files

- [resources/posts.yaml](./resources/posts.yaml) — post schema, endpoints, controller declaration
- [resources/posts.controller.rs](./resources/posts.controller.rs) — `set_created_by` business logic
- [resources/comments.yaml](./resources/comments.yaml) — comment schema and endpoints
- [migrations/0001_create_posts.sql](./migrations/0001_create_posts.sql)
- [migrations/0002_create_comments.sql](./migrations/0002_create_comments.sql)
- [requests.http](./requests.http) — sample HTTP requests with versioned URLs

## Controllers

The posts resource declares a `controller: { before: set_created_by }` on the
create endpoint. The matching function lives in `resources/posts.controller.rs`
and auto-fills `created_by` from the authenticated user's JWT token, so the
client doesn't need to send it explicitly.

## Notes

- `owner` auth works by comparing the token user ID to `created_by`
- this example keeps reads public and requires auth only for writes
- the app uses the standard Rust scaffold created by `shaperail init`
- all routes are prefixed with `/v1/` because both resources set `version: 1`
- resources omit `db:` so they use the default connection; with `databases:` in config you can set `db: <name>` per resource
