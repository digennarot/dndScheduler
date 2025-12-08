#!/bin/bash

EMAIL="tiziano.digennaro@gmail.com"
BASE_URL="http://localhost:3000/api"

echo "=========================================="
echo "Gestione Account: $EMAIL"
echo "=========================================="
echo ""
echo "L'email è già registrata nel sistema."
echo ""
echo "OPZIONI DISPONIBILI:"
echo "=========================================="
echo ""

echo "1️⃣  FARE LOGIN (Opzione consigliata)"
echo "   Se hai già un account, prova a fare login:"
echo ""
read -p "   Vuoi provare a fare login? (s/n): " LOGIN_CHOICE
echo ""

if [ "$LOGIN_CHOICE" = "s" ] || [ "$LOGIN_CHOICE" = "S" ]; then
    echo "   Inserisci la password per $EMAIL:"
    read -s PASSWORD
    echo ""
    
    echo "   Tentativo di login..."
    RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$BASE_URL/auth/login" \
      -H "Content-Type: application/json" \
      -d "{\"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}")
    
    HTTP_STATUS=$(echo "$RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
    BODY=$(echo "$RESPONSE" | sed '/HTTP_STATUS:/d')
    
    echo ""
    if [ "$HTTP_STATUS" = "200" ]; then
        echo "   ✅ LOGIN RIUSCITO!"
        echo ""
        echo "   Dettagli account:"
        echo "$BODY" | jq .
        echo ""
        TOKEN=$(echo "$BODY" | jq -r '.token')
        ROLE=$(echo "$BODY" | jq -r '.user.role')
        echo "   Token: ${TOKEN:0:30}..."
        echo "   Ruolo: $ROLE"
        echo ""
        
        if [ "$ROLE" = "player" ]; then
            echo "   ⚠️  Hai il ruolo 'player'. Se devi creare sessioni, chiedi di essere promosso a 'dm'."
        elif [ "$ROLE" = "dm" ]; then
            echo "   ✅ Hai il ruolo 'dm'. Puoi creare sessioni!"
        fi
    else
        echo "   ❌ LOGIN FALLITO"
        echo "   Risposta: $BODY"
        echo ""
        echo "   La password potrebbe essere sbagliata."
        echo "   Prova le altre opzioni qui sotto."
    fi
    echo ""
fi

echo "=========================================="
echo ""
echo "2️⃣  ELIMINARE L'ACCOUNT ESISTENTE"
echo "   Per eliminare l'account e ricrearne uno nuovo:"
echo ""
echo "   Richiede installazione di sqlite3:"
echo "   sudo apt-get install sqlite3"
echo ""
echo "   Poi esegui:"
echo "   sqlite3 dnd_scheduler.db \"DELETE FROM users WHERE email = '$EMAIL';\""
echo ""

echo "=========================================="
echo ""
echo "3️⃣  USARE UN'ALTRA EMAIL"
echo "   Registrati con un'email diversa."
echo ""

echo "=========================================="
echo ""
echo "4️⃣  RESET PASSWORD (Se implementato)"
echo "   Contatta l'amministratore per resettare la password."
echo ""

echo "=========================================="
