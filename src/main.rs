use clap::{Arg, Command};
use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct ProcessRequest {
    input: String,
    context: Option<String>,
    system_prompt: Option<String>,
    request_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessResponse {
    output: String,
    usage: UsageStats,
    request_id: String,
    timestamp: String,
    duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct UsageStats {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: ErrorDetails,
    request_id: Option<String>,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorDetails {
    code: String,
    message: String,
    details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicConfig {
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
    temperature: f64,
    timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DaemonConfig {
    log_level: String,
    timeout_seconds: u64,
    max_input_size: usize,
    max_retries: u32,
    retry_delay_ms: u64,
    backoff_factor: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoggingConfig {
    level: String,
    format: String,
    file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    anthropic: AnthropicConfig,
    daemon: DaemonConfig,
    logging: LoggingConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("cognitord")
        .version("0.1.0")
        .about("Cognitord - DSRs LLM Processing Daemon")
        .arg(Arg::new("config")
            .long("config")
            .value_name("FILE")
            .help("Configuration file path")
            .default_value("/etc/cognitord/config.json"))
        .arg(Arg::new("log-level")
            .long("log-level")
            .value_name("LEVEL")
            .help("Log level")
            .default_value("info"))
        .arg(Arg::new("validate-config")
            .long("validate-config")
            .value_name("FILE")
            .help("Validate configuration file"))
        .arg(Arg::new("interactive")
            .long("interactive")
            .help("Run in interactive mode"))
        .get_matches();

    // Handle validation mode
    if let Some(config_file) = matches.get_one::<String>("validate-config") {
        return validate_config(config_file);
    }

    // Load configuration
    let config_file = matches.get_one::<String>("config").unwrap();
    let config = load_config(config_file)?;

    // Initialize logging
    init_logging(&config.logging);

    if matches.get_flag("interactive") {
        run_interactive(&config)
    } else {
        run_daemon(&config)
    }
}

fn validate_config(config_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Validating configuration: {}", config_file);
    
    let config = load_config(config_file)?;
    
    // Validate API key format
    if !config.anthropic.api_key.starts_with("sk-") {
        return Err("Invalid API key format".into());
    }
    
    // Validate URL
    if !config.anthropic.base_url.starts_with("http") {
        return Err("Invalid base URL".into());
    }
    
    // Validate timeout
    if config.anthropic.timeout_seconds == 0 {
        return Err("Timeout must be greater than 0".into());
    }
    
    println!("✓ Configuration is valid");
    Ok(())
}

fn load_config(config_file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(config_file)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

fn init_logging(logging: &LoggingConfig) {
    use tracing_subscriber::{fmt, EnvFilter};
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&logging.level));
    
    fmt()
        .with_env_filter(filter)
        .init();
}

fn run_daemon(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Cognitord daemon in background mode...");
    
    // Simple stdin/stdout processing loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    for line in stdin.lock().lines() {
        let input = line?;
        
        match process_input(&input, config) {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", json)?;
                stdout.flush()?;
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: ErrorDetails {
                        code: "INTERNAL_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                    },
                    request_id: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                let json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", json)?;
                stdout.flush()?;
            }
        }
    }
    
    Ok(())
}

fn run_interactive(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Cognitord Interactive Mode");
    println!("Enter JSON requests (Ctrl+D to exit):");
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    loop {
        print!("> ");
        stdout.flush()?;
        
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        
        if input.trim().is_empty() {
            continue;
        }
        
        match process_input(&input.trim(), config) {
            Ok(response) => {
                let json = serde_json::to_string_pretty(&response)?;
                println!("{}", json);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn process_input(input: &str, config: &Config) -> Result<ProcessResponse, Box<dyn std::error::Error>> {
    let request: ProcessRequest = serde_json::from_str(input)?;
    
    // Validate input
    if request.input.trim().is_empty() {
        return Err("Input cannot be empty".into());
    }
    
    // Generate request ID if not provided
    let request_id = request.request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Simulate processing (in real implementation, this would call DSRs and Anthropic API)
    let start_time = std::time::Instant::now();
    
    // Mock response - in real implementation, this would call the actual LLM
    let output = format!("Processed: {}", request.input);
    let usage = UsageStats {
        input_tokens: request.input.len() as u32,
        output_tokens: output.len() as u32,
        total_tokens: (request.input.len() + output.len()) as u32,
    };
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    
    Ok(ProcessResponse {
        output,
        usage,
        request_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        duration_ms,
    })
}