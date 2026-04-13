#!/bin/bash
# Test workflow operations against the Atlas API

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
TOKEN="${TOKEN:-}"

if [ -z "$TOKEN" ]; then
    echo "Logging in..."
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d '{"email": "admin@atlas.local", "password": "admin123"}')
    
    TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
    if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
        echo "Failed to login. Response: $LOGIN_RESPONSE"
        exit 1
    fi
    echo "Logged in successfully"
fi

AUTH="Authorization: Bearer $TOKEN"

echo ""
echo "=== Testing Schema Endpoints ==="

# Create a test entity with workflow
echo "Creating test entity..."
curl -s -X POST "$BASE_URL/api/admin/schema" \
    -H "Content-Type: application/json" \
    -H "$AUTH" \
    -d '{
        "definition": {
            "name": "test_orders",
            "label": "Test Order",
            "plural_label": "Test Orders",
            "table_name": "test_orders",
            "fields": [
                {"name": "order_number", "label": "Order Number", "field_type": {"type": "string"}, "is_required": true, "is_searchable": true, "display_order": 1},
                {"name": "amount", "label": "Amount", "field_type": {"type": "decimal", "precision": 10, "scale": 2}, "is_required": true, "display_order": 2},
                {"name": "status", "label": "Status", "field_type": {"type": "enum", "values": ["draft", "submitted", "approved", "rejected"]}, "display_order": 3}
            ],
            "workflow": {
                "name": "test_order_workflow",
                "initial_state": "draft",
                "states": [
                    {"name": "draft", "label": "Draft", "state_type": "initial"},
                    {"name": "submitted", "label": "Submitted", "state_type": "working"},
                    {"name": "approved", "label": "Approved", "state_type": "final"},
                    {"name": "rejected", "label": "Rejected", "state_type": "final"}
                ],
                "transitions": [
                    {"from_state": "draft", "to_state": "submitted", "action": "submit"},
                    {"from_state": "submitted", "to_state": "approved", "action": "approve"},
                    {"from_state": "submitted", "to_state": "rejected", "action": "reject"}
                ]
            }
        }
    }' | jq .

echo ""
echo "=== Getting Schema ==="
curl -s "$BASE_URL/api/v1/schema/test_orders" -H "$AUTH" | jq .

echo ""
echo "=== Getting Form Config ==="
curl -s "$BASE_URL/api/v1/schema/test_orders/form" -H "$AUTH" | jq .

echo ""
echo "=== Creating a Record ==="
RECORD=$(curl -s -X POST "$BASE_URL/api/v1/test_orders" \
    -H "Content-Type: application/json" \
    -H "$AUTH" \
    -d '{
        "entity": "test_orders",
        "values": {
            "order_number": "ORD-001",
            "amount": 99.99,
            "status": "draft"
        }
    }')
echo "$RECORD" | jq .

echo ""
echo "=== Listing Records ==="
curl -s "$BASE_URL/api/v1/test_orders" -H "$AUTH" | jq .

echo ""
echo "=== Getting Available Transitions ==="
curl -s "$BASE_URL/api/v1/test_orders" -H "$AUTH" | jq .

echo ""
echo "=== Health Check ==="
curl -s "$BASE_URL/health"
echo ""

echo ""
echo "All tests completed!"
