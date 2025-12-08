#!/bin/bash
# Fix participants with missing access tokens

echo "🔧 Fixing participants with missing access tokens..."

# Create a simple SQL script
cat > /tmp/fix_tokens.sql << 'EOF'
UPDATE participants 
SET access_token = lower(
    hex(randomblob(4)) || '-' || 
    hex(randomblob(2)) || '-' || 
    hex(randomblob(2)) || '-' || 
    hex(randomblob(2)) || '-' || 
    hex(randomblob(6))
)
WHERE access_token IS NULL;

SELECT 'Fixed ' || changes() || ' participants';
EOF

# Check if sqlite3 is available
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ sqlite3 not found. Installing..."
    sudo apt install -y sqlite3 > /dev/null 2>&1
fi

# Run the fix
sqlite3 dnd_scheduler.db < /tmp/fix_tokens.sql

if [ $? -eq 0 ]; then
    echo "✅ Access tokens generated successfully!"
    echo ""
    echo "📋 Now you need to rejoin the session to get your token:"
    echo "   1. Clear browser localStorage: localStorage.clear()"
    echo "   2. Refresh the page"
    echo "   3. Join the session again with your email"
    echo ""
else
    echo "❌ Failed to fix tokens"
    exit 1
fi

# Clean up
rm /tmp/fix_tokens.sql
