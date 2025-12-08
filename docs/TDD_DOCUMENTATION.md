# Test-Driven Development (TDD) - Documentazione Completa

## 📋 Indice
1. [Panoramica](#panoramica)
2. [Struttura dei Test](#struttura-dei-test)
3. [Esecuzione dei Test](#esecuzione-dei-test)
4. [Test Implementati](#test-implementati)
5. [Best Practices](#best-practices)
6. [CI/CD Integration](#cicd-integration)

---

## 🎯 Panoramica

Il progetto D&D Scheduler implementa una strategia completa di **Test-Driven Development (TDD)** con:

- ✅ **Test Unitari** - Test delle singole funzioni e moduli
- ✅ **Test di Integrazione** - Test delle interazioni tra componenti
- ✅ **Test RBAC** - Test del sistema di controllo accessi
- ✅ **Test Helper Utilities** - Funzioni di supporto per i test

### Vantaggi del TDD in questo progetto:
- 🔒 **Sicurezza**: Verifica che RBAC funzioni correttamente
- 🐛 **Bug Detection**: Trova bug prima del deployment
- 📚 **Documentazione**: I test documentano il comportamento atteso
- 🔄 **Refactoring Sicuro**: Modifica il codice con confidenza

---

## 📁 Struttura dei Test

```
tests/
├── integration/
│   ├── main.rs              # Entry point dei test di integrazione
│   ├── helpers.rs           # Utilities e helpers per i test
│   ├── auth_tests.rs        # Test autenticazione
│   └── rbac_tests.rs        # Test Role-Based Access Control
└── (unit tests in src/)     # Test unitari nei moduli source

Cargo.toml                   # Dipendenze di test
run_tests.sh                 # Script runner per test
```

### File Principali

#### `tests/integration/helpers.rs`
Contiene:
- `setup_test_db()` - Crea database temporaneo per i test
- `create_test_user()` - Crea utente di test
- `create_test_session()` - Crea sessione di test
- `cleanup_test_db()` - Pulisce il database dopo i test

#### `tests/integration/auth_tests.rs`
Test per:
- Registrazione utenti
- Login/Logout
- Validazione password
- Gestione sessioni

#### `tests/integration/rbac_tests.rs`
Test per:
- Assegnazione ruoli
- Verifica permessi DM
- Restrizioni player
- Persistenza ruoli

---

## 🚀 Esecuzione dei Test

### Metodo 1: Script Runner (Consigliato)

```bash
# Tutti i test
./run_tests.sh all

# Solo test di autenticazione
./run_tests.sh auth

# Solo test RBAC
./run_tests.sh rbac

# Test con coverage
./run_tests.sh coverage

# Modalità watch (riesegue test ad ogni modifica)
./run_tests.sh watch

# Pulizia file temporanei
./run_tests.sh clean

# Help
./run_tests.sh help
```

### Metodo 2: Cargo Diretto

```bash
# Tutti i test
cargo test

# Solo test unitari
cargo test --lib

# Solo test di integrazione
cargo test --test integration

# Test specifico
cargo test test_user_registration_with_valid_data

# Test con output dettagliato
cargo test -- --nocapture

# Test in parallelo (default)
cargo test -- --test-threads=4
```

### Metodo 3: Test Singoli Moduli

```bash
# Test autenticazione
cargo test --test integration auth_integration_tests

# Test RBAC
cargo test --test integration rbac_integration_tests

# Test helpers
cargo test --lib helpers
```

---

## 📝 Test Implementati

### 🔐 Test di Autenticazione (`auth_tests.rs`)

| Test | Descrizione | Stato |
|------|-------------|-------|
| `test_user_registration_with_valid_data` | Verifica registrazione con dati validi | ✅ |
| `test_user_cannot_register_with_duplicate_email` | Verifica rifiuto email duplicate | ✅ |
| `test_user_login_with_valid_credentials` | Verifica login con credenziali corrette | ✅ |
| `test_user_login_with_invalid_password` | Verifica rifiuto password errata | ✅ |
| `test_session_creation_and_validation` | Verifica creazione e validazione sessioni | ✅ |
| `test_logout_invalidates_session` | Verifica che logout invalidi la sessione | ✅ |
| `test_password_strength_requirements` | Verifica requisiti forza password | ✅ |

### 👥 Test RBAC (`rbac_tests.rs`)

| Test | Descrizione | Stato |
|------|-------------|-------|
| `test_new_user_has_player_role_by_default` | Verifica ruolo default 'player' | ✅ |
| `test_dm_role_can_be_assigned` | Verifica assegnazione ruolo 'dm' | ✅ |
| `test_player_role_cannot_create_polls` | Verifica restrizioni player | ✅ |
| `test_dm_role_can_create_polls` | Verifica permessi DM | ✅ |
| `test_promote_user_from_player_to_dm` | Verifica promozione utente | ✅ |
| `test_role_persists_across_sessions` | Verifica persistenza ruolo | ✅ |
| `test_invalid_role_values_not_allowed` | Verifica solo ruoli validi | ✅ |

### 🛠️ Test Helper Utilities (`helpers.rs`)

| Test | Descrizione | Stato |
|------|-------------|-------|
| `test_setup_test_db` | Verifica creazione database temporaneo | ✅ |
| `test_create_test_user` | Verifica creazione utente di test | ✅ |
| `test_create_test_session` | Verifica creazione sessione di test | ✅ |

---

## 📊 Metriche dei Test

### Coverage Obiettivi

| Modulo | Coverage Target | Coverage Attuale |
|--------|-----------------|------------------|
| auth.rs | 80% | 🎯 In Progress |
| handlers.rs | 70% | 🎯 In Progress |
| models.rs | 60% | 🎯 In Progress |
| db.rs | 50% | 🎯 In Progress |

### Generare Report Coverage

```bash
# Installa cargo-tarpaulin
cargo install cargo-tarpaulin

# Genera report
./run_tests.sh coverage

# Apri report HTML
xdg-open coverage/index.html
```

---

## ✅ Best Practices

### 1. Naming Convention

```rust
// ✅ BUONO - Nome descrittivo
#[tokio::test]
async fn test_user_cannot_register_with_duplicate_email() { }

// ❌ CATTIVO - Nome generico
#[tokio::test]
async fn test_registration() { }
```

### 2. Arrange-Act-Assert Pattern

```rust
#[tokio::test]
async fn test_example() {
    // ARRANGE - Setup
    let pool = setup_test_db().await;
    let email = "test@example.com";

    // ACT - Esegui azione
    let user_id = create_test_user(&pool, email, "password", "player").await;

    // ASSERT - Verifica risultato
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(result.0, 1);

    // CLEANUP
    cleanup_test_db(&pool).await;
}
```

### 3. Test Isolation

- ✅ Ogni test usa un database temporaneo indipendente
- ✅ Cleanup dopo ogni test
- ✅ Nessuna dipendenza tra test
- ✅ Test eseguibili in qualsiasi ordine

### 4. Data di Test

```rust
// ✅ BUONO - Dati realistici
let email = "realistic.user@example.com";
let password = "SecurePassword123!@#";

// ❌ CATTIVO - Dati troppo semplici
let email = "a@a";
let password = "pass";
```

---

## 🔄 CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all
```

### GitLab CI Example

```yaml
test:
  image: rust:latest
  script:
    - cargo test --all
  only:
    - merge_requests
    - main
```

---

## 🐛 Debugging Test Failures

### Vedere Output Dettagliato

```bash
# Output completo
cargo test -- --nocapture

# Test specifico con output
cargo test test_user_login_with_valid_credentials -- --nocapture --test-threads=1
```

### Logging nei Test

```rust
#[tokio::test]
async fn test_with_logging() {
    // Abilita logging
    tracing_subscriber::fmt::init();

    tracing::info!("Starting test");
    // ... test code ...
    tracing::info!("Test completed");
}
```

---

## 📈 Aggiungere Nuovi Test

### 1. Test Unitario (nel modulo)

```rust
// In src/auth.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
    }
}
```

### 2. Test di Integrazione

```rust
// In tests/integration/new_feature_tests.rs
#[cfg(test)]
mod new_feature_tests {
    use crate::helpers::*;

    #[tokio::test]
    async fn test_new_feature() {
        let pool = setup_test_db().await;
        // ... test logic ...
        cleanup_test_db(&pool).await;
    }
}
```

### 3. Registrare in `main.rs`

```rust
// In tests/integration/main.rs
mod new_feature_tests; // Aggiungi questa linea
```

---

## 🚀 Comandi Rapidi

```bash
# Test rapido
cargo test

# Test con dettagli
cargo test -- --nocapture

# Test specifici
./run_tests.sh auth
./run_tests.sh rbac

# Watch mode
./run_tests.sh watch

# Coverage
./run_tests.sh coverage
```

---

## 📚 Risorse

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [TDD Best Practices](https://martinfowler.com/bliki/TestDrivenDevelopment.html)

---

## 🎉 Conclusione

Il sistema TDD implementato garantisce:
- ✅ Codice affidabile e testato
- ✅ Sicurezza RBAC verificata
- ✅ Facile manutenzione e refactoring
- ✅ Documentazione vivente del comportamento

**Happy Testing!** 🎲
