#!/bin/bash
# scripts/decrypt.sh

if ! command -v sops >/dev/null 2>&1; then
    # If sops is not installed, output the encrypted content as-is so checkout doesn't fail
    cat
    exit 0
fi

TMP_FILE=$(mktemp --suffix .env)
cat > "$TMP_FILE"

# Try decrypting. If it fails (e.g., no age key configured), fallback to original encrypted content
# so that the developer isn't blocked from checking out the repo initially.
sops --decrypt --input-type dotenv --output-type dotenv "$TMP_FILE" 2>/dev/null
if [ $? -ne 0 ]; then
    cat "$TMP_FILE"
fi

rm -f "$TMP_FILE"
