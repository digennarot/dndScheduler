#!/bin/bash

# Configuration
API_URL="http://localhost:3000/api"
EMAIL="test_ics_strong@example.com"
PASSWORD="CorrectHorseBatteryStaple123!" # Stronger password

# 0. Register (Ensure user exists)
echo "Registering..."
curl -v -X POST "$API_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$EMAIL\", \"password\": \"$PASSWORD\", \"name\": \"Test User\"}"

# 1. Login
echo "Logging in..."
TOKEN=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}" | jq -r '.token')

if [ "$TOKEN" == "null" ]; then
  echo "Login failed"
  exit 1
fi

echo "Logged in successfully. Token: ${TOKEN:0:10}..."

# 2. Create Recurring Poll
echo "Creating Poll..."
CREATE_RESPONSE=$(curl -s -v -X POST "$API_URL/polls" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "ICS Test Campaign",
    "description": "Testing ICS export",
    "location": "Discord",
    "dates": ["2026-12-01"], 
    "timeRange": "[\"19:00\"]",
    "participants": [],
    "periodicity": "weekly",
    "recurrence_rule": "FREQ=WEEKLY;BYDAY=FR"
  }')

echo "Create Response: $CREATE_RESPONSE"
POLL_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')

if [ "$POLL_ID" == "null" ]; then
  echo "Poll creation failed"
  exit 1
fi

echo "Poll Created: $POLL_ID"

# 3. Export ICS
echo "Exporting ICS..."
curl -v "$API_URL/polls/$POLL_ID/export" \
  -H "Authorization: Bearer $TOKEN" > test_export.ics

echo "ICS Content:"
cat test_export.ics
