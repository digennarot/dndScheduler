# 🔒 OWASP TOP 10 2021 - SECURITY AUDIT

## Data: 2025-12-06
## Applicazione: D&D Scheduler

---

## 📋 OWASP TOP 10 2021 CHECKLIST

### A01:2021 – Broken Access Control

#### ❌ VULNERABILITÀ CRITICHE TROVATE

**1. Google OAuth Token Non Verificato**
```rust
// src/auth.rs - google_login()
// ⚠️ CRITICO: Il token Google NON viene verificato!
pub async fn google_login(
    State(pool): State<DbPool>,
    Json(payload): Json<GoogleLoginRequest>,
) -> Response {
    // TODO: Verify token with Google API
    // ATTUALMENTE: Accetta qualsiasi token senza verifica!
```

**Rischio:** Chiunque può creare un token falso e autenticarsi come admin.

**Fix Necessario:**
```rust
// Verificare token con Google
let client = reqwest::Client::new();
let response = client
    .get("https://oauth2.googleapis.com/tokeninfo")
    .query(&[("id_token", &payload.token)])
    .send()
    .await?;

if !response.status().is_success() {
    return json_error(StatusCode::UNAUTHORIZED, "Invalid token");
}
```

**2. Nessuna Autorizzazione su API Admin**
```rust
// Manca middleware di autorizzazione
// Chiunque può chiamare /api/admin/* se conosce l'endpoint
```

**Fix Necessario:**
- Implementare middleware di autenticazione
- Verificare ruolo admin prima di ogni operazione

**3. Session Token Non Scade**
```rust
// I token di sessione non hanno scadenza
// Un token rubato funziona per sempre
```

**Fix Necessario:**
- Aggiungere campo `expires_at` alla tabella sessions
- Verificare scadenza ad ogni richiesta

#### ✅ IMPLEMENTAZIONI CORRETTE

- ✅ Password hashing con bcrypt
- ✅ Validazione email
- ✅ Sanitizzazione input

---

### A02:2021 – Cryptographic Failures

#### ✅ IMPLEMENTAZIONI CORRETTE

```rust
// src/auth.rs
use bcrypt::{hash, verify, DEFAULT_COST};

let hashed = hash(password, DEFAULT_COST)?;
```

- ✅ Password hashate con bcrypt (cost 12)
- ✅ Salt automatico
- ✅ Algoritmo sicuro

#### ⚠️ MIGLIORAMENTI CONSIGLIATI

**1. HTTPS Non Forzato**
```rust
// TODO: Forzare HTTPS in produzione
// Aggiungere middleware redirect HTTP → HTTPS
```

**2. Nessuna Crittografia Database**
```sql
-- I dati sensibili non sono crittografati nel DB
-- Email, nomi utente in chiaro
```

**Consiglio:** Crittografare dati PII (email, nomi) a riposo.

---

### A03:2021 – Injection

#### ✅ IMPLEMENTAZIONI CORRETTE

```rust
// Uso di SQLx con query parametrizzate
sqlx::query!(
    "INSERT INTO users (id, email, password_hash, name) VALUES (?, ?, ?, ?)",
    user_id, email, hashed_password, name
)
```

- ✅ Query parametrizzate (protezione SQL Injection)
- ✅ Nessun concatenamento stringhe in query

#### ⚠️ MIGLIORAMENTI CONSIGLIATI

**1. Sanitizzazione Input Limitata**
```rust
// src/handlers.rs
fn sanitize_string(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?-_".contains(*c))
        .collect()
}
```

**Manca:**
- Validazione lunghezza massima
- Protezione contro Unicode malformato
- Sanitizzazione HTML

**Fix:**
```rust
fn sanitize_string(s: &str) -> Result<String, String> {
    if s.len() > 1000 {
        return Err("Input too long".to_string());
    }
    
    // HTML escape
    let escaped = html_escape::encode_text(s);
    Ok(escaped.to_string())
}
```

---

### A04:2021 – Insecure Design

#### ❌ PROBLEMI TROVATI

**1. Nessun Rate Limiting**
```rust
// Nessuna protezione contro brute force
// Un attacker può provare infinite password
```

