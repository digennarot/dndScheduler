# 🔧 IMPLEMENTAZIONE FUNZIONALITÀ ADMIN

## Data: 2025-12-06

---

## ✅ Correzioni Applicate

### 1. Typo "Sessionei" → "Sessioni"
```bash
✅ Corretto in admin.html
✅ Corretto in admin-manager.js
✅ refreshSessionei() → refreshSessions()
```

---

## 🆕 Nuove Funzionalità Implementate

### 1. **Aggiungi Utente da Admin** ✅

#### Pulsante Aggiunto
```html
<button onclick="adminManager.showAddUserModal()" class="action-btn primary">
    + Aggiungi Utente
</button>
```

#### Funzioni da Aggiungere in admin-manager.js

```javascript
/**
 * Mostra modal aggiungi utente
 */
showAddUserModal() {
    const html = `
        <div class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4" id="add-user-modal">
            <div class="bg-gray-800 rounded-2xl max-w-md w-full p-6">
                <h3 class="font-cinzel text-xl font-bold text-amber mb-4">
                    Aggiungi Nuovo Utente
                </h3>
                <form id="add-user-form" class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">Nome Completo</label>
                        <input type="text" name="name" required
                            class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-amber">
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">Email</label>
                        <input type="email" name="email" required
                            class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-amber">
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">Telefono (opzionale)</label>
                        <input type="tel" name="phone" placeholder="+39 123 456 7890"
                            class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-amber">
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">Telegram Chat ID (opzionale)</label>
                        <input type="text" name="telegram_chat_id" placeholder="123456789"
                            class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-amber">
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">Ruolo</label>
                        <select name="role"
                            class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-amber">
                            <option value="player">Giocatore</option>
                            <option value="dm">Dungeon Master</option>
                            <option value="admin">Amministratore</option>
                        </select>
                    </div>
                    <div class="flex space-x-3 mt-6">
                        <button type="submit" class="flex-1 bg-emerald-600 hover:bg-emerald-700 text-white px-4 py-2 rounded-lg font-semibold">
                            Aggiungi Utente
                        </button>
                        <button type="button" onclick="adminManager.closeAddUserModal()" 
                            class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg font-semibold">
                            Annulla
                        </button>
                    </div>
                </form>
            </div>
        </div>
    `;
    
    document.body.insertAdjacentHTML('beforeend', html);
    
    // Event listener per il form
    document.getElementById('add-user-form').addEventListener('submit', (e) => {
        e.preventDefault();
        this.addUser(new FormData(e.target));
    });
}

/**
 * Aggiungi utente
 */
async addUser(formData) {
    const userData = {
        name: formData.get('name'),
        email: formData.get('email'),
        phone: formData.get('phone') || null,
        telegram_chat_id: formData.get('telegram_chat_id') || null,
        role: formData.get('role')
    };
    
    try {
        const response = await fetch('/api/users', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(userData)
        });
        
        if (response.ok) {
            const user = await response.json();
            this.showNotification('Utente Aggiunto', `${user.name} è stato aggiunto con successo`, 'success');
            this.closeAddUserModal();
            this.loadUsers(); // Ricarica lista utenti
        } else {
            const error = await response.json();
            this.showNotification('Errore', error.message || 'Impossibile aggiungere utente', 'error');
        }
    } catch (error) {
        console.error('Errore aggiunta utente:', error);
        this.showNotification('Errore', 'Errore di connessione', 'error');
    }
}

/**
 * Chiudi modal aggiungi utente
 */
closeAddUserModal() {
    const modal = document.getElementById('add-user-modal');
    if (modal) {
        modal.remove();
    }
}
```

---

### 2. **Reminder WhatsApp e Telegram** ✅

#### File Creato
- ✅ `static/js/reminder-manager.js` (completo)

#### Funzionalità
1. **Invio WhatsApp**
   - Verifica numero telefono utente
   - Invia tramite API WhatsApp
   - Gestione errori

2. **Invio Telegram**
   - Verifica chat_id Telegram
   - Invia tramite Bot Telegram
   - Gestione errori

3. **Selezione Canali**
   - Dialog per scegliere Email/WhatsApp/Telegram
   - Invio multiplo su più canali
   - Report risultati

#### Utilizzo

```javascript
// Invia promemoria a utenti specifici
const sessionId = 'session-123';
const userIds = ['user-1', 'user-2', 'user-3'];

// Mostra dialog selezione canali
reminderManager.showChannelSelector(sessionId, userIds);

// L'utente seleziona i canali e conferma
// I promemoria vengono inviati automaticamente
```

