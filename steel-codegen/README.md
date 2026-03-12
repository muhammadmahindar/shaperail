# steel-codegen

YAML parser, validator, and code generator for the [SteelAPI](https://github.com/muhammadmahindar/steel-api) framework.

## What it does

- **Parses** resource YAML files into `ResourceDefinition` structs
- **Validates** semantic correctness (enum needs values, refs must be uuid, etc.)
- **Generates** OpenAPI 3.1 specs from resource definitions
- **Generates** TypeScript client SDKs from OpenAPI specs

## Modules

| Module | Purpose |
|--------|---------|
| `parser` | YAML string → `ResourceDefinition` |
| `config_parser` | `steel.config.yaml` → `ProjectConfig` |
| `validator` | Semantic validation with human-readable errors |
| `openapi` | Resource definitions → OpenAPI 3.1 JSON/YAML |
| `typescript` | OpenAPI spec → TypeScript client SDK |

## Usage

```toml
[dependencies]
steel-codegen = "0.2"
```

```rust
use steel_codegen::parser::parse_resource;
use steel_codegen::validator::validate_resource;
use steel_codegen::openapi::generate_openapi;

let yaml = std::fs::read_to_string("resources/users.yaml")?;
let resource = parse_resource(&yaml)?;
let errors = validate_resource(&resource);
if errors.is_empty() {
    let spec = generate_openapi(&[resource]);
    println!("{}", serde_json::to_string_pretty(&spec)?);
}
```

## License

Dual-licensed under [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE).
