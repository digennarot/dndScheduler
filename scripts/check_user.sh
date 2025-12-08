#!/bin/bash

# Check if user exists
EMAIL="tiziano.digennaro@gmail.com"
DB_FILE="dnd_scheduler.db"

echo "=========================================="
echo "Verifica Utente nel Database"
echo "=========================================="
echo ""
echo "Email: $EMAIL"
echo ""

# Check if sqlite3 is available
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ sqlite3 non è installato"
    echo ""
    echo "Installa sqlite3 con:"
    echo "  sudo apt-get install sqlite3"
    echo ""
    exit 1
fi

echo "Cerco l'utente nel database..."
echo ""

# Query the database
RESULT=$(sqlite3 "$DB_FILE" "SELECT id, email, name, role, datetime(created_at, 'unixepoch', 'localtime') as created_at FROM users WHERE email = '$EMAIL';" 2>&1)

if [ $? -ne 0 ]; then
    echo "❌ Errore nella query al database"
    echo "$RESULT"
    exit 1
fi

if [ -z "$RESULT" ]; then
    echo "❌ Utente NON trovato nel database"
    echo "Puoi procedere con la registrazione"
else
    echo "✅ Utente TROVATO nel database:"
    echo ""
    sqlite3 -header -column "$DB_FILE" "SELECT id, email, name, role, datetime(created_at, 'unixepoch', 'localtime') as created_at FROM users WHERE email = '$EMAIL';"
    echo ""
    
    # Get user role
    ROLE=$(sqlite3 "$DB_FILE" "SELECT role FROM users WHERE email = '$EMAIL';")
    
    echo "=========================================="
    echo "Opzioni disponibili:"
    echo "=========================================="
    echo ""
    echo "1. 🔑 FARE LOGIN invece di registrarsi"
    echo "   Endpoint: POST /api/auth/login"
    echo "   Body: {\"email\": \"$EMAIL\", \"password\": \"YOUR_PASSWORD\"}"
    echo ""
    echo "2. 🗑️  ELIMINARE l'account esistente e ricrearne uno nuovo"
    echo "   sqlite3 $DB_FILE \"DELETE FROM users WHERE email = '$EMAIL';\""
    echo ""
    echo "3. 📧 USARE un'altra email per la registrazione"
    echo ""
    
    if [ "$ROLE" = "player" ]; then
        echo "4. 👑 PROMUOVERE a DM (se necessario)"
        echo "   sqlite3 $DB_FILE \"UPDATE users SET role = 'dm' WHERE email = '$EMAIL';\""
        echo ""
    fi
fi
