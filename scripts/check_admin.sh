#!/bin/bash
echo "Checking default admin credentials..."
echo "Default admin username: admin"
echo "Default admin password: password123"
echo ""
echo "Testing admin login..."
curl -s -X POST http://localhost:3000/api/admin/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}' | jq .
