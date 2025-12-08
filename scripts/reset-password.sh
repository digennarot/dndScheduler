#!/bin/bash
# Reset admin password to default

echo "🔐 Resetting admin password..."

# Check if database exists
if [ ! -f "dnd_scheduler.db" ]; then
    echo "❌ Database not found!"
    exit 1
fi

# Install sqlite3 if not available
if ! command -v sqlite3 &> /dev/null; then
    echo "📦 Installing sqlite3..."
    sudo apt install -y sqlite3
fi

# Generate bcrypt hash for "password123"
# This is a pre-computed hash for "password123"
HASH='$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYqYr8OYQOK'

# Update admin password
sqlite3 dnd_scheduler.db "UPDATE admins SET password_hash = '$HASH' WHERE username = 'admin';"

if [ $? -eq 0 ]; then
    echo "✅ Admin password reset successfully!"
    echo ""
    echo "📋 New credentials:"
    echo "   Username: admin"
    echo "   Password: password123"
    echo ""
    echo "🌐 Login at: http://localhost:3000/admin.html"
else
    echo "❌ Failed to reset password"
    exit 1
fi
