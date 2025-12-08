#!/usr/bin/env python3
"""
Script per tradurre automaticamente tutte le pagine HTML dall'inglese all'italiano
"""

import re
from pathlib import Path

# Dizionario di traduzioni
TRANSLATIONS = {
    # Navigation
    "Dashboard": "Bacheca",
    "Create Poll": "Crea Sondaggio",
    "Join Session": "Partecipa",
    "Manage": "Gestisci",
    "Admin": "Amministrazione",
    "Login": "Accedi",
    "Sign Up": "Registrati",
    "Logout": "Esci",
    "Back to Dashboard": "Torna alla Bacheca",
    "Back to Home": "Torna alla Home",
    
    # Titles
    "Coordinate Your Epic Adventures": "Coordina le Tue Avventure Epiche",
    "Manage Your Campaigns": "Gestisci le Tue Campagne",
    "Mark Your Availability": "Indica la Tua Disponibilità",
    "Create Account": "Crea Account",
    "Session Scheduler": "Pianificatore Sessioni",
    
    # Buttons
    "Start New Campaign": "Inizia Nuova Campagna",
    "Join Adventure": "Unisciti all'Avventura",
    "Create New Campaign": "Crea Nuova Campagna",
    "Submit Availability": "Invia Disponibilità",
    "Save Draft": "Salva Bozza",
    "Reset": "Ripristina",
    "Cancel": "Annulla",
    "Save Changes": "Salva Modifiche",
    "Confirm & Notify Players": "Conferma e Notifica Giocatori",
    "Send Reminders": "Invia Promemoria",
    "Send Reminder": "Invia Promemoria",
    "View Details": "Vedi Dettagli",
    "Edit Session": "Modifica Sessione",
    "Finalize Time": "Finalizza Orario",
    "Session Finalized": "Sessione Finalizzata",
    "Create Account": "Crea Account",
    "Sign in": "Accedi",
    
    # Descriptions
    "Bring your party together with intelligent scheduling that finds the perfect time for every quest. No more scheduling conflicts, just pure adventure planning magic.": 
        "Riunisci il tuo gruppo con una pianificazione intelligente che trova l'orario perfetto per ogni missione. Niente più conflitti di orario, solo pura magia nella pianificazione delle avventure.",
    "Track responses, analyze availability, and finalize session times for your adventures":
        "Monitora le risposte, analizza la disponibilità e finalizza gli orari delle sessioni per le tue avventure",
    "Help your party find the perfect time for your next adventure":
        "Aiuta il tuo gruppo a trovare l'orario perfetto per la prossima avventura",
    "Your ongoing adventures waiting for the perfect scheduling moment":
        "Le tue avventure in corso in attesa del momento perfetto per la pianificazione",
    "Track your campaign scheduling activity and response patterns":
        "Monitora l'attività di pianificazione della campagna e i pattern di risposta",
    "Daily poll creation and response activity":
        "Attività giornaliera di creazione sondaggi e risposte",
    "Manage your account and preferences":
        "Gestisci il tuo account e le preferenze",
    "Mark your availability for sessions":
        "Indica la tua disponibilità per le sessioni",
    
    # Sections
    "Active Campaigns": "Campagne Attive",
    "My Sessions": "Le Mie Sessioni",
    "Quick Stats": "Statistiche Rapide",
    "Quick Actions": "Azioni Rapide",
    "Profile Settings": "Impostazioni Profilo",
    "Session Details": "Dettagli Sessione",
    "Availability Overlap Analysis": "Analisi Sovrapposizione Disponibilità",
    "Recommended Times": "Orari Consigliati",
    "Participant Responses": "Risposte Partecipanti",
    "Group Overview": "Panoramica Gruppo",
    "Best Times": "Orari Migliori",
    "Total Players": "Giocatori Totali",
    "Scheduling Insights": "Statistiche Pianificazione",
    "Recent Activity": "Attività Recente",
    
    # Stats
    "Total Sessions": "Sessioni Totali",
    "Availability Submitted": "Disponibilità Inviate",
    "Active Sessions": "Sessioni Attive",
    "Finalized Sessions": "Sessioni Finalizzate",
    "Avg Response Rate": "Tasso Risposta Medio",
    "Success Rate": "Tasso di Successo",
    "Avg. Response Time": "Tempo Risposta Medio",
    "This Week's Activity": "Attività di Questa Settimana",
    "Polls successfully scheduled": "Sondaggi pianificati con successo",
    "Time for players to respond": "Tempo per i giocatori di rispondere",
    
    # Messages
    "Welcome back": "Bentornato",
    "Adventurer": "Avventuriero",
    "Loading...": "Caricamento...",
    "Please wait while we fetch your campaigns.": "Attendere mentre carichiamo le tue campagne.",
    "No recent activity": "Nessuna attività recente",
    "You haven't joined any sessions yet.": "Non hai ancora partecipato a nessuna sessione.",
    "Join your first session": "Partecipa alla tua prima sessione",
    
    # Form
    "Full Name": "Nome Completo",
    "Email Address": "Indirizzo Email",
    "Password": "Password",
    "Confirm Password": "Conferma Password",
    "Enter your name": "Inserisci il tuo nome",
    "you@example.com": "tuo@email.com",
    "At least 8 characters": "Almeno 8 caratteri",
    "Re-enter your password": "Reinserisci la password",
    "Must be at least 8 characters long": "Deve essere lungo almeno 8 caratteri",
    
    # Auth
    "Already have an account?": "Hai già un account?",
    "Don't have an account?": "Non hai un account?",
    "Create one": "Creane uno",
    "Forgot password?": "Password dimenticata?",
    
    # Footer
    "Bringing adventurers together, one session at a time": "Riuniamo avventurieri, una sessione alla volta",
    "Crafted with magical precision": "Creato con precisione magica",
    "D&D Session Scheduler": "Pianificatore Sessioni D&D",
}

def translate_file(file_path):
    """Traduce un file HTML"""
    print(f"Traduzione di {file_path.name}...")
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # Applica le traduzioni
    for english, italian in TRANSLATIONS.items():
        # Usa regex per sostituire solo testo, non attributi o codice
        content = content.replace(f">{english}<", f">{italian}<")
        content = content.replace(f'"{english}"', f'"{italian}"')
        content = content.replace(f"'{english}'", f"'{italian}'")
    
    # Se il contenuto è cambiato, salva
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"  ✅ {file_path.name} tradotto")
        return True
    else:
        print(f"  ⏭️  {file_path.name} già tradotto o nessuna traduzione necessaria")
        return False

def main():
    """Funzione principale"""
    static_dir = Path("/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/static")
    
    # File da tradurre
    html_files = [
        "index.html",
        "dashboard.html",
        "create-poll.html",
        "participate.html",
        "login.html",
        "register.html",
        "admin.html",
        "profile.html",
    ]
    
    print("🇮🇹 Inizio traduzione automatica...")
    print("=" * 50)
    
    translated_count = 0
    for filename in html_files:
        file_path = static_dir / filename
        if file_path.exists():
            if translate_file(file_path):
                translated_count += 1
        else:
            print(f"  ⚠️  {filename} non trovato")
    
    print("=" * 50)
    print(f"✅ Traduzione completata! {translated_count} file tradotti")

if __name__ == "__main__":
    main()
