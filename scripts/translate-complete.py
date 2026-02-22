#!/usr/bin/env python3
"""
Script completo per tradurre TUTTE le stringhe rimanenti in italiano
"""

import re
from pathlib import Path

# Dizionario COMPLETO di traduzioni
TRANSLATIONS = {
    # Titles
    "D&D Session Scheduler": "Pianificatore Sessioni D&D",
    "Session Scheduler": "Pianificatore Sessioni",
    "Coordinate Your Adventures": "Coordina le Tue Avventure",
    "My Dashboard": "La Mia Bacheca",
    "Join Session": "Partecipa alla Sessione",
    "Create New Campaign": "Crea Nuova Campagna",
    "Admin Dashboard": "Pannello Amministrazione",
    "Profile": "Profilo",
    "Register": "Registrati",
    "Login": "Accedi",
    "Manage Sessions": "Gestisci Sessioni",
    
    # Footer
    "Crafted with magical precision": "Creato con precisione magica",
    "Manage your D&D Session Scheduler": "Gestisci il tuo Pianificatore Sessioni D&D",
    "Monitor your D&D Session Scheduler platform": "Monitora la tua piattaforma Pianificatore Sessioni D&D",
    
    # Common phrases
    "Welcome back": "Bentornato",
    "Adventurer": "Avventuriero",
    "My Sessions": "Le Mie Sessioni",
    "Active Campaigns": "Campagne Attive",
    "Recent Activity": "Attività Recente",
    "Quick Stats": "Statistiche Rapide",
    "Quick Actions": "Azioni Rapide",
    
    # Actions
    "Create Poll": "Crea Sondaggio",
    "Start New Campaign": "Inizia Nuova Campagna",
    "Join Adventure": "Unisciti all'Avventura",
    "View Details": "Vedi Dettagli",
    "Edit Session": "Modifica Sessione",
    "Delete": "Elimina",
    "Save": "Salva",
    "Cancel": "Annulla",
    "Submit": "Invia",
    "Reset": "Ripristina",
    "Back": "Indietro",
    "Next": "Avanti",
    "Previous": "Precedente",
    "Finish": "Termina",
    
    # Stats
    "Total Sessions": "Sessioni Totali",
    "Active Sessions": "Sessioni Attive",
    "Finalized Sessions": "Sessioni Finalizzate",
    "Pending Responses": "Risposte in Attesa",
    "Response Rate": "Tasso di Risposta",
    "Avg Response Rate": "Tasso Risposta Medio",
    "Success Rate": "Tasso di Successo",
    "Avg. Response Time": "Tempo Risposta Medio",
    
    # Messages
    "Loading...": "Caricamento...",
    "No data available": "Nessun dato disponibile",
    "No sessions found": "Nessuna sessione trovata",
    "No recent activity": "Nessuna attività recente",
    "Please wait": "Attendere prego",
    "Error": "Errore",
    "Success": "Successo",
    
    # Form labels
    "Email": "Email",
    "Password": "Password",
    "Confirm Password": "Conferma Password",
    "Full Name": "Nome Completo",
    "Username": "Nome Utente",
    "Campaign Name": "Nome Campagna",
    "Description": "Descrizione",
    "Date": "Data",
    "Time": "Ora",
    "Location": "Luogo",
    "Participants": "Partecipanti",
    "Notes": "Note",
    
    # Placeholders
    "Enter your email": "Inserisci la tua email",
    "Enter your password": "Inserisci la password",
    "Enter campaign name": "Inserisci nome campagna",
    "Add description": "Aggiungi descrizione",
    "Select date": "Seleziona data",
    "Select time": "Seleziona ora",
    
    # Status
    "Active": "Attiva",
    "Pending": "In Attesa",
    "Finalized": "Finalizzata",
    "Cancelled": "Annullata",
    "Completed": "Completata",
    "Draft": "Bozza",
    
    # Time
    "Today": "Oggi",
    "Yesterday": "Ieri",
    "Tomorrow": "Domani",
    "This Week": "Questa Settimana",
    "Last Week": "Settimana Scorsa",
    "This Month": "Questo Mese",
    "Last Month": "Mese Scorso",
    
    # Days
    "Monday": "Lunedì",
    "Tuesday": "Martedì",
    "Wednesday": "Mercoledì",
    "Thursday": "Giovedì",
    "Friday": "Venerdì",
    "Saturday": "Sabato",
    "Sunday": "Domenica",
    
    # Months
    "January": "Gennaio",
    "February": "Febbraio",
    "March": "Marzo",
    "April": "Aprile",
    "May": "Maggio",
    "June": "Giugno",
    "July": "Luglio",
    "August": "Agosto",
    "September": "Settembre",
    "October": "Ottobre",
    "November": "Novembre",
    "December": "Dicembre",
}

def translate_content(content):
    """Applica tutte le traduzioni al contenuto"""
    for english, italian in TRANSLATIONS.items():
        # Sostituisci in vari contesti
        # 1. Tra tag HTML
        content = re.sub(f'>{re.escape(english)}<', f'>{italian}<', content)
        # 2. In attributi con doppi apici
        content = re.sub(f'"{re.escape(english)}"', f'"{italian}"', content)
        # 3. In attributi con apici singoli
        content = re.sub(f"'{re.escape(english)}'", f"'{italian}'", content)
        # 4. In title tag
        content = re.sub(f'<title>{re.escape(english)}', f'<title>{italian}', content)
        content = re.sub(f'{re.escape(english)}</title>', f'{italian}</title>', content)
        # 5. In testo con spazi
        content = re.sub(f'>{re.escape(english)} ', f'>{italian} ', content)
        content = re.sub(f' {re.escape(english)}<', f' {italian}<', content)
    
    return content

def translate_file(file_path):
    """Traduce un file HTML"""
    print(f"Traduzione di {file_path.name}...")
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    content = translate_content(content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        
        # Conta le sostituzioni
        changes = sum(1 for eng in TRANSLATIONS.keys() if eng in original_content)
        print(f"  ✅ {file_path.name} tradotto ({changes} stringhe)")
        return True
    else:
        print(f"  ⏭️  {file_path.name} già tradotto")
        return False

def main():
    """Funzione principale"""
    static_dir = Path("/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/static")
    
    # TUTTI i file HTML
    html_files = list(static_dir.glob("*.html"))
    
    print("🇮🇹 Traduzione COMPLETA in italiano...")
    print("=" * 60)
    print(f"File da tradurre: {len(html_files)}")
    print(f"Stringhe nel dizionario: {len(TRANSLATIONS)}")
    print("=" * 60)
    
    translated_count = 0
    for file_path in sorted(html_files):
        if translate_file(file_path):
            translated_count += 1
    
    print("=" * 60)
    print(f"✅ Traduzione completata!")
    print(f"   File tradotti: {translated_count}/{len(html_files)}")
    print(f"   Stringhe tradotte: {len(TRANSLATIONS)}")
    print("=" * 60)

if __name__ == "__main__":
    main()
