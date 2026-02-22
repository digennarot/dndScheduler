# ✅ FIX: JSON ERROR RESPONSES

## Data: 2025-12-06

---

## 🐛 Problema Risolto

### Errore Iniziale
```
Unexpected token 'P', "Password m"... is not valid JSON
```

**Causa**: Il backend restituiva errori come testo semplice invece di JSON.

Quando c'era un errore (es. password troppo corta), il server restituiva:
```
Password must be at least 8 characters
```

Ma il frontend si aspettava JSON:
```json
{
  "error": "Password must be at least 8 characters"
}
```

---

## 🔧 Soluzione Applicata

### 1. Aggiunta Struct ErrorResponse

```rust
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
```

### 2. Helper Function

```rust
fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(ErrorResponse { error: message.into() })).into_response()
}
```

### 3. Modificata Funzione `register()`

**Prima:**
```rust
pub async fn register(...) -> Result<Json<UserAuthResponse>, (StatusCode, String)> {
    validate_email(&payload.email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    // Restituiva: "Invalid email format" (testo)
}
```

**Dopo:**
```rust
pub async fn register(...) -> Response {
    if let Err(e) = validate_email(&payload.email) {
        return json_error(StatusCode::BAD_REQUEST, e);
    }
    // Restituisce: {"error": "Invalid email format"} (JSON)
}
```

---

## 📊 Esempi Risposta

### Errore Email Invalida

**Request:**
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "invalid",
    "password": "password123",
    "name": "Test User"
  }'
```

**Response (Prima):** ❌
```
Invalid email format
```

**Response (Dopo):** ✅
```json
{
  "error": "Invalid email format"
}
```

### Errore Password Troppo Corta

**Request:**
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "123",
    "name": "Test User"
  }'
```

**Response:** ✅
```json
{
  "error": "Password must be at least 8 characters"
}
```

### Errore Email Già Registrata

**Request:**
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "existing@example.com",
    "password": "password123",
    "name": "Test User"
  }'
```

**Response:** ✅
```json
{
  "error": "Email already registered"
}
```

### Successo

**Request:**
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com",
    "password": "password123",
    "name": "New User"
  }'
```

**Response:** ✅
```json
{
  "token": "uuid-token-here",
  "user": {
    "id": "user-uuid",
    "email": "newuser@example.com",
    "name": "New User",
    "created_at": 1733508000
  }
}
```

---

## 🧪 Test

### 1. Test Email Invalida
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"bad","password":"password123","name":"Test"}'
```

**Aspettato:**
```json
{"error":"Invalid email format"}
```

### 2. Test Password Corta
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"123","name":"Test"}'
```

**Aspettato:**
```json
{"error":"Password must be at least 8 characters"}
```

### 3. Test Registrazione Valida
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"password123","name":"Test User"}'
```

**Aspettato:**
```json
{
  "token": "...",
  "user": {...}
}
```

---

## 📝 File Modificati

### src/auth.rs
- ✅ Aggiunta `ErrorResponse` struct
- ✅ Aggiunta `json_error()` helper
- ✅ Modificata `register()` per usare `Response`
- ✅ Convertiti tutti gli errori a JSON

---

## ✅ Risultato

### Prima ❌
```javascript
// Frontend
fetch('/api/auth/register', {...})
  .then(r => r.json())
  .catch(e => {
    // Error: Unexpected token 'P'
    // Perché riceve testo invece di JSON
  })
```

### Dopo ✅
```javascript
// Frontend
fetch('/api/auth/register', {...})
  .then(r => r.json())
  .then(data => {
    if (data.error) {
      // Gestisce errore JSON correttamente
      console.error(data.error);
    } else {
      // Successo
      console.log(data.user);
    }
  })
```

---

## 🔄 Prossimi Passi

### Altre Funzioni da Aggiornare

Le stesse modifiche dovrebbero essere applicate a:
- ✅ `register()` - FATTO
- ⏳ `login()` - Da fare
- ⏳ `logout()` - Da fare  
- ⏳ `get_current_user()` - Da fare

Ma per ora `register()` è il più importante perché è quello che stavi testando.

---

## 🎯 Status

**Problema**: ✅ RISOLTO  
**Server**: ✅ Riavviato  
**Compilazione**: ✅ Pulita  
**Test**: ✅ Pronto  

🎉 **ERRORI JSON RISOLTI!** ✨

Ora prova a registrarti dalla pagina `/register.html` - gli errori saranno visualizzati correttamente!
