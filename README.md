# Cognitord

A Rust-based systemd daemon that processes requests through the DSRs (Distributed Semantic Reasoning System) and interfaces with Anthropic's LLM endpoints.

## Features

- **Systemd Integration**: Runs as a background service
- **DSRs Processing**: Integrates with the DSRs repository for semantic reasoning
- **Anthropic LLM Integration**: Interfaces with configured Anthropic endpoints
- **JSON Line Protocol**: Uses stdin/stdout for request/response handling
- **Docker Support**: Containerized deployment with multi-stage builds
- **Configuration Management**: Reads from `~/.claude.settings.json`
- **Multiple Modes**: Supports both daemon and interactive modes

## Quick Start

### Building

```bash
cargo build --release
```

### Running

```bash
# Daemon mode (default)
cognitord --config ~/.claude.settings.json

# Interactive mode
cognitord --config ~/.claude.settings.json --interactive

# Validate configuration
cognitord --validate-config ~/.claude.settings.json
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