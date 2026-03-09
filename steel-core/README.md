# steel-core

Shared type definitions for the [SteelAPI](https://github.com/your-org/steel-api) framework.

This crate provides the foundational types that all other SteelAPI crates depend on:

- **`ResourceDefinition`** — The parsed representation of a resource YAML file
- **`FieldType`** — All supported schema types (uuid, string, integer, enum, json, etc.)
- **`FieldSchema`** — Field definition with constraints (required, unique, min/max, etc.)
- **`EndpointSpec`** — Endpoint configuration (method, path, auth, cache, hooks, events)
- **`AuthRule`** — Authentication rules (Public, Roles, Owner)
- **`RelationSpec`** — Relationships between resources (belongs_to, has_many, has_one)
- **`SteelError`** — Standardized error type with HTTP status codes
- **`ProjectConfig`** — Parsed `steel.config.yaml` project configuration
- **`ChannelDefinition`** — WebSocket channel configuration

## Usage

This crate is used internally by `steel-codegen` and `steel-runtime`. You typically don't need to depend on it directly unless you're building custom tooling around SteelAPI.

```toml
[dependencies]
steel-core = "0.2"
```

```rust
use steel_core::{ResourceDefinition, FieldType, SteelError};
```

## License

Dual-licensed under [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE).
