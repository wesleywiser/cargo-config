extern crate toml;
extern crate serde_json;

use std::fs;
use std::io;
use std::env;
use std::path::PathBuf;


fn run(args: &[&str]) -> Result<String, io::Error> {
    let args_len = args.len();

    if args_len == 0 {
        return Err(io::Error::from(io::ErrorKind::NotFound));
    }

    let mut cmd = std::process::Command::new(&args[0]);

    if args_len > 1 {
        cmd.args(&args[1..]);
    }

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(io::Error::new(io::ErrorKind::Other, stderr))
    }
}

fn query_manifest_path() -> Result<String, Box<dyn std::error::Error>> {
    let output = run(&["cargo", "locate-project"])?;

    let value = serde_json::from_str::<serde_json::Value>(&output)?;
    
    match value {
        serde_json::Value::Object(obj) => {
            match obj.get("root") {
                Some(val) => {
                    match val {
                        serde_json::Value::String(s) => Ok(s.clone()),
                        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid manifest json input").into()),
                    }
                },
                None => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid manifest json input").into()),
            }
        },
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid manifest json input").into()),
    }
}

fn table_query<'a>(value: &'a toml::Value, query: &str) -> Option<&'a toml::Value> {
    match value {
        &toml::Value::Table(ref table) => table.get(query),
        _ => None,
    }
}

fn lookup<'a>(mut value: &'a toml::Value, keys: &str) -> Option<&'a toml::Value> {
    let mut founded = false;
    for key in keys.split(".") {
        match table_query(&value, key) {
            Some(val) => {
                value = val;
                if !founded {
                    founded = true;
                }
            },
            None => break,
        }
    }

    if founded {
        Some(value)
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    args.next();
    args.next();

    let mut toml_paths = vec![ PathBuf::from(query_manifest_path()?) ];

    if let Some(project_dir) = &toml_paths[0].parent() {
        let p1 = project_dir.join(".cargo/config");
        let p2 = project_dir.join(".cargo/config.toml");

        if p1.exists() && p1.is_file() {
            toml_paths.push(p1);
        } else if p2.exists() && p2.is_file() {
            toml_paths.push(p2);
        }

        if let Ok(cargo_home_dir) = env::var("CARGO_HOME") {
            let cargo_home_path = PathBuf::from(cargo_home_dir);
            let p1 = cargo_home_path.join("config");
            let p2 = cargo_home_path.join("config.toml");

            if p1.exists() && p1.is_file() {
                toml_paths.push(p1);
            } else if p2.exists() && p2.is_file() {
                toml_paths.push(p2);
            }
        }
    }

    if let Some(args) = args.next() {
        println!("KEYS: {:?}", args);
        for toml_path in toml_paths.iter() {
            let content = fs::read_to_string(toml_path)?;
            let toml_value = toml::from_str::<toml::Value>(&content)?;

            if let Some(val) = lookup(&toml_value, &args) {
                println!("TOML_FILE: {:?}", toml_path);
                println!("TOML_VALUE:\n{}", val);
                break;
            }
        }
    }

    Ok(())
}
