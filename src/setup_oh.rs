//! Setup wizard for Open Horizons integration
//!
//! Provides an interactive setup flow that:
//! 1. Opens browser to get API key
//! 2. Prompts user to paste key
//! 3. Creates ~/.config/openhorizons/config.json

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

/// Get the global OH config path
pub fn global_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("openhorizons")
        .join("config.json")
}

/// Open a URL in the default browser
fn open_browser(url: &str) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/c", "start", url]).spawn()?;
    }
    Ok(())
}

/// Read a line from stdin
fn read_line() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Verify the API key works by making a test request
fn verify_api_key(api_url: &str, api_key: &str) -> Result<bool, String> {
    let url = format!("{}/api/contexts", api_url);

    let response = attohttpc::get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    Ok(response.is_success())
}

/// Run the interactive setup wizard
pub fn run() -> Result<(), String> {
    let config_path = global_config_path();
    let api_url = "https://app.openhorizons.me";
    let api_keys_url = format!("{}/settings/api-keys", api_url);

    println!("Open Horizons Setup");
    println!("===================\n");

    // Check for existing config
    if config_path.exists() {
        println!("Existing configuration found at:");
        println!("  {}\n", config_path.display());
        print!("Overwrite? [y/N]: ");
        io::stdout().flush().map_err(|e| e.to_string())?;

        let answer = read_line().map_err(|e| e.to_string())?;
        if !answer.to_lowercase().starts_with('y') {
            println!("Setup cancelled.");
            return Ok(());
        }
        println!();
    }

    println!("Step 1: Get your API key");
    println!("------------------------");
    println!("Opening {} in your browser...\n", api_keys_url);

    if let Err(e) = open_browser(&api_keys_url) {
        println!("Could not open browser automatically: {}", e);
        println!("Please open this URL manually:");
        println!("  {}\n", api_keys_url);
    }

    println!("1. Sign in to your Open Horizons account");
    println!("2. Create a new API key (or copy an existing one)");
    println!("3. Paste the key below\n");

    print!("API Key: ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let api_key = read_line().map_err(|e| e.to_string())?;

    if api_key.is_empty() {
        return Err("No API key provided. Setup cancelled.".to_string());
    }

    // Verify the key
    println!("\nVerifying API key...");
    match verify_api_key(api_url, &api_key) {
        Ok(true) => println!("API key verified successfully!\n"),
        Ok(false) => {
            return Err(
                "API key verification failed. Please check your key and try again.".to_string(),
            );
        }
        Err(e) => {
            println!("Warning: Could not verify API key: {}", e);
            println!("Proceeding anyway...\n");
        }
    }

    // Create config directory
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Write config file
    let config = serde_json::json!({
        "api_key": api_key,
        "api_url": api_url
    });

    let config_str = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, &config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    println!("Step 2: Configuration saved");
    println!("---------------------------");
    println!("Created: {}\n", config_path.display());

    println!("Setup complete!\n");
    println!("The OH MCP server will now use this configuration.");
    println!("Superego will also use it for OH integration.\n");
    println!("Next steps:");
    println!("  1. Install OH MCP: npm install -g @cloud-atlas-ai/oh-mcp-server");
    println!("  2. Add to Claude Code settings (~/.claude/settings.json):");
    println!(r#"     {{"mcpServers": {{"oh-mcp": {{"command": "oh-mcp"}}}}}}"#);
    println!("  3. Restart Claude Code\n");

    Ok(())
}
