#!/usr/bin/env python3
"""
Script per aggiungere nav-protection.js a tutte le pagine HTML
"""

import re
from pathlib import Path

def add_nav_protection_script(file_path):
    """Aggiunge lo script nav-protection.js se non presente"""
    print(f"Aggiornamento {file_path.name}...")
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Controlla se lo script è già presente
    if 'nav-protection.js' in content:
        print(f"  ⏭️  {file_path.name} ha già lo script")
        return False
    
    # Trova la sezione degli script prima di </body>
    # Aggiunge lo script prima di auth.js o app.js se presenti
    patterns = [
        # Prima di auth.js
        (r'(\s*)<script src="js/auth\.js">', r'\1<script src="js/nav-protection.js"></script>\n\1<script src="js/auth.js">'),
        # Prima di app.js se non c'è auth.js
        (r'(\s*)<script src="js/app\.js">', r'\1<script src="js/nav-protection.js"></script>\n\1<script src="js/app.js">'),
        # Prima di </body> come fallback
        (r'(\s*)</body>', r'\1<script src="js/nav-protection.js"></script>\n\1</body>'),
    ]
    
    original_content = content
    for pattern, replacement in patterns:
        new_content = re.sub(pattern, replacement, content, count=1)
        if new_content != content:
            content = new_content
            break
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"  ✅ {file_path.name} aggiornato")
        return True
    else:
        print(f"  ⚠️  {file_path.name} - pattern non trovato")
        return False

def main():
    """Funzione principale"""
    static_dir = Path("/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/static")
    
    # File da aggiornare (tutte le pagine con navigazione)
    html_files = [
        "index.html",
        "dashboard.html",
        "participate.html",
        "manage.html",
        "admin.html",
        "profile.html",
    ]
    
    print("🔒 Aggiunta protezione navigazione...")
    print("=" * 50)
    
    updated_count = 0
    for filename in html_files:
        file_path = static_dir / filename
        if file_path.exists():
            if add_nav_protection_script(file_path):
                updated_count += 1
        else:
            print(f"  ⚠️  {filename} non trovato")
    
    print("=" * 50)
    print(f"✅ Completato! {updated_count} file aggiornati")

if __name__ == "__main__":
    main()
