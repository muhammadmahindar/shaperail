# steel-cli

The developer-facing CLI for [SteelAPI](https://github.com/your-org/steel-api).

## Install

```bash
cargo install steel-cli
```

This installs the `steel` binary.

## Commands

```
steel init <name>          Scaffold a new SteelAPI project
steel generate             Generate Rust code from resource YAML files
steel serve                Start dev server with hot reload
steel build                Build release binary
steel build --docker       Build scratch-based Docker image
steel validate             Validate all resource files
steel test                 Run generated + custom tests
steel migrate              Generate + apply SQL migrations
steel migrate --rollback   Rollback last migration batch
steel seed                 Load fixture YAML files into database
steel export openapi       Export OpenAPI 3.1 spec
steel export sdk --lang ts Generate TypeScript client SDK
steel doctor               Check system dependencies
steel routes               Print all routes with auth requirements
steel jobs:status          Show job queue depth and recent failures
```

## Quick Start

```bash
steel init my-app
cd my-app
docker compose up -d
steel generate
steel migrate
steel serve
```

## License

Dual-licensed under [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE).
