#!/bin/bash
set -e

echo "Starting Cognitord container..."

# Set default values
CONFIG_FILE=${CONFIG_FILE:-/tmp/config.json}
LOG_LEVEL=${LOG_LEVEL:-info}
API_KEY=${ANTHROPIC_API_KEY:-}

# Validate required environment variables
if [ -z "$API_KEY" ]; then
    echo "Error: ANTHROPIC_API_KEY environment variable is required"
    exit 1
fi

# Create configuration file if it doesn't exist
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating configuration file..."
    cat > "$CONFIG_FILE" << EOF
{
  "anthropic": {
    "api_key": "$API_KEY",
    "base_url": "https://api.anthropic.com",
    "model": "claude-3-sonnet-20240229",
    "max_tokens": 1000,
    "temperature": 0.7,
    "timeout_seconds": 30
  },
  "daemon": {
    "log_level": "$LOG_LEVEL",
    "timeout_seconds": 30,
    "max_input_size": 10000,
    "max_retries": 3,
    "retry_delay_ms": 1000,
    "backoff_factor": 2.0
  },
  "logging": {
    "level": "$LOG_LEVEL",
    "format": "json"
  },
  "dsrs": {
    "enable_context": true,
    "enable_system_prompt": true,
    "max_context_length": 5000,
    "retry_attempts": 3
  }
}
EOF
fi

# Handle different run modes
case "$1" in
    "test")
        echo "Running test..."
        echo '{"input": "Hello, Docker!", "request_id": "docker-test-001"}' | /usr/local/bin/cognitord --config "$CONFIG_FILE"
        ;;
    "daemon")
        echo "Starting daemon mode..."
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE"
        ;;
    "socket")
        echo "Starting socket mode..."
        # Create socket directory
        mkdir -p /run/cognitord
        # Start daemon (stdin/stdout mode for container)
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE"
        ;;
    "interactive")
        echo "Starting interactive mode..."
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE" --interactive
        ;;
    *)
        echo "Starting default daemon mode..."
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE"
        ;;
esac