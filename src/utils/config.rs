use std::fs;
use std::path::PathBuf;
use datex_core::decompiler::{decompile_value, DecompileOptions};
use datex_core::network::com_interfaces::default_com_interfaces::websocket::websocket_common::WebSocketClientInterfaceSetupData;
use datex_core::runtime::{Runtime, RuntimeConfig};
use datex_core::values::core_values::endpoint::Endpoint;
use datex_core::values::serde::deserializer::DatexDeserializer;
use datex_core::values::serde::error::SerializationError;
use datex_core::values::serde::serializer::to_value_container;
use serde::Deserialize;

pub fn read_config_file(path: PathBuf) -> Result<RuntimeConfig, SerializationError> {
    let deserializer = DatexDeserializer::from_dx_file(path)?;
    let config: RuntimeConfig = Deserialize::deserialize(deserializer)?;
    Ok(config)
}

fn get_dx_files(base_path: PathBuf) -> Result<Vec<PathBuf>, SerializationError> {
    let mut config_dir = base_path.clone();
    config_dir.push(".datex");

    // Create the directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| {
            SerializationError(e.to_string())
        })?;
    }

    // Collect all files ending with `.dx`
    let dx_files = fs::read_dir(&config_dir)
        .map_err(|e| SerializationError(e.to_string()))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("dx") {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect();

    Ok(dx_files)
}

pub fn create_new_config_file(base_path: PathBuf, endpoint: Endpoint) -> Result<PathBuf, SerializationError> {
    let mut config = RuntimeConfig::new_with_endpoint(endpoint.clone());

    // add default interface
    config.add_interface("websocket-client".to_string(), WebSocketClientInterfaceSetupData {
        address: "wss://example.unyt.land".to_string(),
    })?;

    let mut config_path = base_path.clone();
    config_path.push(".datex");
    config_path.push(format!("{}.dx", endpoint));
    let config = to_value_container(&config)?;
    let datex_script = decompile_value(&config, DecompileOptions {formatted: true, ..DecompileOptions::default()});
    fs::write(config_path.clone(), datex_script).map_err(|e| {
        SerializationError(e.to_string())
    })?;

    println!("Created new config file for {endpoint} at {config_path:?}");

    Ok(config_path)
}

pub fn get_config(custom_config_path: Option<PathBuf>) -> Result<RuntimeConfig, SerializationError> {
    Ok(match custom_config_path {
        Some(path) => read_config_file(path)?,
        None => {
            match home::home_dir() {
                Some(path) if !path.as_os_str().is_empty() => {
                    // get all .dx files in the home directory .datex folder
                    let dx_files = get_dx_files(path.clone())?;
                    // if no files yet, create a new config file for a random endpoint
                    if dx_files.is_empty() {
                        let endpoint = Endpoint::random();
                        let config_path = create_new_config_file(path.clone(), endpoint)?;
                        read_config_file(config_path)?
                    }
                    else {
                        // if there are files, read the first one
                        let config_path = dx_files.first().unwrap().clone();
                        read_config_file(config_path)?
                    }
                },
                _ => {
                    eprintln!("Unable to get home directory, using temporary endpoint.");
                    RuntimeConfig::new_with_endpoint(Endpoint::random())
                }
            }
        },
    })
}

pub async fn create_runtime_with_config(custom_config_path: Option<PathBuf>, force_debug: bool) -> Result<Runtime, SerializationError> {
    let mut config = get_config(custom_config_path)?;
    // overwrite debug mode if force_debug is true
    if force_debug {
        config.debug = Some(true);
    }
    let runtime = Runtime::create_native(config).await;

    let cli_version = env!("CARGO_PKG_VERSION");

    println!("================================================");
    println!("DATEX REPL v{cli_version}");
    println!("DATEX Core version: {}", runtime.version);
    println!("Endpoint: {}", runtime.endpoint());
    println!("\nexit using [CTRL + C]");
    println!("================================================\n");

    Ok(runtime)
}