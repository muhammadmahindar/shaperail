use std::process::Command;

/// Start dev server with hot reload via cargo-watch.
pub fn run(port: Option<u16>) -> i32 {
    let config = match super::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    let port = port.unwrap_or(config.port);

    // Try cargo-watch first for hot reload
    let has_cargo_watch = Command::new("cargo")
        .args(["watch", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_cargo_watch {
        println!("Starting dev server on port {port} with hot reload...");
        let status = Command::new("cargo")
            .args([
                "watch",
                "-x",
                &format!("run -- --port {port}"),
                "-w",
                "src",
                "-w",
                "resources",
            ])
            .env("STEEL_PORT", port.to_string())
            .status();

        match status {
            Ok(s) => s.code().unwrap_or(1),
            Err(e) => {
                eprintln!("Failed to start cargo-watch: {e}");
                1
            }
        }
    } else {
        println!("cargo-watch not found, starting without hot reload...");
        println!("Install cargo-watch for hot reload: cargo install cargo-watch");
        println!("Starting dev server on port {port}...");

        let status = Command::new("cargo")
            .args(["run"])
            .env("STEEL_PORT", port.to_string())
            .status();

        match status {
            Ok(s) => s.code().unwrap_or(1),
            Err(e) => {
                eprintln!("Failed to start server: {e}");
                1
            }
        }
    }
}
