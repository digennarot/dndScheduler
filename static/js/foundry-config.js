/**
 * 🎲 FoundryVTT Configuration Module
 * 
 * Gestisce la configurazione del link FoundryVTT con storage in localStorage.
 * L'admin può configurare l'URL e abilitare/disabilitare il pulsante.
 */

const FoundryConfig = {
    // Chiavi localStorage
    STORAGE_KEYS: {
        URL: 'foundry_url',
        ENABLED: 'foundry_enabled'
    },

    // Default values
    DEFAULTS: {
        URL: 'http://127.0.0.1:30000/auth',
        ENABLED: true
    },

    /**
     * Ottiene l'URL FoundryVTT configurato
     * @returns {string} URL FoundryVTT
     */
    getUrl() {
        return localStorage.getItem(this.STORAGE_KEYS.URL) || this.DEFAULTS.URL;
    },

    /**
     * Imposta l'URL FoundryVTT
     * @param {string} url - Nuovo URL
     */
    setUrl(url) {
        if (url && url.trim()) {
            localStorage.setItem(this.STORAGE_KEYS.URL, url.trim());
            this.updateAllButtons();
        }
    },

    /**
     * Controlla se il pulsante FoundryVTT è abilitato
     * @returns {boolean}
     */
    isEnabled() {
        const stored = localStorage.getItem(this.STORAGE_KEYS.ENABLED);
        // Default a true se non configurato
        return stored === null ? this.DEFAULTS.ENABLED : stored === 'true';
    },

    /**
     * Abilita/disabilita il pulsante FoundryVTT
     * @param {boolean} enabled
     */
    setEnabled(enabled) {
        localStorage.setItem(this.STORAGE_KEYS.ENABLED, enabled.toString());
        this.updateAllButtons();
    },

    /**
     * Aggiorna tutti i pulsanti FoundryVTT nella pagina
     */
    updateAllButtons() {
        const buttons = document.querySelectorAll('[data-foundry-btn]');
        const url = this.getUrl();
        const enabled = this.isEnabled();

        buttons.forEach(btn => {
            if (btn.tagName === 'A') {
                btn.href = url;
            }
            btn.style.display = enabled ? '' : 'none';
        });
    },

    /**
     * Inizializza i pulsanti FoundryVTT sulla pagina
     * Da chiamare dopo DOMContentLoaded
     */
    init() {
        this.updateAllButtons();
    },

    /**
     * Crea un pulsante FoundryVTT con lo stile standard
     * @param {string} variant - 'navbar' | 'header' | 'sidebar'
     * @returns {string} HTML del pulsante
     */
    createButton(variant = 'navbar') {
        const url = this.getUrl();
        const enabled = this.isEnabled();
        const display = enabled ? '' : 'display: none;';

        switch (variant) {
            case 'header':
                return `
                    <a href="${url}" target="_blank" data-foundry-btn
                       class="action-btn primary flex items-center space-x-2" style="${display}">
                        <span>🎲</span>
                        <span>FoundryVTT</span>
                    </a>
                `;
            case 'sidebar':
                return `
                    <a href="${url}" target="_blank" data-foundry-btn
                       class="nav-item flex items-center w-full text-left p-2 rounded hover:bg-gray-700 transition-colors text-sm text-gray-200" style="${display}">
                        <span class="mr-3">🎲</span>
                        <span>FoundryVTT</span>
                    </a>
                `;
            case 'navbar':
            default:
                return `
                    <a href="${url}" target="_blank" data-foundry-btn
                       class="flex items-center space-x-2 bg-gradient-to-r from-amber to-copper text-white px-4 py-2 rounded-lg font-semibold hover:shadow-lg transition-all" style="${display}">
                        <span>🎲</span>
                        <span>FoundryVTT</span>
                    </a>
                `;
        }
    },

    /**
     * Resetta la configurazione ai valori di default
     */
    reset() {
        localStorage.removeItem(this.STORAGE_KEYS.URL);
        localStorage.removeItem(this.STORAGE_KEYS.ENABLED);
        this.updateAllButtons();
    }
};

// Auto-init quando il DOM è pronto
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => FoundryConfig.init());
} else {
    FoundryConfig.init();
}

// Esporta per uso globale
window.FoundryConfig = FoundryConfig;
