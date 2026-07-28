#!/bin/bash

# Warden Demo Server Test Script
# Run this after starting the server with: PORT=8080 ./target/release/warden-demo-server

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
PASS=0
FAIL=0

test_endpoint() {
    local name="$1"
    local expected="$2"
    local actual="$3"
    
    if [ "$expected" = "$actual" ]; then
        echo "✓ $name: PASS"
        ((PASS++))
    else
        echo "✗ $name: FAIL (expected: $expected, got: $actual)"
        ((FAIL++))
    fi
}

echo "=== Warden Demo Server Tests ==="
echo "Base URL: $BASE_URL"
echo ""

# Test 1: Health Check
echo "Testing /health endpoint..."
response=$(curl -s "$BASE_URL/health")
expected='{"status":"healthy","warden":"ready","mode":"demo"}'
test_endpoint "Health Check" "$expected" "$response"

# Test 2: Version
echo "Testing /version endpoint..."
response=$(curl -s "$BASE_URL/version")
if echo "$response" | grep -q '"service":"warden-demo-server"'; then
    echo "✓ Version Check: PASS"
    ((PASS++))
else
    echo "✗ Version Check: FAIL"
    ((FAIL++))
fi

# Test 3: Stats
echo "Testing /stats endpoint..."
response=$(curl -s "$BASE_URL/stats")
if echo "$response" | grep -q '"approved"'; then
    echo "✓ Stats Check: PASS"
    ((PASS++))
else
    echo "✗ Stats Check: FAIL"
    ((FAIL++))
fi

# Test 4: Evaluate - Safe Command
echo "Testing /evaluate endpoint (safe command)..."
response=$(curl -s -X POST "$BASE_URL/evaluate" \
    -H "Content-Type: application/json" \
    -d '{"identity":"demo-agent","command":"echo hello","capability":"execute","priority":1.0,"reward":1.0,"risk":0.1}')
if echo "$response" | grep -q '"decision":"APPROVED"'; then
    echo "✓ Evaluate Safe: PASS"
    ((PASS++))
else
    echo "✗ Evaluate Safe: FAIL"
    ((FAIL++))
fi

# Test 5: Evaluate - Shell Injection
echo "Testing /evaluate endpoint (shell injection)..."
response=$(curl -s -X POST "$BASE_URL/evaluate" \
    -H "Content-Type: application/json" \
    -d '{"identity":"demo-agent","command":"echo hello; cat /etc/passwd","capability":"execute","priority":1.0,"reward":1.0,"risk":0.1}')
if echo "$response" | grep -q '"decision":"BLOCKED"' && echo "$response" | grep -q '"SHELL_INJECTION"'; then
    echo "✓ Evaluate Shell Injection: PASS"
    ((PASS++))
else
    echo "✗ Evaluate Shell Injection: FAIL"
    ((FAIL++))
fi

# Test 6: Evaluate - Unknown Identity
echo "Testing /evaluate endpoint (unknown identity)..."
response=$(curl -s -X POST "$BASE_URL/evaluate" \
    -H "Content-Type: application/json" \
    -d '{"identity":"attacker","command":"ls","capability":"execute","priority":1.0,"reward":1.0,"risk":0.1}')
if echo "$response" | grep -q '"decision":"BLOCKED"' && echo "$response" | grep -q '"UNKNOWN_IDENTITY"'; then
    echo "✓ Evaluate Unknown Identity: PASS"
    ((PASS++))
else
    echo "✗ Evaluate Unknown Identity: FAIL"
    ((FAIL++))
fi

# Test 7: Challenge - Command Injection
echo "Testing /challenge endpoint..."
response=$(curl -s -X POST "$BASE_URL/challenge" \
    -H "Content-Type: application/json" \
    -d '{"challenge_id":"command-injection-01"}')
if echo "$response" | grep -q '"decision":"BLOCKED"'; then
    echo "✓ Challenge Command Injection: PASS"
    ((PASS++))
else
    echo "✗ Challenge Command Injection: FAIL"
    ((FAIL++))
fi

# Test 8: Stats after requests
echo "Testing /stats endpoint (after requests)..."
response=$(curl -s "$BASE_URL/stats")
if echo "$response" | grep -q '"approved":[1-9]'; then
    echo "✓ Stats Updated: PASS"
    ((PASS++))
else
    echo "✗ Stats Updated: FAIL"
    ((FAIL++))
fi

# Test 9: CORS Headers
echo "Testing CORS headers..."
response=$(curl -s -o /dev/null -w "%{http_code}" -X OPTIONS "$BASE_URL/evaluate" \
    -H "Origin: http://localhost:3000" \
    -H "Access-Control-Request-Method: POST")
if [ "$response" = "200" ]; then
    echo "✓ CORS Preflight: PASS"
    ((PASS++))
else
    echo "✗ CORS Preflight: FAIL"
    ((FAIL++))
fi

echo ""
echo "=== Test Summary ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "All tests passed!"
    exit 0
else
    echo "Some tests failed."
    exit 1
fi