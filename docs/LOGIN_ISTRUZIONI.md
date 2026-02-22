# Istruzioni per il Login

## 🔐 Account Esistente Trovato

**Email:** `tiziano.digennaro@gmail.com`

Questo account è già registrato nel sistema. Puoi fare il login invece di registrarti di nuovo.

---

## 📱 Come Fare il Login

### **Metodo 1: Tramite Browser (Consigliato)**

1. **Apri il browser** e vai su:
   ```
   http://localhost:3000/login.html
   ```

2. **Inserisci le credenziali:**
   - **Email:** `tiziano.digennaro@gmail.com`
   - **Password:** (la password che hai usato quando hai creato l'account)

3. **Clicca su "Sign in"**

4. Se il login ha successo, verrai reindirizzato alla dashboard

---

### **Metodo 2: Tramite API (Per test)**

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "tiziano.digennaro@gmail.com",
    "password": "LA_TUA_PASSWORD_QUI"
  }'
```

**Risposta in caso di successo:**
```json
{
  "token": "uuid-token-qui",
  "user": {
    "id": "user-id",
    "email": "tiziano.digennaro@gmail.com",
    "name": "Tuo Nome",
    "role": "player",
    "created_at": 1234567890
  }
}
```

---

## ❓ Non Ricordi la Password?

Se non ricordi la password, hai due opzioni:

### **Opzione A: Eliminare e Ricreare l'Account**

1. Installa sqlite3:
   ```bash
   sudo apt-get install sqlite3
   ```

2. Elimina l'account esistente:
   ```bash
   cd "/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App"
   sqlite3 dnd_scheduler.db "DELETE FROM users WHERE email = 'tiziano.digennaro@gmail.com';"
   ```

3. Ora puoi registrarti di nuovo su:
   ```
   http://localhost:3000/register.html
   ```

### **Opzione B: Usare un'altra Email**

Registrati con un'email diversa, ad esempio:
- `tiziano.digennaro+dnd@gmail.com`
- `tiziano.digennaro+test@gmail.com`

(Gmail ignora il "+qualcosa" ma il sistema li considera email diverse)

---

## 📋 Informazioni sul Ruolo

Quando ti registri, il tuo account ha il ruolo **"player"** di default.

### Se hai bisogno di creare sessioni D&D:

Devi essere promosso a **"dm"** (Dungeon Master):

```bash
# Installa sqlite3 (se non l'hai già fatto)
sudo apt-get install sqlite3

# Promuovi l'utente a DM
cd "/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App"
sqlite3 dnd_scheduler.db "UPDATE users SET role = 'dm' WHERE email = 'tiziano.digennaro@gmail.com';"
```

**Importante:** Dopo la promozione, devi fare **logout e login** di nuovo per ottenere un nuovo token con il ruolo aggiornato.

---

## ✅ Verifica Ruolo

Dopo il login, puoi verificare il tuo ruolo:

```bash
# Usa il token ricevuto dal login
curl http://localhost:3000/api/auth/me/TUO_TOKEN_QUI
```

Risposta:
```json
{
  "id": "user-id",
  "email": "tiziano.digennaro@gmail.com",
  "name": "Tuo Nome",
  "role": "player",  // o "dm" se sei stato promosso
  "created_at": 1234567890
}
```

---

## 🎮 Cosa Puoi Fare

### Con Ruolo "player":
- ✅ Visualizzare le sessioni
- ✅ Partecipare alle sessioni
- ✅ Indicare la tua disponibilità
- ❌ **NON puoi creare nuove sessioni**

### Con Ruolo "dm":
- ✅ Tutto quello che può fare un player
- ✅ **Creare nuove sessioni**
- ✅ Gestire le campagne

---

## 🆘 Problemi?

Se continui ad avere problemi:

1. **Verifica che il server sia in esecuzione:**
   ```bash
   ss -tlnp | grep :3000
   ```

2. **Controlla i log del server:**
   ```bash
   cd "/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App"
   tail -f server.log
   ```

3. **Prova a riavviare il server:**
   ```bash
   pkill -f dnd_scheduler
   cd "/home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App"
   ./target/debug/dnd_scheduler
   ```

---

## 📝 Riepilogo Rapido

1. **Vai su:** `http://localhost:3000/login.html`
2. **Inserisci:**
   - Email: `tiziano.digennaro@gmail.com`
   - Password: (la tua password)
3. **Clicca "Sign in"**
4. **Se necessario DM:** Promuovi il tuo account (vedi sopra)

Buon gioco! 🎲