**Fix Necessario:**
```rust
// Aggiungere rate limiting middleware
use tower::limit::RateLimitLayer;

let app = Router::new()
    .layer(RateLimitLayer::new(
        100, // max requests
        Duration::from_secs(60) // per minute
    ));
```

**2. Nessuna Protezione CSRF**
```html
<!-- Manca CSRF token nei form -->
<form method="POST">
  <!-- Nessun token CSRF -->
</form>
```

**Fix Necessario:**
```rust
// Aggiungere CSRF protection
use axum_csrf::CsrfLayer;

let app = Router::new()
    .layer(CsrfLayer::new());
```

**3. Password Temporanea Debole**
```javascript
// admin-manager.js
password: 'TempPassword123!' // Sempre uguale!
```

**Fix Necessario:**
```javascript
// Generare password random
const password = crypto.randomBytes(16).toString('hex');
```

---

### A05:2021 – Security Misconfiguration

#### ❌ PROBLEMI TROVATI

**1. CORS Troppo Permissivo**
```rust
// src/main.rs
.layer(
    CorsLayer::new()
        .allow_origin(Any) // ⚠️ Permette qualsiasi origine!
        .allow_methods(Any)
        .allow_headers(Any)
)
```

**Fix:**
```rust
.layer(
    CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
)
```

**2. Errori Dettagliati in Produzione**
```rust
// Gli errori rivelano dettagli implementazione
Err(e) => {
    eprintln!("Database error: {}", e); // Log dettagliato
    (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
}
```

**Fix:**
```rust
Err(e) => {
    eprintln!("Database error: {}", e); // Solo in log
    (StatusCode::INTERNAL_SERVER_ERROR, "An error occurred").into_response()
    // Messaggio generico all'utente
}
```

**3. Nessun Security Headers**

**Mancano:**
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Content-Security-Policy`
- `Strict-Transport-Security`

**Fix:**
```rust
use tower_http::set_header::SetResponseHeaderLayer;

let app = Router::new()
    .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff")
    ))
    .layer(SetResponseHeaderLayer::overriding(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY")
    ));
```

---

### A06:2021 – Vulnerable and Outdated Components

#### ✅ STATO ATTUALE

```toml
# Cargo.toml
axum = "0.7"           # ✅ Recente
tokio = "1.0"          # ✅ Recente
sqlx = "0.7"           # ✅ Recente
bcrypt = "0.15"        # ✅ Recente
```

#### ⚠️ RACCOMANDAZIONI

**1. Audit Dipendenze**
```bash
cargo audit
```

**2. Update Regolari**
```bash
cargo update
```

**3. Monitoraggio CVE**
- Iscriversi a security advisories
- Usare Dependabot/Renovate

---

### A07:2021 – Identification and Authentication Failures

#### ❌ PROBLEMI CRITICI

**1. Nessuna Politica Password**
```rust
// Validazione password troppo debole
if password.len() < 8 {
    return Err("Password must be at least 8 characters");
}
// Manca: complessità, caratteri speciali, numeri
```

**Fix:**
```rust
fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 12 {
        return Err("Password must be at least 12 characters".to_string());
    }
    
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "!@#$%^&*".contains(c));
    
    if !(has_uppercase && has_lowercase && has_digit && has_special) {
        return Err("Password must contain uppercase, lowercase, digit, and special character".to_string());
    }
    
    Ok(())
}
```

**2. Nessun Blocco Account**
```rust
// Dopo N tentativi falliti, l'account dovrebbe bloccarsi
// ATTUALMENTE: Tentativi infiniti
```

**3. Nessun 2FA**
- Manca autenticazione a due fattori
- Consigliato per admin

**4. Session Fixation**
```rust
// Il session token non viene rigenerato dopo login
// Vulnerabile a session fixation attacks
```

---

### A08:2021 – Software and Data Integrity Failures

#### ⚠️ PROBLEMI TROVATI

**1. Nessuna Verifica Integrità File**
```html
<!-- CDN senza SRI -->
<script src="https://cdn.tailwindcss.com"></script>
<!-- Manca: integrity="sha384-..." -->
```

**Fix:**
```html
<script 
  src="https://cdn.tailwindcss.com"
  integrity="sha384-..."
  crossorigin="anonymous">
