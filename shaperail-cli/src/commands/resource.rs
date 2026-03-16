use std::fs;
use std::path::Path;

use shaperail_codegen::parser::parse_resource;

/// Scaffold a new resource YAML file and its initial migration.
pub fn run_create(name: &str) -> i32 {
    if let Err(e) = validate_resource_name(name) {
        eprintln!("Error: {e}");
        return 1;
    }

    let resources_dir = Path::new("resources");
    if !resources_dir.is_dir() {
        eprintln!("Error: No resources/ directory found. Run this from a Shaperail project root.");
        return 1;
    }

    let resource_path = resources_dir.join(format!("{name}.yaml"));
    if resource_path.exists() {
        eprintln!("Error: resources/{name}.yaml already exists");
        return 1;
    }

    let yaml = scaffold_resource_yaml(name);

    // Validate the generated YAML to ensure it's correct
    if let Err(e) = parse_resource(&yaml) {
        eprintln!("Internal error: generated YAML is invalid: {e}");
        return 1;
    }

    if let Err(e) = fs::write(&resource_path, &yaml) {
        eprintln!("Error writing {}: {e}", resource_path.display());
        return 1;
    }
    println!("Created {}", resource_path.display());

    // Generate migration SQL
    let migrations_dir = Path::new("migrations");
    if migrations_dir.is_dir() {
        match generate_initial_migration(name, &yaml, migrations_dir) {
            Ok(path) => println!("Created {path}"),
            Err(e) => eprintln!("Warning: could not generate migration: {e}"),
        }
    }

    println!();
    println!("Next steps:");
    println!("  1. Edit resources/{name}.yaml to add your fields");
    println!("  2. Run: shaperail validate");
    println!("  3. Run: shaperail migrate");
    println!("  4. Run: shaperail serve");
    0
}

fn validate_resource_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Resource name cannot be empty".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(
            "Resource name must be lowercase alphanumeric with underscores (e.g., 'blog_posts')"
                .into(),
        );
    }
    if name.starts_with('_') || name.starts_with(|c: char| c.is_ascii_digit()) {
        return Err("Resource name must start with a letter".into());
    }
    Ok(())
}

fn scaffold_resource_yaml(name: &str) -> String {
    format!(
        r#"resource: {name}
version: 1

schema:
  id:         {{ type: uuid, primary: true, generated: true }}
  # Add your fields here, for example:
  # title:    {{ type: string, required: true, min: 1, max: 200 }}
  # status:   {{ type: enum, values: [draft, published, archived], default: draft }}
  created_at: {{ type: timestamp, generated: true }}
  updated_at: {{ type: timestamp, generated: true }}

endpoints:
  list:
    method: GET
    path: /{name}
    auth: public
    pagination: cursor

  get:
    method: GET
    path: /{name}/:id
    auth: public

  create:
    method: POST
    path: /{name}
    auth: [admin]
    input: []

  update:
    method: PATCH
    path: /{name}/:id
    auth: [admin]
    input: []

  delete:
    method: DELETE
    path: /{name}/:id
    auth: [admin]
"#
    )
}

fn generate_initial_migration(
    name: &str,
    yaml: &str,
    migrations_dir: &Path,
) -> Result<String, String> {
    let resource = parse_resource(yaml).map_err(|e| format!("Failed to parse resource: {e}"))?;
    let sql = super::migrate::render_migration_sql(&resource);

    let existing = fs::read_dir(migrations_dir)
        .map_err(|e| format!("Failed to read migrations/: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().is_some_and(|n| n.ends_with(".sql")))
        .count();

    let next_num = existing + 1;
    let filename = format!("{next_num:04}_create_{name}.sql");
    let path = migrations_dir.join(&filename);

    fs::write(&path, &sql).map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_resource_name_accepts_valid() {
        assert!(validate_resource_name("users").is_ok());
        assert!(validate_resource_name("blog_posts").is_ok());
        assert!(validate_resource_name("orders2").is_ok());
    }

    #[test]
    fn validate_resource_name_rejects_invalid() {
        assert!(validate_resource_name("").is_err());
        assert!(validate_resource_name("Users").is_err());
        assert!(validate_resource_name("blog-posts").is_err());
        assert!(validate_resource_name("_private").is_err());
        assert!(validate_resource_name("2things").is_err());
    }

    #[test]
    fn scaffolded_yaml_parses_successfully() {
        let yaml = scaffold_resource_yaml("comments");
        let rd = parse_resource(&yaml).expect("scaffolded YAML must parse");
        assert_eq!(rd.resource, "comments");
        assert_eq!(rd.version, 1);
        assert!(rd.schema.contains_key("id"));
        assert!(rd.schema.contains_key("created_at"));
        assert!(rd.schema.contains_key("updated_at"));
        assert!(rd.endpoints.is_some());
    }
}
