# DSRs Daemon Docker Setup

This directory contains Docker configuration for the DSRs systemd daemon.

## Files

- `Dockerfile` - Multi-stage Docker build
- `docker-compose.yml` - Docker Compose configuration
- `.env.example` - Environment variables template
- `entrypoint.sh` - Container entrypoint script
- `example-config.json` - Example configuration file
- `test-docker.sh` - Comprehensive test suite
- `ci-cd.sh` - CI/CD pipeline script

## Quick Start

1. **Build the image**:
```bash
docker build -t dsrs-daemon:latest .
```

2. **Run tests**:
```bash
./docker/test-docker.sh
```

3. **Run with Docker Compose**:
```bash
cp .env.example .env
# Edit .env with your API key
docker-compose --profile daemon up -d
```

## Usage Examples

### Interactive Mode
```bash
docker-compose --profile interactive up
```

### Run Tests
```bash
docker-compose --profile test up
```

### Validate Configuration
```bash
docker-compose --profile validate up
```

## Security Features

- Multi-stage build for minimal attack surface
- Non-root user execution
- Read-only configuration volumes
- Health checks
- Resource limits

## CI/CD

The included GitHub Actions workflow provides:
- Automated testing
- Security scanning
- Multi-platform builds
- Image signing
- SBOM generation
- Deployment automation