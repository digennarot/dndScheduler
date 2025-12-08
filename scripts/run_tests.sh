#!/bin/bash

# Test Runner Script for D&D Scheduler
# Esegue tutti i test dell'applicazione con opzioni avanzate

set -e

echo "=========================================="
echo "D&D Scheduler - Test Runner"
echo "=========================================="
echo ""

# Colori per output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Funzione di help
show_help() {
    echo "Uso: ./run_tests.sh [OPZIONE]"
    echo ""
    echo "Opzioni:"
    echo "  all              Esegui tutti i test (default)"
    echo "  unit             Esegui solo i test unitari"
    echo "  integration      Esegui solo i test di integrazione"
    echo "  auth             Esegui solo i test di autenticazione"
    echo "  rbac             Esegui solo i test RBAC"
    echo "  coverage         Esegui i test con coverage report"
    echo "  watch            Esegui i test in modalità watch"
    echo "  clean            Pulisci i file di test temporanei"
    echo "  help             Mostra questo help"
    echo ""
    exit 0
}

# Parse arguments
COMMAND=${1:-all}

case "$COMMAND" in
    help)
        show_help
        ;;

    all)
        echo -e "${GREEN}Esecuzione di tutti i test...${NC}"
        echo ""
        cargo test --all -- --nocapture
        ;;

    unit)
        echo -e "${GREEN}Esecuzione dei test unitari...${NC}"
        echo ""
        cargo test --lib -- --nocapture
        ;;

    integration)
        echo -e "${GREEN}Esecuzione dei test di integrazione...${NC}"
        echo ""
        cargo test --test '*' -- --nocapture
        ;;

    auth)
        echo -e "${GREEN}Esecuzione dei test di autenticazione...${NC}"
        echo ""
        cargo test --test integration auth_integration_tests -- --nocapture
        ;;

    rbac)
        echo -e "${GREEN}Esecuzione dei test RBAC...${NC}"
        echo ""
        cargo test --test integration rbac_integration_tests -- --nocapture
        ;;

    coverage)
        echo -e "${GREEN}Esecuzione dei test con coverage...${NC}"
        echo ""
        if ! command -v cargo-tarpaulin &> /dev/null; then
            echo -e "${YELLOW}cargo-tarpaulin non installato${NC}"
            echo "Installa con: cargo install cargo-tarpaulin"
            echo ""
            echo "Eseguo test senza coverage..."
            cargo test --all -- --nocapture
        else
            cargo tarpaulin --out Html --output-dir coverage
            echo ""
            echo -e "${GREEN}Coverage report generato in: coverage/index.html${NC}"
        fi
        ;;

    watch)
        echo -e "${GREEN}Esecuzione dei test in modalità watch...${NC}"
        echo ""
        if ! command -v cargo-watch &> /dev/null; then
            echo -e "${YELLOW}cargo-watch non installato${NC}"
            echo "Installa con: cargo install cargo-watch"
            exit 1
        fi
        cargo watch -x test
        ;;

    clean)
        echo -e "${GREEN}Pulizia file temporanei di test...${NC}"
        echo ""
        cargo clean
        rm -rf coverage/
        rm -f *.db *.db-shm *.db-wal
        echo -e "${GREEN}Pulizia completata!${NC}"
        ;;

    *)
        echo -e "${RED}Comando sconosciuto: $COMMAND${NC}"
        echo ""
        show_help
        ;;
esac

echo ""
echo "=========================================="
echo -e "${GREEN}Test completati!${NC}"
echo "=========================================="
