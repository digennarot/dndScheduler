#!/bin/bash
# Migration script for Story 5.1 Recurrence
DB_FILE="dnd_scheduler.db"

echo "Applying migration to $DB_FILE..."

sqlite3 "$DB_FILE" <<EOF
-- Add columns to polls table
ALTER TABLE polls ADD COLUMN periodicity TEXT;
ALTER TABLE polls ADD COLUMN recurrence_rule TEXT;

-- Create poll_instances table
CREATE TABLE IF NOT EXISTS poll_instances (
    id TEXT PRIMARY KEY,
    poll_id TEXT NOT NULL,
    date TEXT NOT NULL,
    start_time TEXT,
    end_time TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(poll_id) REFERENCES polls(id) ON DELETE CASCADE
);

-- Index for performance
CREATE INDEX IF NOT EXISTS idx_poll_instances_poll_id ON poll_instances(poll_id);
EOF

echo "Migration complete."
