use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/index.html");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/vite.config.js");

    let build_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    println!("cargo:rustc-env=BUILD_ID={build_id}");

    let frontend_dir = std::path::Path::new("frontend");

    if !frontend_dir.join("node_modules").exists() {
        let status = Command::new("npm")
            .arg("install")
            .current_dir(frontend_dir)
            .status()
            .expect("Failed to run npm install — is Node.js installed?");
        assert!(status.success(), "npm install failed");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run npm build — is Node.js installed?");
    assert!(status.success(), "Frontend build failed");
}
