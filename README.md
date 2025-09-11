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

### Docker

```bash
# Build image
docker build -t cognitord .

# Run in daemon mode
docker run -d --name cognitord cognitord

# Interactive mode
docker run -it cognitord interactive

# Test mode
docker run cognitord test
```

## Configuration

The daemon reads configuration from `~/.claude.settings.json` which should contain:

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