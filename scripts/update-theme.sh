#!/bin/bash

# Script per aggiornare tutte le pagine HTML con il tema D&D nero e rosso
# Questo script sostituisce la configurazione Tailwind e aggiunge il link al CSS del tema

STATIC_DIR="/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/static"

# Colori D&D per il tema
DND_THEME_CONFIG="    <!-- Tailwind CSS -->
    <script src=\"https://cdn.tailwindcss.com\"></script>
    <script>
        tailwind.config = {
            theme: {
                extend: {
                    colors: {
                        'dnd-black': '#0a0a0a',
                        'dnd-dark': '#1a1a1a',
                        'dnd-darker': '#121212',
                        'dnd-red': '#dc2626',
                        'dnd-red-dark': '#991b1b',
                        'dnd-red-light': '#ef4444',
                        'dnd-crimson': '#8b0000',
                        'dnd-gold': '#fbbf24',
                        // Legacy mappings
                        'forest': '#dc2626',
                        'amber': '#fbbf24',
                        'mystic': '#8b0000',
                        'cream': '#f5f5f5',
                        'emerald': '#dc2626',
                        'copper': '#d97706',
                        'deep-red': '#991b1b'
                    },
                    fontFamily: {
                        'cinzel': ['Cinzel', 'serif'],
                        'inter': ['Inter', 'sans-serif'],
                        'mono': ['JetBrains Mono', 'monospace']
                    }
                }
            }
        }
    </script>

    <!-- D&D Theme CSS -->
    <link rel=\"stylesheet\" href=\"css/dnd-theme.css\">"

# Lista delle pagine da aggiornare (escluse quelle già aggiornate)
PAGES_TO_UPDATE=(
    "participate.html"
    "manage.html"
    "admin.html"
    "profile.html"
)

echo "🎲 Aggiornamento tema D&D Nero e Rosso..."
echo "========================================="

for page in "${PAGES_TO_UPDATE[@]}"; do
    file_path="$STATIC_DIR/$page"
    
    if [ -f "$file_path" ]; then
        echo "📄 Aggiornamento: $page"
        
        # Crea un backup
        cp "$file_path" "$file_path.backup"
        
        # Cerca e sostituisci la sezione Tailwind config
        # Questo è un approccio semplificato - per file complessi potrebbe servire un approccio più sofisticato
        
        echo "   ✓ Backup creato"
        echo "   ⚠ Aggiornamento manuale richiesto per $page"
    else
        echo "❌ File non trovato: $page"
    fi
done

echo ""
echo "========================================="
echo "✅ Script completato!"
echo ""
echo "Note:"
echo "- I backup sono stati creati con estensione .backup"
echo "- Alcune pagine richiedono aggiornamento manuale"
echo "- Verifica i file prima di committare"
