#!/bin/bash
set -e

# DSRs Daemon Docker Entrypoint

# Function to handle signals
cleanup() {
    echo "Received shutdown signal, stopping daemon..."
    if [ -n "$DAEMON_PID" ]; then
        kill $DAEMON_PID 2>/dev/null || true
    fi
    exit 0
}

# Register signal handlers
trap cleanup SIGTERM SIGINT

# Set default values
CONFIG_FILE=${CONFIG_FILE:-/app/config.json}
LOG_LEVEL=${LOG_LEVEL:-info}
API_KEY=${ANTHROPIC_API_KEY:-}

# Validate required environment variables
if [ -z "$API_KEY" ]; then
    echo "Error: ANTHROPIC_API_KEY environment variable is required"
    exit 1
fi

# Create configuration file if it doesn't exist
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating configuration file from template..."
    # Use a temporary file first
    sed "s|YOUR_API_KEY_HERE|$API_KEY|g" /app/config.json.example > /tmp/config.json.tmp
    # Try to copy to target location
    if cp /tmp/config.json.tmp "$CONFIG_FILE" 2>/dev/null; then
        echo "Configuration created at $CONFIG_FILE"
    else
        echo "Using temporary configuration file"
        CONFIG_FILE="/tmp/config.json.tmp"
    fi
fi

# Validate configuration file
if ! /usr/local/bin/dsrs-daemon --validate-config "$CONFIG_FILE"; then
    echo "Error: Invalid configuration file"
    exit 1
fi

# Handle different run modes
case "$1" in
    "daemon")
        echo "Starting DSRs daemon in background mode..."
        exec /usr/local/bin/dsrs-daemon --config "$CONFIG_FILE" --log-level "$LOG_LEVEL"
        ;;
    
    "interactive")
        echo "Starting DSRs daemon in interactive mode..."
        echo "Send JSON requests via stdin, responses will appear on stdout"
        echo "Press Ctrl+C to exit"
        exec /usr/local/bin/dsrs-daemon --config "$CONFIG_FILE" --log-level "$LOG_LEVEL" --interactive
        ;;
    
    "test")
        echo "Running DSRs daemon test mode..."
        echo '{"input": "Hello, Docker!", "request_id": "docker-test-001"}' | \
        /usr/local/bin/dsrs-daemon --config "$CONFIG_FILE" --log-level "$LOG_LEVEL"
        ;;
    
    "validate")
        echo "Validating configuration..."
        exec /usr/local/bin/dsrs-daemon --validate-config "$CONFIG_FILE"
        ;;
    
    *)
        # Default to daemon mode
        echo "Starting DSRs daemon in default mode..."
        exec /usr/local/bin/dsrs-daemon --config "$CONFIG_FILE" --log-level "$LOG_LEVEL"
        ;;
esac