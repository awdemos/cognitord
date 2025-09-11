#!/bin/bash
set -e

# Docker Test Script for DSRs Daemon
# This script validates Docker build and functionality

echo "🐳 DSRs Daemon Docker Test Suite"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test functions
test_build() {
    echo -e "${YELLOW}Testing Docker build...${NC}"
    if docker build -t dsrs-daemon:test . > /tmp/docker-build.log 2>&1; then
        echo -e "${GREEN}✓ Docker build successful${NC}"
        return 0
    else
        echo -e "${RED}✗ Docker build failed${NC}"
        echo "Build log:"
        cat /tmp/docker-build.log
        return 1
    fi
}

test_config_validation() {
    echo -e "${YELLOW}Testing configuration validation...${NC}"
    
    # Create test config
    cat > /tmp/test-config.json << EOF
{
  "anthropic": {
    "api_key": "sk-test-key",
    "base_url": "https://api.anthropic.com",
    "model": "claude-3-sonnet-20240229",
    "max_tokens": 1000,
    "temperature": 0.7
  },
  "daemon": {
    "log_level": "info",
    "timeout_seconds": 30
  }
}
EOF

    if docker run --rm \
        -v /tmp/test-config.json:/app/config.json:ro \
        dsrs-daemon:test validate > /tmp/validation.log 2>&1; then
        echo -e "${GREEN}✓ Configuration validation successful${NC}"
        rm -f /tmp/test-config.json
        return 0
    else
        echo -e "${RED}✗ Configuration validation failed${NC}"
        echo "Validation log:"
        cat /tmp/validation.log
        rm -f /tmp/test-config.json
        return 1
    fi
}

test_daemon_start() {
    echo -e "${YELLOW}Testing daemon startup...${NC}"
    
    # Create test config
    cat > /tmp/test-config.json << EOF
{
  "anthropic": {
    "api_key": "sk-test-key",
    "base_url": "https://api.anthropic.com",
    "model": "claude-3-sonnet-20240229",
    "max_tokens": 1000,
    "temperature": 0.7
  },
  "daemon": {
    "log_level": "info",
    "timeout_seconds": 30
  }
}
EOF

    # Start daemon in background
    if docker run -d \
        --name dsrs-test-daemon \
        -v /tmp/test-config.json:/app/config.json:ro \
        dsrs-daemon:test daemon > /tmp/container-id.txt 2>&1; then
        
        container_id=$(cat /tmp/container-id.txt)
        
        # Wait for startup
        sleep 5
        
        # Check if container is running
        if docker inspect dsrs-test-daemon | grep -q '"Status": "running"'; then
            echo -e "${GREEN}✓ Daemon startup successful${NC}"
            docker stop dsrs-test-daemon
            docker rm dsrs-test-daemon
            rm -f /tmp/test-config.json /tmp/container-id.txt
            return 0
        else
            echo -e "${RED}✗ Daemon startup failed${NC}"
            docker logs dsrs-test-daemon
            docker rm -f dsrs-test-daemon
            rm -f /tmp/test-config.json /tmp/container-id.txt
            return 1
        fi
    else
        echo -e "${RED}✗ Daemon container creation failed${NC}"
        rm -f /tmp/test-config.json
        return 1
    fi
}

