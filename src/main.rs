use clap::{Arg, Command};
use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request payload for processing input through the daemon
#[derive(Debug, Serialize, Deserialize)]
struct ProcessRequest {
    /// The input text to be processed
    input: String,
    /// Optional context to include in processing
    context: Option<String>,
    /// Optional system prompt to guide processing
    system_prompt: Option<String>,
    /// Optional request ID for tracking (auto-generated if not provided)
    request_id: Option<String>,
}

/// Response payload returned after processing
#[derive(Debug, Serialize, Deserialize)]
struct ProcessResponse {
    /// The processed output text
    output: String,
    /// Token usage statistics for this request
    usage: UsageStats,
    /// Unique identifier for this request
    request_id: String,
    /// ISO 8601 timestamp when response was generated
    timestamp: String,
    /// Processing duration in milliseconds
    duration_ms: u64,
}

/// Token usage statistics for a request
#[derive(Debug, Serialize, Deserialize)]
struct UsageStats {
    /// Number of tokens in the input
    input_tokens: u32,
    /// Number of tokens in the output
    output_tokens: u32,
    /// Total tokens used (input + output)
    total_tokens: u32,
}

/// Error response returned when processing fails
#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    /// Detailed error information
    error: ErrorDetails,
    /// Request ID if available
    request_id: Option<String>,
    /// ISO 8601 timestamp when error occurred
    timestamp: String,
}

/// Detailed error information
#[derive(Debug, Serialize, Deserialize)]
struct ErrorDetails {
    /// Error code for programmatic handling
    code: String,
    /// Human-readable error message
    message: String,
    /// Additional error context if available
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
    /// Maximum retry attempts for failed API calls (TODO: implement retry logic)
    max_retries: u32,
    /// Initial delay between retries in milliseconds (TODO: implement retry logic)
    retry_delay_ms: u64,
    /// Exponential backoff multiplier for retries (TODO: implement retry logic)
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
    dsrs: DsrsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DsrsConfig {
    enable_context: bool,
    enable_system_prompt: bool,
    max_context_length: usize,
    retry_attempts: u32,
}

/// Configuration subset needed for input processing
#[derive(Debug, Clone)]
struct ProcessingConfig {
    /// DSRs-specific settings
    dsrs: DsrsConfig,
    /// Model identifier for processing
    model: String,
    /// Maximum input size in bytes
    max_input_size: usize,
}

impl Config {
    /// Extract the processing-relevant configuration subset
    fn to_processing_config(&self) -> ProcessingConfig {
        ProcessingConfig {
            dsrs: self.dsrs.clone(),
            model: self.anthropic.model.clone(),
            max_input_size: self.daemon.max_input_size,
        }
    }
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
            .action(clap::ArgAction::SetTrue)
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

    if *matches.get_one::<bool>("interactive").unwrap_or(&false) {
        run_interactive(&config)
    } else {
        run_daemon(&config)
    }
}

fn validate_config(config_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Validating configuration: {config_file}");
    
    let config = load_config(config_file)?;
    
    // Validate API key format (sk- prefix + at least 20 alphanumeric chars)
    let api_key = &config.anthropic.api_key;
    if !api_key.starts_with("sk-") || api_key.len() < 23 {
        return Err("[E003] Invalid API key format: must start with 'sk-' and be at least 23 characters".into());
    }
    // Validate API key contains only alphanumeric after prefix
    if !api_key[3..].chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("[E003] Invalid API key format: must contain only alphanumeric characters after 'sk-'".into());
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

/// Helper function to write a JSON response to stdout
fn write_json_response<T: serde::Serialize>(stdout: &mut impl std::io::Write, response: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(response)?;
    writeln!(stdout, "{json}")?;
    stdout.flush()?;
    Ok(())
}

fn run_daemon(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Cognitord daemon in background mode...");
    
    // Simple stdin/stdout processing loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let stdin_handle = stdin.lock();
    
    for line in stdin_handle.lines() {
        let input = line?;
        
        match process_input(&input, &config.to_processing_config()) {
            Ok(response) => {
                write_json_response(&mut stdout, &response)?;
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
                write_json_response(&mut stdout, &error_response)?;
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
        
        if input.trim() == "exit" || input.trim() == "quit" {
            println!("Goodbye!");
            break Ok(());
        }
        
        match process_input(input.trim(), &config.to_processing_config()) {
            Ok(response) => {
                let json = serde_json::to_string_pretty(&response)?;
                println!("{json}");
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }
}

fn process_input(input: &str, config: &ProcessingConfig) -> Result<ProcessResponse, Box<dyn std::error::Error>> {
    let request: ProcessRequest = serde_json::from_str(input)?;
    
    // Validate input
    if request.input.trim().is_empty() {
        return Err("[E001] Input cannot be empty".into());
    }
    
    // Validate input size
    if request.input.len() > config.max_input_size {
        return Err(format!("[E002] Input exceeds maximum size of {} bytes", config.max_input_size).into());
    }
    let request: ProcessRequest = serde_json::from_str(input)?;
    
    // Validate input
    if request.input.trim().is_empty() {
        return Err("[E001] Input cannot be empty".into());
    }
    
    // Validate input size
    if request.input.len() > config.max_input_size {
        return Err(format!("[E002] Input exceeds maximum size of {} bytes", config.max_input_size).into());
    }
    // Generate request ID if not provided
    let request_id = request.request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Start processing timer
    let start_time = std::time::Instant::now();
    
    // Mock processing mode - returns formatted input for testing
    // TODO: Implement actual DSRs LLM integration
    let mut output = format!("Processed: {}", request.input);
    
    // Add context if provided and enabled
    if config.dsrs.enable_context && request.context.is_some() {
        if let Some(context) = &request.context {
            let truncated_context: String = context.chars()
                .take(config.dsrs.max_context_length)
                .collect();
            output.push_str(&format!("\nContext: {truncated_context}"));
        }
    }
    
    // Add system prompt if provided and enabled
    if config.dsrs.enable_system_prompt && request.system_prompt.is_some() {
        if let Some(system_prompt) = &request.system_prompt {
            output.push_str(&format!("\nSystem: {system_prompt}"));
        }
    }
    
    // Add model info
    output.push_str(&format!("\nModel: {}", config.model));
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    
    // Calculate usage statistics (estimated based on token count)
    let input_tokens = estimate_token_count(&request.input);
    let output_tokens = estimate_token_count(&output);
    
    Ok(ProcessResponse {
        output,
        usage: UsageStats {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        },
        request_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        duration_ms,
    })
}

// Helper function to estimate token count (rough approximation)
fn estimate_token_count(text: &str) -> u32 {
    // Simple heuristic: ~4 characters per token on average
    (text.len() as f64 / 4.0).ceil() as u32
}