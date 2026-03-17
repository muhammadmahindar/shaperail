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
- protected post creation with a before-controller (`prepare_post`)
- owner-based post and comment updates through `created_by`
- post/comment relations
- cursor pagination on posts
- offset pagination on comments
- soft delete on posts
- rich controller patterns across both resources

## Files

- [resources/posts.yaml](./resources/posts.yaml) — post schema, endpoints, controller declarations
- [resources/posts.controller.rs](./resources/posts.controller.rs) — `prepare_post`, `enforce_edit_rules`, `cleanup_comments`
- [resources/comments.yaml](./resources/comments.yaml) — comment schema, endpoints, controller declarations
- [resources/comments.controller.rs](./resources/comments.controller.rs) — `validate_comment`, `check_comment_ownership`
- [migrations/0001_create_posts.sql](./migrations/0001_create_posts.sql)
- [migrations/0002_create_comments.sql](./migrations/0002_create_comments.sql)
- [requests.http](./requests.http) — sample HTTP requests with versioned URLs

## Controllers

### Posts

**`prepare_post`** (before create) — Prepares a new post for insertion:
- Auto-fills `created_by` from the authenticated user's JWT so the client never sends it.
- Generates a URL-safe `slug` from the title (lowercase, hyphens, special characters stripped).
- Defaults `status` to `"draft"` when the client omits it.
- Validates that `body` is not empty or whitespace-only.

**`enforce_edit_rules`** (before update) — Guards post editing with business rules:
- Blocks edits to archived posts entirely.
- Non-admin users cannot change `status` to `"published"`.
- Reverting a published post to draft requires an `X-Edit-Reason` request header.
- Auto-updates the `slug` when the title changes.

**`cleanup_comments`** (after delete) — Post-deletion bookkeeping:
- Queries the count of comments that belonged to the deleted post.
- Adds an `X-Comments-Archived` response header with the count.
- Logs the post ID and comment count via `tracing`.

### Comments

**`validate_comment`** (before create) — Validates a new comment before insertion:
- Checks that the referenced post exists and has `status: published` (rejects draft/archived).
- Auto-fills `created_by` from the JWT if authenticated.
- Strips HTML tags from the comment body as basic XSS prevention.
- Rate-limits users to 10 comments per hour via a DB count query; returns 429 if exceeded.

**`check_comment_ownership`** (before update) — Enforces ownership and edit windows:
- Verifies the user owns the comment or has the `admin` role; returns 403 otherwise.
- Non-admin users cannot edit comments older than 15 minutes.

### Patterns Demonstrated

| Pattern                  | Controller            | How                                        |
|--------------------------|-----------------------|--------------------------------------------|
| Auto-fill from JWT       | `prepare_post`        | `ctx.user.id` into `ctx.input["created_by"]` |
| Derived fields           | `prepare_post`        | Slug generated from title                  |
| Default values           | `prepare_post`        | Status defaults to `"draft"`               |
| Input validation         | `prepare_post`        | Body cannot be whitespace-only             |
| DB lookups in controller | `enforce_edit_rules`  | Fetches current post status from DB        |
| Role-based logic         | `enforce_edit_rules`  | Only admins can publish                    |
| Required headers         | `enforce_edit_rules`  | `X-Edit-Reason` for status revert          |
| Cross-resource checks    | `validate_comment`    | Verifies referenced post is published      |
| XSS prevention           | `validate_comment`    | Strips HTML tags from body                 |
| Rate limiting            | `validate_comment`    | Max 10 comments/user/hour via DB query     |
| Ownership enforcement    | `check_comment_ownership` | Owner or admin check               |
| Time-based edit window   | `check_comment_ownership` | 15-minute edit window for non-admins |
| Response headers         | `cleanup_comments`    | `X-Comments-Archived` header               |
| After-controller logging | `cleanup_comments`    | `tracing::info!` with structured fields    |

## Notes

- `owner` auth works by comparing the token user ID to `created_by`
- this example keeps reads public and requires auth only for writes
- the app uses the standard Rust scaffold created by `shaperail init`
- all routes are prefixed with `/v1/` because both resources set `version: 1`
- resources omit `db:` so they use the default connection; with `databases:` in config you can set `db: <name>` per resource
