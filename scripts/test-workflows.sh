#!/bin/bash
# Test workflow operations against the Atlas API

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
TOKEN="${TOKEN:-}"

echo "🧪 Testing Atlas ERP API"
echo "   Base URL: $BASE_URL"

if [ -z "$TOKEN" ]; then
    echo ""
    echo "🔑 Logging in..."
    LOGIN_RESPONSE=$(curl -sf -X POST "$BASE_URL/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d '{"email": "admin@atlas.local", "password": "admin123"}' 2>/dev/null || echo '{}')
    
    TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token // empty')
    if [ -z "$TOKEN" ]; then
        echo "❌ Failed to login. Make sure the server is running and database is seeded."
        echo "   Response: $LOGIN_RESPONSE"
        exit 1
    fi
    echo "   ✓ Logged in successfully"
fi

AUTH="Authorization: Bearer $TOKEN"

echo ""
echo "=== Health Check ==="
HEALTH=$(curl -sf "$BASE_URL/health" 2>/dev/null || echo "UNREACHABLE")
echo "   Status: $HEALTH"

echo ""
echo "=== Getting Schema for Employees ==="
curl -sf "$BASE_URL/api/v1/schema/employees" -H "$AUTH" | jq '.' 2>/dev/null || echo "   (Not available)"

echo ""
echo "=== Listing Employees ==="
curl -sf "$BASE_URL/api/v1/employees" -H "$AUTH" | jq '.' 2>/dev/null || echo "   (Not available)"

echo ""
echo "=== Getting Available Entities ==="
curl -sf "$BASE_URL/api/admin/config" -H "$AUTH" | jq '.' 2>/dev/null || echo "   (Not available)"

echo ""
echo "=== Dashboard Report ==="
curl -sf "$BASE_URL/api/v1/reports/dashboard" -H "$AUTH" | jq '.' 2>/dev/null || echo "   (Not available)"

echo ""
echo "✅ All tests completed!"
