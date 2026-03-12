---
title: Build explicit Rust APIs from one schema
description: Shaperail turns declarative YAML resources into a production-ready Rust backend with Docker-first local development, live browser docs, and deterministic OpenAPI output.
eyebrow: Product documentation
---

<p class="lead">Shaperail is a framework for teams that want a small source of truth, predictable generation, and a runtime that behaves exactly like the schema says it should.</p>

<div class="cta-row">
  <a class="button button-primary" href="{{ '/getting-started/' | relative_url }}">Start with the quickstart</a>
  <a class="button button-secondary" href="https://crates.io/crates/shaperail-cli">Open crates.io</a>
</div>

<div class="card-grid">
  <div class="card">
    <h3>One schema, one runtime</h3>
    <p>Resources, routes, auth rules, relations, and docs all come from the same YAML definitions.</p>
  </div>
  <div class="card">
    <h3>Docker-first local setup</h3>
    <p>New apps ship with Postgres and Redis already wired. Users should not have to create databases by hand.</p>
  </div>
  <div class="card">
    <h3>Generated docs you can trust</h3>
    <p>Every app serves browser docs and an OpenAPI 3.1 document generated from the declared resource surface.</p>
  </div>
</div>

## The shortest correct path

```bash
cargo install shaperail-cli
shaperail init my-app
cd my-app
docker compose up -d
shaperail serve
```

Open the generated app:

- `http://localhost:3000/docs`
- `http://localhost:3000/openapi.json`
- `http://localhost:3000/health`

## What makes Shaperail different

<div class="metric-grid">
  <div class="metric-card">
    <strong>Explicit</strong>
    <p>No hidden route generation. If an endpoint is not declared, it does not exist.</p>
  </div>
  <div class="metric-card">
    <strong>Flat</strong>
    <p>The resource file maps directly to runtime behavior. There is no deep abstraction stack to memorize.</p>
  </div>
  <div class="metric-card">
    <strong>Deterministic</strong>
    <p>OpenAPI output, route registration, and validation behavior stay aligned with the schema.</p>
  </div>
  <div class="metric-card">
    <strong>Production-ready</strong>
    <p>Apps start with auth, observability, health checks, migrations, and Docker wiring already in place.</p>
  </div>
</div>

## What you actually author

These are the files a Shaperail user edits in day-to-day work:

| File | Why it matters |
| --- | --- |
| `resources/*.yaml` | Defines schema, endpoints, auth rules, relations, filters, pagination, and indexes |
| `migrations/*.sql` | Stores the SQL that changes the running database |
| `shaperail.config.yaml` | Holds service-level settings such as port, DB, cache, and auth config |
| `.env` | Connects the app to local or deployed services |
| `docker-compose.yml` | Boots Postgres and Redis for development |

## Recommended reading order

<div class="workflow">
  <div class="workflow-step">
    <strong>1. Getting started</strong>
    <span>Scaffold the app, boot Docker, and verify the live docs in a browser.</span>
  </div>
  <div class="workflow-step">
    <strong>2. Resource guide</strong>
    <span>Learn the exact YAML contract and which keys control generation.</span>
  </div>
  <div class="workflow-step">
    <strong>3. Blog API example</strong>
    <span>See a complete two-resource app with relations, ownership, and checked-in migrations.</span>
  </div>
  <div class="workflow-step">
    <strong>4. CLI reference</strong>
    <span>Use the commands that drive validation, migrations, route inspection, and Docker builds.</span>
  </div>
</div>

## Core workflows

<div class="link-grid">
  <div class="card">
    <h3><a href="{{ '/getting-started/' | relative_url }}">Start your first app</a></h3>
    <p>Use the scaffold, boot the local services, and confirm docs, OpenAPI, and health checks.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/resource-guide/' | relative_url }}">Author resource files</a></h3>
    <p>Define fields, endpoint contracts, relations, and indexes in the canonical YAML format.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/auth-and-ownership/' | relative_url }}">Lock down access</a></h3>
    <p>Model public routes, roles, and owner-based writes without scattering auth logic across files.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/migrations-and-schema-changes/' | relative_url }}">Handle schema changes</a></h3>
    <p>Generate, review, and apply SQL migrations whenever the source schema changes.</p>
  </div>
</div>

<div class="callout">
  <p class="callout-label">Public entrypoint</p>
  <p><strong><code>shaperail.io</code> is the canonical docs and install domain.</strong> Point the GitHub Pages site and install script at the same domain so the framework has one public home.</p>
</div>
