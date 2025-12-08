#!/bin/bash

echo "=========================================="
echo "Test Admin Login con Email"
echo "=========================================="
echo ""

BASE_URL="http://localhost:3000/api"

echo "Test 1: Login con EMAIL (nuovo sistema)"
echo "Email: admin@example.com"
echo "Password: password123"
echo ""

RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$BASE_URL/admin/login" \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "password123"}')

HTTP_STATUS=$(echo "$RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_STATUS:/d')

echo "HTTP Status: $HTTP_STATUS"
echo "Response:"
echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
echo ""

if [ "$HTTP_STATUS" = "200" ]; then
    echo "✅ Login con email FUNZIONA!"
    TOKEN=$(echo "$BODY" | jq -r '.token')
    echo "Token ricevuto: ${TOKEN:0:30}..."
else
    echo "❌ Login con email FALLITO!"
fi

echo ""
echo "=========================================="
echo "Test 2: Login con USERNAME (vecchio sistema - dovrebbe fallire)"
echo "Username: admin"
echo "Password: password123"
echo ""

RESPONSE2=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$BASE_URL/admin/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}')

HTTP_STATUS2=$(echo "$RESPONSE2" | grep "HTTP_STATUS:" | cut -d':' -f2)
BODY2=$(echo "$RESPONSE2" | sed '/HTTP_STATUS:/d')

echo "HTTP Status: $HTTP_STATUS2"
echo "Response:"
echo "$BODY2" | jq . 2>/dev/null || echo "$BODY2"
echo ""

if [ "$HTTP_STATUS2" = "422" ] || [ "$HTTP_STATUS2" = "400" ]; then
    echo "✅ Correttamente rifiutato (il campo username non è più supportato)"
else
    echo "⚠️  Risposta inaspettata"
fi

echo ""
echo "=========================================="
echo "Riepilogo:"
echo "- Login con EMAIL: $([ "$HTTP_STATUS" = "200" ] && echo '✅ FUNZIONA' || echo '❌ FALLITO')"
echo "- Login con USERNAME: $([ "$HTTP_STATUS2" != "200" ] && echo '✅ Correttamente rifiutato' || echo '⚠️ Ancora accettato')"
echo "=========================================="
