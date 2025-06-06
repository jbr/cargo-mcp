#!/bin/bash

# Simple test script for the Cargo MCP server
# Usage: ./test-mcp.sh

echo "Testing Cargo MCP Server..."

# Build the server first
echo "Building server..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"

# Test the MCP protocol
echo "Testing MCP protocol..."

# Start the server and send test requests
{
    echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}'
    echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}'
} | ./target/release/cargo-mcp serve > test_output.json 2>&1

if [ $? -eq 0 ]; then
    echo "✅ MCP protocol test passed"
    echo "Output saved to test_output.json"
else
    echo "❌ MCP protocol test failed"
    cat test_output.json
    exit 1
fi

# Test with this project itself
echo "Testing cargo_check on self..."
{
    echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}'
    echo '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "cargo_check", "arguments": {"path": "."}}}'
} | ./target/release/cargo-mcp serve > self_test_output.json 2>&1

echo "✅ Self-test completed - check self_test_output.json for results"

echo "All tests completed!"