</script>
```

**2. Nessun Controllo Versione Assets**
```html
<script src="js/app.js"></script>
<!-- Vulnerabile a cache poisoning -->
```

**Fix:**
```html
<script src="js/app.js?v=1.0.0"></script>
```

---

### A09:2021 – Security Logging and Monitoring Failures

#### ❌ PROBLEMI TROVATI

**1. Logging Insufficiente**
```rust
// Mancano log per:
// - Tentativi login falliti
// - Modifiche dati sensibili
// - Accessi admin
// - Errori di autorizzazione
```

**Fix:**
```rust
use tracing::{info, warn, error};

// Login fallito
warn!("Failed login attempt for email: {}", email);

// Accesso admin
info!("Admin access: user {} accessed {}", user_id, endpoint);

// Modifica dati
info!("User {} modified poll {}", user_id, poll_id);
```

**2. Nessun Monitoraggio Anomalie**
- Manca detection tentativi brute force
- Manca alerting su attività sospette

**3. Log Non Protetti**
```rust
// I log potrebbero contenere dati sensibili
eprintln!("Error: {:?}", user); // Potrebbe loggare password!
```

---

### A10:2021 – Server-Side Request Forgery (SSRF)

#### ✅ BASSO RISCHIO

L'applicazione non fa molte richieste server-side a URL forniti dall'utente.

#### ⚠️ POTENZIALI RISCHI

**1. Reminder Service**
```rust
// src/reminder_service.rs
// Se l'URL Twilio/Telegram fosse configurabile dall'utente
// potrebbe essere vulnerabile a SSRF
```

**Mitigazione:**
- URL hardcoded
- Whitelist domini permessi

---

## 📊 RIEPILOGO VULNERABILITÀ

### 🔴 CRITICHE (Fix Immediato)

1. **Google OAuth Non Verificato** - A01
2. **Nessuna Autorizzazione Admin** - A01
3. **CORS Troppo Permissivo** - A05
4. **Nessun Rate Limiting** - A04

### 🟡 ALTE (Fix Prioritario)

5. **Session Token Non Scade** - A01
6. **Nessuna Protezione CSRF** - A04
7. **Password Policy Debole** - A07
8. **Nessun Security Headers** - A05

### 🟢 MEDIE (Fix Consigliato)

9. **Logging Insufficiente** - A09
10. **Nessun SRI su CDN** - A08
11. **Sanitizzazione Limitata** - A03
12. **Password Temp Statica** - A04

---

## ✅ PUNTI DI FORZA

1. ✅ Password hashing con bcrypt
2. ✅ Query parametrizzate (no SQL injection)
3. ✅ Dipendenze aggiornate
4. ✅ Validazione input base
5. ✅ Separazione frontend/backend

---

## 🎯 PIANO D'AZIONE PRIORITARIO

### Fase 1: CRITICHE (1-2 giorni)

```rust
// 1. Verificare Google OAuth
// 2. Aggiungere middleware auth admin
// 3. Configurare CORS correttamente
// 4. Implementare rate limiting
```

### Fase 2: ALTE (3-5 giorni)

```rust
// 5. Aggiungere scadenza token
// 6. Implementare CSRF protection
// 7. Migliorare password policy
// 8. Aggiungere security headers
```

### Fase 3: MEDIE (1 settimana)

```rust
// 9. Migliorare logging
// 10. Aggiungere SRI
// 11. Migliorare sanitizzazione
// 12. Password random
```

---

## 📝 SCORE OWASP

**Stato Attuale:** 4/10 ⚠️

**Dopo Fix Critiche:** 6/10 🟡

**Dopo Tutti i Fix:** 8.5/10 ✅

---

## 🔒 CONCLUSIONE

L'applicazione ha **buone basi** (bcrypt, query parametrizzate) ma presenta **vulnerabilità critiche** che devono essere risolte prima del deployment in produzione.

**Priorità Assoluta:**
1. Verificare token Google OAuth
2. Implementare autorizzazione admin
3. Configurare CORS
4. Rate limiting

**Status:** ⚠️ NON PRONTO PER PRODUZIONE

Dopo i fix critici: ✅ PRONTO PER PRODUZIONE