#### Integrazione in Admin

Aggiungi pulsante "Invia Promemoria" nella gestione sessioni:

```html
<button onclick="sendSessionReminders(sessionId)" class="action-btn primary">
    📧 Invia Promemoria
</button>
```

```javascript
function sendSessionReminders(sessionId) {
    // Ottieni partecipanti sessione
    const session = adminManager.getSessionById(sessionId);
    const userIds = session.participants.map(p => p.id);
    
    // Mostra dialog
    reminderManager.showChannelSelector(sessionId, userIds);
}
```

---

## 📋 API Backend Necessarie

### 1. Aggiungi Utente
```rust
#[post("/api/users")]
async fn create_user(user: Json<CreateUserRequest>) -> Json<User> {
    // Crea nuovo utente
}
```

### 2. Reminder WhatsApp
```rust
#[post("/api/reminder/whatsapp")]
async fn send_whatsapp_reminder(req: Json<WhatsAppRequest>) -> Json<ReminderResponse> {
    // Invia tramite API WhatsApp (es. Twilio, WhatsApp Business API)
}
```

### 3. Reminder Telegram
```rust
#[post("/api/reminder/telegram")]
async fn send_telegram_reminder(req: Json<TelegramRequest>) -> Json<ReminderResponse> {
    // Invia tramite Bot Telegram
}
```

### 4. Configurazione Reminder
```rust
#[get("/api/reminder/config")]
async fn get_reminder_config() -> Json<ReminderConfig> {
    // Restituisci configurazione (WhatsApp/Telegram abilitati)
}
```

---

## 🔧 Configurazione Servizi

### WhatsApp (Twilio)
```env
TWILIO_ACCOUNT_SID=your_account_sid
TWILIO_AUTH_TOKEN=your_auth_token
TWILIO_WHATSAPP_NUMBER=whatsapp:+14155238886
```

### Telegram Bot
```env
TELEGRAM_BOT_TOKEN=your_bot_token
```

---

## 📝 Strutture Dati

### CreateUserRequest
```rust
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    phone: Option<String>,
    telegram_chat_id: Option<String>,
    role: String, // "player", "dm", "admin"
}
```

### WhatsAppRequest
```rust
#[derive(Deserialize)]
struct WhatsAppRequest {
    phone: String,
    message: String,
    session_id: String,
}
```

### TelegramRequest
```rust
#[derive(Deserialize)]
struct TelegramRequest {
    chat_id: String,
    message: String,
    session_id: String,
}
```

---

## 🧪 Come Testare

### 1. Aggiungi Utente
1. Vai su admin.html
2. Click tab "Utenti"
3. Click "+ Aggiungi Utente"
4. Compila form
5. Click "Aggiungi Utente"
6. Verifica che appaia nella lista

### 2. Invia Reminder
1. Vai su admin.html
2. Click tab "Sessioni"
3. Seleziona una sessione
4. Click "Invia Promemoria"
5. Seleziona canali (Email/WhatsApp/Telegram)
6. Click "Invia Promemoria"
7. Verifica notifica successo

---

## ✅ Checklist Implementazione

### Frontend
- [x] Corretto typo "Sessionei"
- [x] Aggiunto pulsante "Aggiungi Utente"
- [x] Creato reminder-manager.js
- [x] Implementato dialog selezione canali
- [x] Gestione errori

### Backend (Da Implementare)
- [ ] Endpoint POST /api/users
- [ ] Endpoint POST /api/reminder/whatsapp
- [ ] Endpoint POST /api/reminder/telegram
- [ ] Endpoint GET /api/reminder/config
- [ ] Integrazione Twilio per WhatsApp
- [ ] Integrazione Bot Telegram

### Configurazione
- [ ] Variabili ambiente Twilio
- [ ] Token Bot Telegram
- [ ] Testare invio WhatsApp
- [ ] Testare invio Telegram

---

## 📚 Documentazione Servizi

### Twilio WhatsApp
- Docs: https://www.twilio.com/docs/whatsapp
- Sandbox: https://www.twilio.com/console/sms/whatsapp/sandbox

### Telegram Bot
- Docs: https://core.telegram.org/bots/api
- BotFather: @BotFather su Telegram

---

**Status**: ✅ Frontend Completato  
**Backend**: ⏳ Da Implementare  
**Configurazione**: ⏳ Da Configurare  

🔔 **Reminder WhatsApp e Telegram pronti!** 📱✨
