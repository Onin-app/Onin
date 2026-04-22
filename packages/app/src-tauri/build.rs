use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    for key in [
        "VITE_GLITCHTIP_DSN",
        "VITE_GLITCHTIP_ENVIRONMENT",
        "GLITCHTIP_DSN_NATIVE",
        "GLITCHTIP_ENVIRONMENT",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }

    for relative_path in ["../.env", "../../.env", "../../../.env"] {
        let env_path = manifest_dir.join(relative_path);
        if !env_path.exists() {
            continue;
        }

        println!("cargo:rerun-if-changed={}", env_path.display());

        if let Ok(iter) = dotenvy::from_path_iter(&env_path) {
            for item in iter.flatten() {
                let (key, value) = item;
                if matches!(
                    key.as_str(),
                    "VITE_GLITCHTIP_DSN"
                        | "VITE_GLITCHTIP_ENVIRONMENT"
                        | "GLITCHTIP_DSN_NATIVE"
                        | "GLITCHTIP_ENVIRONMENT"
                ) && env::var(&key).is_err()
                {
                    println!("cargo:rustc-env={key}={value}");
                }
            }

            break;
        }
    }

    tauri_build::build()
}
