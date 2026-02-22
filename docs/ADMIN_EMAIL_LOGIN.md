# Modifica Sistema Login Admin - Uso Email

## ✅ Completato

Il sistema di login amministratore è stato modificato per utilizzare **email** invece di **username**, rendendolo coerente con il resto dell'applicazione.

---

## Modifiche Implementate

### 1. Modello LoginRequest ✅
**File:** `src/models.rs:65-69`

```rust
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,     // ← Modificato da 'username' a 'email'
    pub password: String,
}
```

**Cambio:** Il campo `username` è stato sostituito con `email`.

---

### 2. Handler admin_login ✅
**File:** `src/handlers.rs:526-569`

```rust
pub async fn admin_login(
    State(pool): State<DbPool>,
    Json(payload): Json<models::LoginRequest>,
) -> Result<Json<models::AuthResponse>, (StatusCode, String)> {
    // Validate inputs
    validate_email(&payload.email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_string_length(&payload.password, 128, "Password")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // 1. Fetch admin by email (non più per username)
    let admin: models::Admin = sqlx::query_as("SELECT * FROM admins WHERE email = ?")
        .bind(&payload.email)  // ← Cerca per email
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),  // ← Messaggio aggiornato
        ))?;

    // 2. Verify password
    let valid = bcrypt::verify(&payload.password, &admin.password_hash).unwrap_or(false);

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),
        ));
    }

    // 3. Create session
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now().timestamp() + 86400; // 24 hours

    sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&token)
        .bind(&admin.id)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(models::AuthResponse { token, user: admin }))
}
```

**Modifiche:**
- Query SQL cerca per `email` invece di `username`
- Validazione dell'email invece del username
- Messaggi di errore aggiornati da "Invalid username or password" a "Invalid email or password"

---

### 3. Creazione Admin di Default ✅
**File:** `src/db.rs:98-121`

```rust
// Check if default admin exists, if not create one
// Password: "password123"
let default_admin_exists: bool =
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM admins WHERE email = 'admin@example.com')")
        .fetch_one(&pool)
        .await?;

if !default_admin_exists {
    let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
    let admin_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO admins (id, username, password_hash, email, role, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(admin_id)
    .bind("admin")
    .bind(password_hash)
    .bind("admin@example.com")  // ← Email dell'admin di default
    .bind("superadmin")
    .bind(now)
    .execute(&pool)
    .await?;
}
```

**Verifica:** Il controllo per l'esistenza dell'admin usa `WHERE email = 'admin@example.com'`

---

## Test Effettuati

### ✅ Test 1: Login con Email (Nuovo Sistema)
```bash
curl -X POST http://localhost:3000/api/admin/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "password123"}'
```

**Risultato:**
```json
{
  "token": "f99fc482-1d7a-4038-81d4-ee6bf050d199",
  "user": {
    "id": "6426bf05-98b7-40db-9d7a-9d31309ae101",
    "username": "admin",
    "email": "admin@example.com",
    "role": "super_admin",
    "created_at": 1763583038
  }
}
```

**HTTP Status:** `200 OK` ✅

---

### ✅ Test 2: Login con Username (Vecchio Sistema)
```bash
curl -X POST http://localhost:3000/api/admin/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}'
```

**Risultato:**
```
Failed to deserialize the JSON body into the target type: missing field `email` at line 1 column 48
```

**HTTP Status:** `422 Unprocessable Entity` ✅

**Spiegazione:** Il campo `username` non è più accettato, viene richiesto il campo `email`.

---

## Credenziali Admin di Default

### 🔑 Nuove Credenziali
- **Email:** `admin@example.com`
- **Password:** `password123`

### ❌ Vecchie Credenziali (NON PIÙ VALIDE)
- ~~**Username:** `admin`~~
- ~~**Password:** `password123`~~

---

## Uso nell'Applicazione

### Formato Richiesta Login Admin

**Endpoint:** `POST /api/admin/login`

**Headers:**
```
Content-Type: application/json
```

**Body:**
```json
{
  "email": "admin@example.com",
  "password": "password123"
}
```

**Risposta (Success - 200 OK):**
```json
{
  "token": "uuid-token-here",
  "user": {
    "id": "admin-id",
    "username": "admin",
    "email": "admin@example.com",
    "role": "super_admin",
    "created_at": 1234567890
  }
}
```

**Risposta (Error - 401 Unauthorized):**
```json
{
  "error": "Invalid email or password"
}
```

---

## Coerenza con il Resto dell'Applicazione

Ora **tutti i sistemi di autenticazione** usano **email** come identificatore:

| Sistema | Campo Login | Endpoint |
|---------|-------------|----------|
| User Registration | `email` | `POST /api/auth/register` |
| User Login | `email` | `POST /api/auth/login` |
| **Admin Login** | **`email`** | `POST /api/admin/login` |
| Google Login | `email` | `POST /api/admin/google-login` |

✅ **Sistema completamente coerente!**

---

## Note Importanti

1. **Retrocompatibilità:** Il vecchio sistema con `username` **non è più supportato**. Tutte le richieste devono usare `email`.

2. **Frontend:** Assicurarsi di aggiornare tutti i form di login admin per richiedere `email` invece di `username`.

3. **Database:** La tabella `admins` mantiene ancora il campo `username` per compatibilità, ma il login avviene esclusivamente tramite `email`.

4. **Validazione:** L'email viene validata con controlli di formato e lunghezza prima dell'autenticazione.

---

## File di Test

**Script:** `test_admin_email_login.sh`

Per eseguire i test:
```bash
./test_admin_email_login.sh
```

Il script verifica:
- ✅ Login con email funziona correttamente
- ✅ Login con username viene rifiutato
- ✅ Token viene generato correttamente

---

## Sommario

| Componente | Stato | Note |
|-----------|-------|------|
| Modello LoginRequest | ✅ Modificato | Campo `email` invece di `username` |
| Handler admin_login | ✅ Aggiornato | Cerca admin per email |
| Creazione admin default | ✅ Verificato | Usa email `admin@example.com` |
| Test funzionali | ✅ Passati | Login email OK, username rifiutato |
| Coerenza sistema | ✅ Completa | Tutti i login usano email |

**🎉 Modifica completata con successo!**
