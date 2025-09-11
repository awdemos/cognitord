#!/bin/bash

# Test script to verify cognitord functionality
set -e

echo "=== Testing Cognitord Docker Container ==="

# Test 1: Basic functionality
echo "1. Testing basic functionality..."
timeout 5s bash -c "echo '{\"input\": \"Hello test\", \"request_id\": \"test-001\"}' | docker run --rm -i -e ANTHROPIC_API_KEY=test-key cognitord:latest daemon" || echo "Timeout reached"
echo ""

# Test 2: Interactive mode
echo "2. Testing interactive mode..."
docker run --rm -e ANTHROPIC_API_KEY=test-key cognitord:latest interactive << 'EOF'
{"input": "Hello interactive", "request_id": "test-002"}
EOF

echo "3. Testing test mode..."
docker run --rm -e ANTHROPIC_API_KEY=test-key cognitord:latest test

echo "=== All tests completed ==="