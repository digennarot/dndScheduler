/**
 * Navigation Protection
 * Nasconde il link "Gestisci" se l'utente non è autenticato
 */

(function () {
    'use strict';

    /**
     * Controlla se l'utente è loggato
     */
    function isUserLoggedIn() {
        // Controlla se esiste authManager e se l'utente è loggato
        if (window.authManager && typeof window.authManager.isLoggedIn === 'function') {
            return window.authManager.isLoggedIn();
        }

        // Fallback: controlla localStorage
        const currentUser = localStorage.getItem('currentUser');
        if (currentUser) {
            try {
                const user = JSON.parse(currentUser);
                return user && user.id;
            } catch (e) {
                console.error('Errore parsing currentUser:', e);
                return false;
            }
        }

        return false;
    }

    /**
     * Protegge la navigazione nascondendo elementi per utenti non loggati
     */
    function protectNavigation() {
        const manageLink = document.getElementById('nav-manage');

        if (manageLink) {
            if (!isUserLoggedIn()) {
                // Nasconde il link "Gestisci" se non loggato
                manageLink.style.display = 'none';
                console.log('🔒 Link "Gestisci" nascosto (utente non autenticato)');
            } else {
                // Mostra il link se loggato
                manageLink.style.display = '';
                console.log('✅ Link "Gestisci" visibile (utente autenticato)');
            }
        }
    }

    /**
     * Inizializza la protezione quando il DOM è pronto
     */
    function init() {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', protectNavigation);
        } else {
            protectNavigation();
        }
    }

    // Avvia l'inizializzazione
    init();

    // Espone la funzione globalmente per eventuali aggiornamenti
    window.updateNavProtection = protectNavigation;

})();
