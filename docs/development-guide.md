# Development Guide

## Prerequisites
- **Rust**: Latest stable version ([Install Rust](https://rustup.rs/))
- **SQLite**: Database engine
- **GCC/Build Essentials**: For compiling dependencies

## Local Setup
1. **Clone the repository**
2. **Environment Configuration**
   ```bash
   cp .env.example .env
   # Edit .env with your local settings (e.g., SMTP, Admin credentials)
   ```
3. **Database Initialization**
   The application automatically initializes the SQLite database (`dnd_scheduler.db`) on the first run.

## Running the Application
```bash
# Debug mode (faster compilation)
cargo run

# Release mode (optimized)
cargo run --release
```
The server will start at `http://localhost:3000`.

## Testing
The project includes a comprehensive test runner script:

```bash
# Run all tests
./scripts/run_tests.sh all

# Run specific categories
./scripts/run_tests.sh unit       # Unit tests
./scripts/run_tests.sh integration # Integration tests
./scripts/run_tests.sh auth       # Authentication tests only
./scripts/run_tests.sh rbac       # Role-based access control tests
```

### Coverage
To generate a coverage report (requires `cargo-tarpaulin`):
```bash
./scripts/run_tests.sh coverage
```

## Project Structure
- `src/`: Rust backend code
- `static/`: Frontend assets (HTML, JS, CSS)
- `tests/`: Integration test suite
- `scripts/`: Helper utilities

## Common Tasks
- **Reset Database**: Delete `dnd_scheduler.db` and restart the server.
- **Admin Access**: Default admin credentials are defined in your `.env` (Defaults: `admin@example.com` / `password123` if not set).
- **Frontend Changes**: Edit files in `static/`. No build step required (Vanilla JS).
