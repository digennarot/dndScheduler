#!/bin/bash

DB_FILE="dnd_scheduler.db"

echo "=========================================="
echo "Elimina TUTTI gli Utenti (tranne Admin)"
echo "=========================================="
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

echo "⚠️  ATTENZIONE: Questo comando eliminerà TUTTI gli utenti dalla tabella 'users'"
echo "   (La tabella 'admins' rimarrà intatta)"
echo ""

# Count users before deletion
USER_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM users;" 2>/dev/null)

if [ $? -ne 0 ]; then
    echo "❌ Errore: impossibile accedere al database"
    exit 1
fi

echo "Utenti attualmente registrati: $USER_COUNT"
echo ""

if [ "$USER_COUNT" -eq 0 ]; then
    echo "✅ Non ci sono utenti da eliminare"
    exit 0
fi

echo "Lista utenti che verranno eliminati:"
sqlite3 -header -column "$DB_FILE" "SELECT email, name, role FROM users;"
echo ""

read -p "Sei sicuro di voler eliminare TUTTI questi utenti? (s/n): " CONFIRM

if [ "$CONFIRM" != "s" ] && [ "$CONFIRM" != "S" ]; then
    echo ""
    echo "❌ Operazione annullata"
    exit 0
fi

echo ""
echo "Eliminazione in corso..."

# Delete all users
sqlite3 "$DB_FILE" "DELETE FROM users;" 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Tutti gli utenti sono stati eliminati con successo!"
    echo ""
    
    # Verify deletion
    REMAINING=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM users;")
    echo "Utenti rimanenti nella tabella 'users': $REMAINING"
    
    # Show admins are still there
    ADMIN_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM admins;")
    echo "Admin rimanenti nella tabella 'admins': $ADMIN_COUNT"
    echo ""
    echo "✅ La tabella 'admins' è rimasta intatta"
else
    echo "❌ Errore durante l'eliminazione"
    exit 1
fi

echo ""
echo "=========================================="
echo "Ora puoi registrare nuovi utenti su:"
echo "  http://localhost:3000/register.html"
echo "=========================================="
