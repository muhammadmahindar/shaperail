use std::path::Path;

/// Export OpenAPI 3.1 spec to stdout or a file.
pub fn run_openapi(output: Option<&Path>) -> i32 {
    let resources = match super::load_all_resources() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    let config = match super::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    let spec = generate_openapi_spec(&config, &resources);

    match output {
        Some(path) => {
            let content = if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                match serde_yaml::to_string(&spec) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error serializing YAML: {e}");
                        return 1;
                    }
                }
            } else {
                match serde_json::to_string_pretty(&spec) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error serializing JSON: {e}");
                        return 1;
                    }
                }
            };

            if let Err(e) = std::fs::write(path, content) {
                eprintln!("Error writing {}: {e}", path.display());
                return 1;
            }
            println!("OpenAPI spec written to {}", path.display());
        }
        None => match serde_json::to_string_pretty(&spec) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("Error serializing JSON: {e}");
                return 1;
            }
        },
    }

    0
}

/// Generate a TypeScript client SDK from the resource definitions.
pub fn run_sdk(lang: &str, output: Option<&Path>) -> i32 {
    if lang != "ts" && lang != "typescript" {
        eprintln!("Unsupported SDK language: '{lang}'. Supported: ts");
        return 1;
    }

    let resources = match super::load_all_resources() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    let output_dir = output.unwrap_or_else(|| Path::new("sdk"));
    if let Err(e) = std::fs::create_dir_all(output_dir) {
        eprintln!("Error creating SDK directory: {e}");
        return 1;
    }

    for resource in &resources {
        let ts_code = generate_ts_types(resource);
        let file_path = output_dir.join(format!("{}.ts", resource.resource));
        if let Err(e) = std::fs::write(&file_path, &ts_code) {
            eprintln!("Error writing {}: {e}", file_path.display());
            return 1;
        }
        println!("Generated {}", file_path.display());
    }

    // Generate index.ts
    let index: String = resources
        .iter()
        .map(|r| format!("export * from './{}';", r.resource))
        .collect::<Vec<_>>()
        .join("\n");
    if let Err(e) = std::fs::write(output_dir.join("index.ts"), format!("{index}\n")) {
        eprintln!("Error writing index.ts: {e}");
        return 1;
    }

    println!("TypeScript SDK generated in {}", output_dir.display());
    0
}

fn generate_openapi_spec(
    config: &steel_core::ProjectConfig,
    resources: &[steel_core::ResourceDefinition],
) -> serde_json::Value {
    let mut paths = serde_json::Map::new();
    let mut schemas = serde_json::Map::new();

    for resource in resources {
        let struct_name = to_pascal_case(&resource.resource);

        // Generate schema
        let mut properties = serde_json::Map::new();
        let mut required_fields = Vec::new();

        for (name, schema) in &resource.schema {
            let type_info = field_type_to_openapi(&schema.field_type);
            properties.insert(name.clone(), type_info);
            if schema.required && !schema.generated {
                required_fields.push(serde_json::Value::String(name.clone()));
            }
        }

        schemas.insert(
            struct_name.clone(),
            serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": required_fields,
            }),
        );

        // Generate paths from endpoints
        if let Some(endpoints) = &resource.endpoints {
            for (action, ep) in endpoints {
                let openapi_path = ep.path.replace(":id", "{id}");
                let method = ep.method.to_string().to_lowercase();

                let mut operation = serde_json::json!({
                    "operationId": format!("{}_{}", resource.resource, action),
                    "tags": [resource.resource],
                    "responses": {
                        "200": {
                            "description": "Successful response",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": format!("#/components/schemas/{struct_name}")
                                    }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "403": { "description": "Forbidden" },
                        "404": { "description": "Not found" },
                        "422": { "description": "Validation error" },
                        "429": { "description": "Rate limited" },
                        "500": { "description": "Internal server error" }
                    }
                });

                // Add auth info
                if let Some(auth) = &ep.auth {
                    operation["x-steelapi-auth"] =
                        serde_json::to_value(auth).unwrap_or(serde_json::Value::Null);
                }

                // Add hooks/events extensions
                if let Some(hooks) = &ep.hooks {
                    if !hooks.is_empty() {
                        operation["x-steelapi-hooks"] = serde_json::json!(hooks);
                    }
                }
                if let Some(events) = &ep.events {
                    if !events.is_empty() {
                        operation["x-steelapi-events"] = serde_json::json!(events);
                    }
                }

                let entry = paths
                    .entry(openapi_path)
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                if let serde_json::Value::Object(map) = entry {
                    map.insert(method, operation);
                }
            }
        }
    }

    serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": config.project,
            "version": "1.0.0"
        },
        "paths": paths,
        "components": {
            "schemas": schemas,
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                },
                "apiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key"
                }
            }
        }
    })
}

fn field_type_to_openapi(field_type: &steel_core::FieldType) -> serde_json::Value {
    match field_type {
        steel_core::FieldType::Uuid => serde_json::json!({ "type": "string", "format": "uuid" }),
        steel_core::FieldType::String => serde_json::json!({ "type": "string" }),
        steel_core::FieldType::Integer => serde_json::json!({ "type": "integer" }),
        steel_core::FieldType::Bigint => {
            serde_json::json!({ "type": "integer", "format": "int64" })
        }
        steel_core::FieldType::Number => serde_json::json!({ "type": "number" }),
        steel_core::FieldType::Boolean => serde_json::json!({ "type": "boolean" }),
        steel_core::FieldType::Timestamp => {
            serde_json::json!({ "type": "string", "format": "date-time" })
        }
        steel_core::FieldType::Date => serde_json::json!({ "type": "string", "format": "date" }),
        steel_core::FieldType::Enum => serde_json::json!({ "type": "string" }),
        steel_core::FieldType::Json => serde_json::json!({ "type": "object" }),
        steel_core::FieldType::Array => {
            serde_json::json!({ "type": "array", "items": {} })
        }
        steel_core::FieldType::File => serde_json::json!({ "type": "string", "format": "uri" }),
    }
}

fn generate_ts_types(resource: &steel_core::ResourceDefinition) -> String {
    let struct_name = to_pascal_case(&resource.resource);
    let mut fields = Vec::new();

    for (name, schema) in &resource.schema {
        let ts_type = match &schema.field_type {
            steel_core::FieldType::Uuid | steel_core::FieldType::String => "string",
            steel_core::FieldType::Integer
            | steel_core::FieldType::Bigint
            | steel_core::FieldType::Number => "number",
            steel_core::FieldType::Boolean => "boolean",
            steel_core::FieldType::Timestamp | steel_core::FieldType::Date => "string",
            steel_core::FieldType::Enum => "string",
            steel_core::FieldType::Json => "Record<string, unknown>",
            steel_core::FieldType::Array => "unknown[]",
            steel_core::FieldType::File => "string",
        };
        let optional = if schema.nullable || schema.generated {
            "?"
        } else {
            ""
        };
        fields.push(format!("  {name}{optional}: {ts_type};"));
    }

    format!(
        "export interface {struct_name} {{\n{}\n}}\n",
        fields.join("\n")
    )
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}
