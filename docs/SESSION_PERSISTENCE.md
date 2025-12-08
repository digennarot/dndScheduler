# ✅ SESSIONE PERSISTENTE E LOGOUT

## Data: 2025-12-06

---

## ✅ FUNZIONALITÀ IMPLEMENTATE

### 1. Sessione Persistente ✅
La sessione viene mantenuta anche dopo refresh/chiusura browser

### 2. Pulsante Logout ✅
Pulsante "Esci" visibile quando loggato

### 3. Display Utente ✅
Mostra nome e avatar quando loggato

---

## 🔧 Come Funziona

### Persistenza Sessione

```javascript
// Al login
localStorage.setItem('authToken', token);
localStorage.setItem('currentUser', JSON.stringify(user));

// Al caricamento pagina
window.authManager = new AuthManager();
// → Legge da localStorage
// → Verifica sessione con backend
// → Aggiorna UI
```

### Verifica Automatica

```javascript
document.addEventListener('DOMContentLoaded', async () => {
    if (window.authManager.isLoggedIn()) {
        await window.authManager.verifySession();
        window.authManager.updateUserDisplay();
    }
});
```

**Cosa fa:**
1. Controlla se c'è token in localStorage
2. Verifica token con backend (`/api/auth/me/{token}`)
3. Se valido: aggiorna user display
4. Se invalido: logout automatico

---

## 🎨 UI Elementi

### Quando NON Loggato

```
[FoundryVTT] [Admin] [Accedi]
```

### Quando Loggato

```
[FoundryVTT] [Admin] [👤 Mario Rossi] [Esci]
                      mario@example.com
```

---

## 📝 Elementi HTML

### index.html (Aggiunto)

```html
<!-- User Display (when logged in) -->
<div id="user-display" style="display: none;"></div>

<!-- Login Link (when not logged in) -->
<a id="login-link" href="login.html">
  Accedi
</a>

<!-- Logout Button (when logged in) -->
<button id="logout-btn" style="display: none;">
  Esci
</button>
```

---

## 🔄 Flusso Completo

### Login

```
1. Utente fa login
   ↓
2. Backend restituisce token + user
   ↓
3. Salva in localStorage:
   - authToken
   - currentUser
   ↓
4. Redirect a dashboard
   ↓
5. auth.js carica automaticamente
   ↓
6. Mostra user display + logout button
```

### Refresh Pagina

```
1. Pagina ricaricata
   ↓
2. auth.js inizializza AuthManager
   ↓
3. Legge token da localStorage
   ↓
4. Verifica con backend
   ↓
5. Se valido: mostra user display
   Se invalido: logout automatico
```

### Logout

```
1. Click su "Esci"
   ↓
2. authManager.logout()
   ↓
3. POST /api/auth/logout/{token}
   ↓
4. Rimuove da localStorage:
   - authToken
   - currentUser
   - rememberMe
   ↓
5. Redirect a login.html
```

---

## 🧪 Test

### Test Persistenza

1. **Login:**
```
http://localhost:3000/login.html
```

2. **Verifica UI:**
- ✅ Vedi nome utente in alto
- ✅ Vedi pulsante "Esci"
- ✅ NO pulsante "Accedi"

3. **Refresh pagina (F5)**
- ✅ Ancora loggato
- ✅ Nome utente ancora visibile
- ✅ Pulsante "Esci" ancora presente

4. **Chiudi e riapri browser**
- ✅ Ancora loggato
- ✅ Sessione mantenuta

### Test Logout

1. **Click "Esci"**
- ✅ Redirect a login.html
- ✅ localStorage pulito
- ✅ Sessione terminata

2. **Torna a homepage**
- ✅ Vedi "Accedi"
- ✅ NO user display
- ✅ NO pulsante "Esci"

---

## 🔐 Sicurezza

### Token Validation

```javascript
async verifySession() {
    const response = await fetch(`/api/auth/me/${this.token}`);
    
    if (!response.ok) {
        // Token invalido/scaduto
        this.logout();
        return false;
    }
    
    // Token valido, aggiorna user
    const user = await response.json();
    localStorage.setItem('currentUser', JSON.stringify(user));
    return true;
}
```

### Auto-Logout

Se il token è scaduto o invalido:
- Verifica fallisce
- Logout automatico
- Redirect a login

---

## 📊 localStorage

### Dati Salvati

```javascript
{
  "authToken": "uuid-token-here",
  "currentUser": {
    "id": "user-id",
    "name": "Mario Rossi",
    "email": "mario@example.com",
    "created_at": 1234567890
  },
  "rememberMe": "true" // opzionale
}
```

### Pulizia

Al logout, tutto viene rimosso:
```javascript
localStorage.removeItem('authToken');
localStorage.removeItem('currentUser');
localStorage.removeItem('rememberMe');
```

---

## 🎯 Vantaggi

### User Experience ✅

- **Persistenza**: Non devi rifare login ogni volta
- **Seamless**: Refresh non interrompe sessione
- **Chiaro**: Sempre visibile se sei loggato
- **Facile**: Logout con un click

### Sicurezza ✅

- **Verifica**: Token verificato ad ogni caricamento
- **Auto-logout**: Se token invalido
- **Pulizia**: localStorage pulito al logout

---

## 📁 File Coinvolti

### auth.js (Esistente)
- `AuthManager` class
- Gestione token
- Verifica sessione
- Update UI

### index.html (Modificato)
- Aggiunto `user-display`
- Aggiunto `login-link`
- Aggiunto `logout-btn`

### Altre pagine (TODO)
Aggiungere stessi elementi a:
- dashboard.html
- participate.html
- manage.html
- create-poll.html

---

## ✅ Checklist

- [x] Sessione persistente con localStorage
- [x] Verifica automatica al caricamento
- [x] User display quando loggato
- [x] Pulsante logout visibile
- [x] Logout funzionante
- [x] Auto-logout se token invalido
- [x] Implementato in index.html
- [ ] TODO: Implementare in altre pagine

---

## 🚀 Prossimi Passi

### 1. Aggiungere a Tutte le Pagine

Copiare gli stessi elementi in:
- `dashboard.html`
- `participate.html`
- `manage.html`
- `create-poll.html`

### 2. Migliorare UI

- Dropdown menu utente
- Link a profilo
- Impostazioni rapide

### 3. Remember Me

- Checkbox al login
- Token long-lived
- Persistenza estesa

---

**Status**: ✅ IMPLEMENTATO  
**Persistenza**: Funzionante  
**Logout**: Funzionante  

🎉 **SESSIONE PERSISTENTE E LOGOUT OPERATIVI!** ✨

Ora la sessione viene mantenuta e puoi fare logout quando vuoi!
