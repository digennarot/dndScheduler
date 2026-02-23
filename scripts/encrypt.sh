#!/bin/bash
# scripts/encrypt.sh

# Ensure sops is installed
if ! command -v sops >/dev/null 2>&1; then
    echo "ERROR: sops is not installed. Will not encrypt or commit sensitive files." >&2
    exit 1
fi

TMP_FILE=$(mktemp --suffix .env)
cat > "$TMP_FILE"

# Encrypt the file using dotenv input/output types
sops --encrypt --input-type dotenv --output-type dotenv "$TMP_FILE"
EC=$?

rm -f "$TMP_FILE"
exit $EC