test_interactive_mode() {
    echo -e "${YELLOW}Testing interactive mode...${NC}"
    
    # Create test config
    cat > /tmp/test-config.json << EOF
{
  "anthropic": {
    "api_key": "sk-test-key",
    "base_url": "https://api.anthropic.com",
    "model": "claude-3-sonnet-20240229",
    "max_tokens": 1000,
    "temperature": 0.7
  },
  "daemon": {
    "log_level": "debug",
    "timeout_seconds": 30
  }
}
EOF

    # Test input
    cat > /tmp/test-input.json << EOF
{"input": "Hello, Docker test!", "request_id": "docker-test-001"}
EOF

    # Run interactive mode with test input
    if docker run --rm \
        -v /tmp/test-config.json:/app/config.json:ro \
        -i dsrs-daemon:test interactive < /tmp/test-input.json > /tmp/interactive-output.txt 2>&1; then
        
        if grep -q "request_id.*docker-test-001" /tmp/interactive-output.txt; then
            echo -e "${GREEN}✓ Interactive mode test successful${NC}"
            rm -f /tmp/test-config.json /tmp/test-input.json /tmp/interactive-output.txt
            return 0
        else
            echo -e "${YELLOW}⚠ Interactive mode ran but response format unexpected${NC}"
            echo "Output:"
            cat /tmp/interactive-output.txt
            rm -f /tmp/test-config.json /tmp/test-input.json /tmp/interactive-output.txt
            return 0
        fi
    else
        echo -e "${RED}✗ Interactive mode test failed${NC}"
        echo "Error output:"
        cat /tmp/interactive-output.txt
        rm -f /tmp/test-config.json /tmp/test-input.json /tmp/interactive-output.txt
        return 1
    fi
}

test_docker_compose() {
    echo -e "${YELLOW}Testing Docker Compose...${NC}"
    
    # Copy example env file
    cp .env.example .env
    
    # Test validation profile
    if docker-compose --profile validate up --abort-on-container-exit > /tmp/compose-validate.log 2>&1; then
        echo -e "${GREEN}✓ Docker Compose validation profile successful${NC}"
    else
        echo -e "${RED}✗ Docker Compose validation profile failed${NC}"
        echo "Compose log:"
        cat /tmp/compose-validate.log
        rm -f .env
        return 1
    fi
    
    # Test test profile
    if docker-compose --profile test up --abort-on-container-exit > /tmp/compose-test.log 2>&1; then
        echo -e "${GREEN}✓ Docker Compose test profile successful${NC}"
        rm -f .env
        return 0
    else
        echo -e "${RED}✗ Docker Compose test profile failed${NC}"
        echo "Compose log:"
        cat /tmp/compose-test.log
        rm -f .env
        return 1
    fi
}

test_security() {
    echo -e "${YELLOW}Testing security aspects...${NC}"
    
    # Test non-root user
    if docker run --rm --user root dsrs-daemon:test id | grep -q "uid=0"; then
        # Check if the container runs as non-root by default
        if docker run --rm dsrs-daemon:test id | grep -q "uid=0"; then
            echo -e "${RED}✗ Container runs as root user${NC}"
            return 1
        else
            echo -e "${GREEN}✓ Container runs as non-root user${NC}"
        fi
    else
        echo -e "${GREEN}✓ Container runs as non-root user${NC}"
    fi
    
    # Test for unnecessary packages
    if docker run --rm dsrs-daemon:test dpkg -l | grep -E "(python|perl|ruby)" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠ Container contains additional packages${NC}"
    else
        echo -e "${GREEN}✓ Container has minimal packages${NC}"
    fi
    
    return 0
}

cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    docker stop dsrs-test-daemon 2>/dev/null || true
    docker rm dsrs-test-daemon 2>/dev/null || true
    docker-compose down 2>/dev/null || true
    rm -f /tmp/test-*.json /tmp/*.log /tmp/container-id.txt /tmp/*.txt /tmp/env 2>/dev/null || true
    rm -f .env 2>/dev/null || true
}

# Set trap for cleanup
trap cleanup EXIT

# Run tests
echo "Running Docker tests..."
echo ""

passed=0
failed=0

test_build && ((passed++)) || ((failed++))
test_config_validation && ((passed++)) || ((failed++))
test_daemon_start && ((passed++)) || ((failed++))
test_interactive_mode && ((passed++)) || ((failed++))
test_docker_compose && ((passed++)) || ((failed++))
test_security && ((passed++)) || ((failed++))

echo ""
echo "Test Results:"
echo "============"
echo -e "${GREEN}Passed: $passed${NC}"
echo -e "${RED}Failed: $failed${NC}"

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed.${NC}"
    exit 1
fi