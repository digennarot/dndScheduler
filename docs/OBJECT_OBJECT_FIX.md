# ✅ FIX: [object Object] Error Display

## Data: 2025-12-06

---

## 🐛 Problema

Quando si tenta di creare un account con dati invalidi, viene mostrato:
```
[object Object]
```

Invece del messaggio di errore leggibile.

---

## 🔍 Causa

### Backend
Il backend ora restituisce errori in questo formato:
```json
{
  "error": "Password must be at least 8 characters"
}
```

### Frontend (register.html)
Il JavaScript cercava il campo sbagliato:
```javascript
// PRIMA (sbagliato)
throw new Error(data.message || data || 'Registration failed');
//               ^^^^^^^^^^^^
//               Campo inesistente!
```

Quando `data.message` è `undefined`, JavaScript prova a usare `data` (l'intero oggetto), che viene convertito in stringa come `[object Object]`.

---

## ✅ Soluzione

### Modificato register.html

```javascript
// DOPO (corretto)
throw new Error(data.error || data.message || 'Registration failed');
//               ^^^^^^^^^^
//               Campo corretto!
```

Ora cerca prima `data.error` (il campo che il backend restituisce), poi `data.message` (per compatibilità), poi il messaggio di default.

---

## 📊 Esempi

### Errore Password Corta

**Backend restituisce:**
```json
{
  "error": "Password must be at least 8 characters"
}
```

**Frontend mostra:**
```
Password must be at least 8 characters
```
✅ Corretto!

### Errore Email Invalida

**Backend restituisce:**
```json
{
  "error": "Invalid email format"
}
```

**Frontend mostra:**
```
Invalid email format
```
✅ Corretto!

### Errore Email Già Registrata

**Backend restituisce:**
```json
{
  "error": "Email already registered"
}
```

**Frontend mostra:**
```
Email already registered
```
✅ Corretto!

---

## 🧪 Test

### 1. Email Invalida
1. Vai su http://localhost:3000/register.html
2. Inserisci:
   - Nome: Test
   - Email: `invalid` (senza @)
   - Password: password123
   - Conferma: password123
3. Click "Crea Account"

**Risultato atteso:**
```
Invalid email format
```

### 2. Password Troppo Corta
1. Vai su http://localhost:3000/register.html
2. Inserisci:
   - Nome: Test
   - Email: test@test.com
   - Password: `123` (troppo corta)
   - Conferma: 123
3. Click "Crea Account"

**Risultato atteso:**
```
Password must be at least 8 characters
```

### 3. Password Non Corrispondenti
1. Vai su http://localhost:3000/register.html
2. Inserisci:
   - Nome: Test
   - Email: test@test.com
   - Password: password123
   - Conferma: password456 (diversa)
3. Click "Crea Account"

**Risultato atteso:**
```
Passwords do not match
```

### 4. Registrazione Valida
1. Vai su http://localhost:3000/register.html
2. Inserisci:
   - Nome: Test User
   - Email: test@test.com
   - Password: password123
   - Conferma: password123
3. Click "Crea Account"

**Risultato atteso:**
- Redirect a index.html
- Utente loggato

---

## 📝 File Modificati

### static/register.html
**Riga 177:**
```javascript
// Prima
throw new Error(data.message || data || 'Registration failed');

// Dopo
throw new Error(data.error || data.message || 'Registration failed');
```

---

## 🔄 Compatibilità

La soluzione è retrocompatibile:
- ✅ Funziona con `data.error` (nuovo formato)
- ✅ Funziona con `data.message` (vecchio formato, se esiste)
- ✅ Fallback a messaggio generico

---

## ✅ Risultato

### Prima ❌
```
[object Object]
```
Messaggio incomprensibile

### Dopo ✅
```
Password must be at least 8 characters
```
Messaggio chiaro e utile

---

## 🎯 Altri File da Verificare

Controllare se altri file hanno lo stesso problema:
- ✅ register.html - RISOLTO
- ⏳ login.html - Da verificare (sembra OK)
- ⏳ create-poll.html - Da verificare
- ⏳ participate.html - Da verificare
- ⏳ manage.html - Da verificare

---

**Status**: ✅ RISOLTO  
**File**: register.html  
**Test**: ✅ Pronto  

🎉 **ERRORI VISUALIZZATI CORRETTAMENTE!** ✨

Ora prova a registrarti con dati invalidi - vedrai messaggi di errore chiari!
