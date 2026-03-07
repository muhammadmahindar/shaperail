pub mod build;
pub mod doctor;
pub mod export;
pub mod generate;
pub mod init;
pub mod jobs_status;
pub mod migrate;
pub mod routes;
pub mod seed;
pub mod serve;
pub mod test;
pub mod validate;

use std::path::{Path, PathBuf};

use steel_core::ResourceDefinition;

/// Collect all .yaml/.yml resource files from a directory.
pub fn collect_resource_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Parse all resource files from the resources/ directory.
pub fn load_all_resources() -> Result<Vec<ResourceDefinition>, String> {
    let resources_dir = Path::new("resources");
    if !resources_dir.is_dir() {
        return Err("No resources/ directory found. Run this from a SteelAPI project root.".into());
    }
    let files = collect_resource_files(resources_dir)
        .map_err(|e| format!("Failed to read resources/ directory: {e}"))?;
    if files.is_empty() {
        return Err("No resource files found in resources/".into());
    }

    let mut resources = Vec::new();
    for file in &files {
        let rd = steel_codegen::parser::parse_resource_file(file)
            .map_err(|e| format!("{}: {e}", file.display()))?;
        resources.push(rd);
    }
    Ok(resources)
}

/// Load steel.config.yaml from the current directory.
pub fn load_config() -> Result<steel_core::ProjectConfig, String> {
    let path = Path::new("steel.config.yaml");
    if !path.exists() {
        return Err("No steel.config.yaml found. Run this from a SteelAPI project root.".into());
    }
    steel_codegen::config_parser::parse_config_file(path)
        .map_err(|e| format!("Failed to parse steel.config.yaml: {e}"))
}
