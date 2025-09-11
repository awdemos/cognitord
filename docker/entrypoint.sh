#!/bin/bash
set -e

# Cognitord Docker Entrypoint

# Function to handle signals
cleanup() {
    echo "Received shutdown signal, stopping daemon..."
    if [ -n "$DAEMON_PID" ]; then
        kill $DAEMON_PID 2>/dev/null || true
    fi
    if [ -n "$SOCKET_PATH" ] && [ -S "$SOCKET_PATH" ]; then
        rm -f "$SOCKET_PATH"
    fi
    exit 0
}

# Register signal handlers
trap cleanup SIGTERM SIGINT

# Set default values
CONFIG_FILE=${CONFIG_FILE:-/app/config.json}
LOG_LEVEL=${LOG_LEVEL:-info}
API_KEY=${ANTHROPIC_API_KEY:-}
SOCKET_PATH=${SOCKET_PATH:-/run/cognitord/socket}

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
if ! /usr/local/bin/cognitord --validate-config "$CONFIG_FILE"; then
    echo "Error: Invalid configuration file"
    exit 1
fi

# Handle different run modes
case "$1" in
    "daemon")
        echo "Starting Cognitord daemon in background mode..."
        echo "Socket path: $SOCKET_PATH"
        
        # Clean up existing socket if it exists
        if [ -S "$SOCKET_PATH" ]; then
            rm -f "$SOCKET_PATH"
        fi
        
        # Create Unix socket and listen for connections
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE" --log-level "$LOG_LEVEL"
        ;;
    
    "interactive")
        echo "Starting Cognitord daemon in interactive mode..."
        echo "Send JSON requests via stdin, responses will appear on stdout"
        echo "Press Ctrl+C to exit"
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE" --log-level "$LOG_LEVEL" --interactive
        ;;
    
    "test")
        echo "Running Cognitord daemon test mode..."
        echo '{"input": "Hello, Docker!", "request_id": "docker-test-001"}' | \
        /usr/local/bin/cognitord --config "$CONFIG_FILE" --log-level "$LOG_LEVEL"
        ;;
    
    "validate")
        echo "Validating configuration..."
        exec /usr/local/bin/cognitord --validate-config "$CONFIG_FILE"
        ;;
    
    "socket")
        echo "Starting Cognitord as socket server..."
        echo "Socket path: $SOCKET_PATH"
        
        # Clean up existing socket if it exists
        if [ -S "$SOCKET_PATH" ]; then
            rm -f "$SOCKET_PATH"
        fi
        
        # Create socket directory if needed
        SOCKET_DIR=$(dirname "$SOCKET_PATH")
        mkdir -p "$SOCKET_DIR"
        
        # Start daemon
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE" --log-level "$LOG_LEVEL"
        ;;
    
    *)
        # Default to socket mode for Docker
        echo "Starting Cognitord in default socket mode..."
        echo "Socket path: $SOCKET_PATH"
        
        # Clean up existing socket if it exists
        if [ -S "$SOCKET_PATH" ]; then
            rm -f "$SOCKET_PATH"
        fi
        
        # Start daemon
        exec /usr/local/bin/cognitord --config "$CONFIG_FILE" --log-level "$LOG_LEVEL"
        ;;
esac