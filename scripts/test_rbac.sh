#!/bin/bash

# Test script for RBAC (Role-Based Access Control) feature
# This script tests that only DMs can create polls

set -e  # Exit on error

echo "=========================================="
echo "RBAC Feature Test"
echo "=========================================="
echo ""

# Base URL
BASE_URL="http://localhost:3000"
API_URL="$BASE_URL/api"

# Test user credentials
TEST_EMAIL="testuser_$(date +%s)@example.com"
TEST_NAME="Test Player"
TEST_PASSWORD="TestPassword123!"  # Must meet password requirements: 12+ chars, uppercase, lowercase, number, special char

echo "Step 1: Register a new user (should default to 'player' role)"
echo "Email: $TEST_EMAIL"
echo ""

REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$TEST_EMAIL\",
    \"name\": \"$TEST_NAME\",
    \"password\": \"$TEST_PASSWORD\"
  }")

echo "Registration response:"
echo "$REGISTER_RESPONSE" | jq .

# Extract token and user info
TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r '.token')
USER_ROLE=$(echo "$REGISTER_RESPONSE" | jq -r '.user.role')
USER_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.user.id')

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    echo "❌ ERROR: Failed to register user"
    exit 1
fi

echo ""
echo "✅ User registered successfully"
echo "   Token: ${TOKEN:0:20}..."
echo "   Role: $USER_ROLE"
echo "   User ID: $USER_ID"
echo ""

# Verify user has 'player' role
if [ "$USER_ROLE" != "player" ]; then
    echo "❌ ERROR: User role should be 'player' but got '$USER_ROLE'"
    exit 1
fi

echo "✅ Verified: Default role is 'player'"
echo ""

echo "=========================================="
echo "Step 2: Try to create a poll as a player (should FAIL)"
echo ""

CREATE_POLL_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$API_URL/polls" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "Test Campaign",
    "description": "This should fail",
    "location": "Test Location",
    "dates": ["2025-12-15"],
    "participants": []
  }')

# Extract HTTP status code
HTTP_STATUS=$(echo "$CREATE_POLL_RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
RESPONSE_BODY=$(echo "$CREATE_POLL_RESPONSE" | sed '/HTTP_STATUS:/d')

echo "HTTP Status: $HTTP_STATUS"
echo "Response:"
echo "$RESPONSE_BODY" | jq . 2>/dev/null || echo "$RESPONSE_BODY"
echo ""

# Should get 403 Forbidden
if [ "$HTTP_STATUS" != "403" ]; then
    echo "❌ ERROR: Expected HTTP 403 (Forbidden) but got $HTTP_STATUS"
    exit 1
fi

echo "✅ Verified: Player cannot create polls (HTTP 403)"
echo ""

echo "=========================================="
echo "Step 3: Promote user to DM role"
echo ""

# Update user role in database
sqlite3 dnd_scheduler.db "UPDATE users SET role = 'dm' WHERE id = '$USER_ID';"

# Verify update
UPDATED_ROLE=$(sqlite3 dnd_scheduler.db "SELECT role FROM users WHERE id = '$USER_ID';")
echo "Updated role in database: $UPDATED_ROLE"

if [ "$UPDATED_ROLE" != "dm" ]; then
    echo "❌ ERROR: Failed to update user role to 'dm'"
    exit 1
fi

echo "✅ User promoted to DM"
echo ""

echo "=========================================="
echo "Step 4: Try to create a poll as a DM (should SUCCEED)"
echo ""

# Re-login to get new token with updated role
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$TEST_EMAIL\",
    \"password\": \"$TEST_PASSWORD\"
  }")

NEW_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
NEW_ROLE=$(echo "$LOGIN_RESPONSE" | jq -r '.user.role')

echo "Re-login response:"
echo "   New Token: ${NEW_TOKEN:0:20}..."
echo "   Role: $NEW_ROLE"
echo ""

if [ "$NEW_ROLE" != "dm" ]; then
    echo "❌ ERROR: User role should be 'dm' but got '$NEW_ROLE'"
    exit 1
fi

# Now try to create poll with DM token
CREATE_POLL_DM_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$API_URL/polls" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $NEW_TOKEN" \
  -d '{
    "title": "DM Test Campaign",
    "description": "This should succeed",
    "location": "Test Dungeon",
    "dates": ["2025-12-15", "2025-12-16"],
    "participants": []
  }')

# Extract HTTP status code
HTTP_STATUS_DM=$(echo "$CREATE_POLL_DM_RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
RESPONSE_BODY_DM=$(echo "$CREATE_POLL_DM_RESPONSE" | sed '/HTTP_STATUS:/d')

echo "HTTP Status: $HTTP_STATUS_DM"
echo "Response:"
echo "$RESPONSE_BODY_DM" | jq . 2>/dev/null || echo "$RESPONSE_BODY_DM"
echo ""

# Should get 201 Created or 200 OK
if [ "$HTTP_STATUS_DM" != "200" ] && [ "$HTTP_STATUS_DM" != "201" ]; then
    echo "❌ ERROR: Expected HTTP 200/201 but got $HTTP_STATUS_DM"
    exit 1
fi

POLL_ID=$(echo "$RESPONSE_BODY_DM" | jq -r '.id')

if [ "$POLL_ID" == "null" ] || [ -z "$POLL_ID" ]; then
    echo "❌ ERROR: Failed to create poll as DM"
    exit 1
fi

echo "✅ Verified: DM can create polls"
echo "   Poll ID: $POLL_ID"
echo ""

echo "=========================================="
echo "✅ ALL TESTS PASSED!"
echo "=========================================="
echo ""
echo "RBAC Feature Verification Summary:"
echo "1. ✅ Users register with 'player' role by default"
echo "2. ✅ Players cannot create polls (403 Forbidden)"
echo "3. ✅ Users can be promoted to 'dm' via database update"
echo "4. ✅ DMs can create polls successfully"
echo ""
echo "To promote users to DM in production, run:"
echo "sqlite3 dnd_scheduler.db \"UPDATE users SET role = 'dm' WHERE email = 'user@example.com';\""
echo ""
