#!/usr/bin/env python3
"""
Script per aggiornare la navigazione in tutte le pagine HTML:
- Rimuove "Crea Sondaggio" dalla navigazione principale
- Mantiene "Gestisci" con id="nav-manage" per protezione
"""

import re
from pathlib import Path

def update_navigation(file_path):
    """Aggiorna la navigazione in un file HTML"""
    print(f"Aggiornamento {file_path.name}...")
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # Pattern per trovare la sezione di navigazione
    # Rimuove la riga con "Crea Sondaggio"
    patterns = [
        # Pattern 1: Link a create-poll.html con "Crea Sondaggio"
        (r'\s*<a href="create-poll\.html"[^>]*>Crea Sondaggio</a>\n', ''),
        
        # Pattern 2: Link a create-poll.html con "Create Poll"
        (r'\s*<a href="create-poll\.html"[^>]*>Create Poll</a>\n', ''),
        
        # Pattern 3: Aggiunge id="nav-manage" al link Gestisci se non c'è
        (r'<a href="manage\.html"([^>]*class="[^"]*")>Gestisci</a>',
         r'<a href="manage.html"\1 id="nav-manage">Gestisci</a>'),
        
        # Pattern 4: Aggiunge id="nav-manage" al link Manage se non c'è
        (r'<a href="manage\.html"([^>]*class="[^"]*")>Manage</a>',
         r'<a href="manage.html"\1 id="nav-manage">Gestisci</a>'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    # Se il contenuto è cambiato, salva
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"  ✅ {file_path.name} aggiornato")
        return True
    else:
        print(f"  ⏭️  {file_path.name} già aggiornato")
        return False

def main():
    """Funzione principale"""
    static_dir = Path("/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/static")
    
    # File da aggiornare
    html_files = [
        "index.html",
        "dashboard.html",
        "participate.html",
        "manage.html",
        "admin.html",
        "profile.html",
        "create-poll.html",
    ]
    
    print("🔧 Aggiornamento navigazione...")
    print("=" * 50)
    print("Rimuovo 'Crea Sondaggio' dalla navigazione")
    print("Mantengo 'Gestisci' con protezione")
    print("=" * 50)
    
    updated_count = 0
    for filename in html_files:
        file_path = static_dir / filename
        if file_path.exists():
            if update_navigation(file_path):
                updated_count += 1
        else:
            print(f"  ⚠️  {filename} non trovato")
    
    print("=" * 50)
    print(f"✅ Aggiornamento completato! {updated_count} file modificati")

if __name__ == "__main__":
    main()
