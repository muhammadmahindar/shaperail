use std::fs;
use std::path::Path;

/// Scaffold a new SteelAPI project with the correct directory structure.
pub fn run(name: &str) -> i32 {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        eprintln!("Error: directory '{name}' already exists");
        return 1;
    }

    if let Err(e) = scaffold(name, project_dir) {
        eprintln!("Error: {e}");
        return 1;
    }

    println!("Created SteelAPI project '{name}'");
    println!();
    println!("  cd {name}");
    println!("  steel serve");
    0
}

fn scaffold(name: &str, root: &Path) -> Result<(), String> {
    // Create directory structure
    let dirs = [
        "",
        "resources",
        "migrations",
        "hooks",
        "seeds",
        "tests",
        "channels",
        "src",
    ];

    for dir in &dirs {
        let path = root.join(dir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create {}: {e}", path.display()))?;
    }

    // steel.config.yaml
    let config = format!(
        r#"project: {name}
port: 3000
workers: auto

database:
  type: postgresql
  host: localhost
  port: 5432
  name: {db_name}
  pool_size: 20

cache:
  type: redis
  url: redis://localhost:6379

auth:
  provider: jwt
  secret_env: JWT_SECRET
  expiry: 24h
  refresh_expiry: 30d

logging:
  level: info
  format: json
"#,
        db_name = name.replace('-', "_")
    );
    write_file(&root.join("steel.config.yaml"), &config)?;

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
steel-runtime = {{ version = "0.2" }}
tokio = {{ version = "1", features = ["full"] }}
"#
    );
    write_file(&root.join("Cargo.toml"), &cargo_toml)?;

    // src/main.rs
    let main_rs = r#"fn main() {
    println!("Starting SteelAPI server...");
}
"#;
    write_file(&root.join("src/main.rs"), main_rs)?;

    // Example resource file
    let example_resource = r#"resource: posts
version: 1

schema:
  id:         { type: uuid, primary: true, generated: true }
  title:      { type: string, min: 1, max: 500, required: true }
  body:       { type: string, required: true }
  author_id:  { type: uuid, required: true }
  published:  { type: boolean, default: false }
  created_at: { type: timestamp, generated: true }
  updated_at: { type: timestamp, generated: true }

endpoints:
  list:
    method: GET
    path: /posts
    auth: public
    filters: [author_id, published]
    search: [title, body]
    pagination: cursor
    sort: [created_at, title]

  get:
    method: GET
    path: /posts/:id
    auth: public

  create:
    method: POST
    path: /posts
    auth: [admin, member]
    input: [title, body, author_id, published]

  update:
    method: PATCH
    path: /posts/:id
    auth: [admin, owner]
    input: [title, body, published]

  delete:
    method: DELETE
    path: /posts/:id
    auth: [admin]
    soft_delete: true
"#;
    write_file(&root.join("resources/posts.yaml"), example_resource)?;

    // .env
    let dotenv = format!(
        r#"DATABASE_URL=postgresql://steel:steel@localhost:5432/{db_name}
REDIS_URL=redis://localhost:6379
JWT_SECRET=change-me-in-production
"#,
        db_name = name.replace('-', "_")
    );
    write_file(&root.join(".env"), &dotenv)?;

    // .gitignore
    let gitignore = r#"/target
.env
*.swp
*.swo
"#;
    write_file(&root.join(".gitignore"), gitignore)?;

    // docker-compose.yml
    let docker_compose = format!(
        r#"services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: {db_name}
      POSTGRES_USER: steel
      POSTGRES_PASSWORD: steel
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U steel"]
      interval: 5s
      timeout: 3s
      retries: 10

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 10

volumes:
  postgres_data:
"#,
        db_name = name.replace('-', "_")
    );
    write_file(&root.join("docker-compose.yml"), &docker_compose)?;

    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}
