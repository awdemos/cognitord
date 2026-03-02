# Cognitord

A Rust-based systemd daemon that processes requests through the DSRs (Distributed Semantic Reasoning System) and interfaces with Anthropic's LLM endpoints.

## Features

- **Systemd Socket Activation**: Runs as local socket-based service
- **DSRs Processing**: Integrates with the DSRs repository for semantic reasoning
- **Anthropic LLM Integration**: Interfaces with configured Anthropic endpoints
- **Unix Socket Protocol**: Uses `/run/cognitord/socket` for local communication
- **Docker Support**: Containerized deployment with multi-stage builds
- **Configuration Management**: Reads from `/etc/cognitord/config.json`
- **Socket-based Service**: Designed for local systemd service integration

## Quick Start

### Building

```bash
cargo build --release
```

### Installation

```bash
# Install systemd service and socket
sudo cp systemd/cognitord.service /etc/systemd/system/
sudo cp systemd/cognitord.socket /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now cognitord.socket

# Check status
sudo systemctl status cognitord.socket
```

### Usage

The service runs automatically via socket activation. Send JSON requests to the socket:

```bash
# Send request via Unix socket
echo '{"input": "Hello", "request_id": "test-001"}' | nc -U /run/cognitord/socket

# Interactive mode for testing
cognitord --config /etc/cognitord/config.json --interactive

# Validate configuration
cognitord --validate-config /etc/cognitord/config.json
```

### Example Sessions

#### Interactive Mode
```bash
$ cognitord --config config.json --interactive
> Hello, world!
Request ID: uuid-here
Timestamp: 2024-01-01T12:00:00Z
Output: Processed: Hello, world!

> What is 2+2?
Request ID: uuid-here
Timestamp: 2024-01-01T12:00:01Z
Output: Processed: What is 2+2?

> exit
Goodbye!
```

#### Daemon Mode with JSON Protocol
```bash
# Start daemon
$ cognitord --config config.json

# In another terminal, send requests via stdin:
$ echo '{"input": "Hello from stdin", "request_id": "test-123"}' | nc -U /run/cognitord/socket
{"output":"Processed: Hello from stdin","usage":{"input_tokens":3,"output_tokens":5,"total_tokens":8},"request_id":"test-123","timestamp":"2024-01-01T12:00:00Z","duration_ms":1}
```

#### With Context and System Prompt
```bash
echo '{"input": "Summarize", "context": "Long article text here...", "system_prompt": "Be concise"}' | nc -U /run/cognitord/socket
```

### Docker

The Docker container runs Cognitord in stdin/stdout mode for socket communication:

```bash
# Build image
docker build -t cognitord .

# Run as socket server (default mode)
docker run -d --name cognitord \
  -e ANTHROPIC_API_KEY=your-api-key \
  -v /run/cognitord:/run/cognitord \
  cognitord

# Interactive mode for testing
docker run -it --rm \
  -e ANTHROPIC_API_KEY=your-api-key \
  cognitord interactive

# Test mode
docker run --rm \
  -e ANTHROPIC_API_KEY=your-api-key \
  cognitord test

# Test socket communication
echo '{"input": "Hello from Docker!", "request_id": "docker-test-001"}' | \
  nc -U /run/cognitord/socket
```

**Note:** For production deployment with proper systemd socket activation, install the service files directly on the host system rather than using Docker containers.

## Configuration

The daemon reads configuration from `/etc/cognitord/config.json` (systemd service) or `~/.claude.settings.json` (CLI usage). The configuration file should contain:

```json
{
  "anthropic": {
    "api_key": "your-api-key",
    "base_url": "https://api.anthropic.com",
    "model": "claude-3-sonnet-20240229",
    "max_tokens": 1000,
    "temperature": 0.7,
    "timeout_seconds": 30
  },
  "daemon": {
    "log_level": "info",
    "timeout_seconds": 30
  }
}
```

## Protocol

The daemon uses a JSON line-based protocol over stdin/stdout:

### Request Format

```json
{
  "input": "Your request here",
  "request_id": "unique-identifier"
}
```

### Response Format

```json
{
  "request_id": "unique-identifier",
  "response": "Processed response",
  "timestamp": "2024-01-01T00:00:00Z",
  "processing_time_ms": 1234
}
```

## Systemd Service

Install the systemd service:

```bash
sudo cp systemd/cognitord.service /etc/systemd/system/
sudo systemctl enable cognitord
sudo systemctl start cognitord
```

## Development

### Project Structure

```
├── src/
│   └── main.rs              # Main daemon implementation
├── docker/                  # Docker-related files
├── systemd/                 # Systemd service files
├── Cargo.toml              # Rust project configuration
├── Dockerfile              # Docker build configuration
└── docker-compose.yml      # Docker Compose configuration
```

### Testing

```bash
# Run unit tests
cargo test

# Test Docker image
./docker/test-docker.sh

# Test with Docker Compose
docker-compose --profile test up
```

## Security

- Runs as non-root user in Docker containers
- Minimal attack surface with only required dependencies
- Configuration file permission validation
- Secure handling of API keys

## License

MIT License - see LICENSE file for details.