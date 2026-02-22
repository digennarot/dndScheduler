/**
 * Cookie Consent Manager
 * GDPR-compliant cookie consent banner with granular controls
 */

(function () {
    'use strict';

    const CONSENT_KEY = 'cookie_consent';
    const CONSENT_VERSION = '1.0';

    // Default consent state (all off except necessary)
    const defaultConsent = {
        version: CONSENT_VERSION,
        necessary: true, // Always required
        functional: false,
        analytics: false,
        timestamp: null
    };

    // Cookie Consent Manager
    window.CookieConsent = {
        consent: null,

        init: function () {
            this.consent = this.getStoredConsent();

            // Show banner if no consent stored or version changed
            if (!this.consent || this.consent.version !== CONSENT_VERSION) {
                this.showBanner();
            } else {
                this.applyConsent();
            }
        },

        getStoredConsent: function () {
            try {
                const stored = localStorage.getItem(CONSENT_KEY);
                return stored ? JSON.parse(stored) : null;
            } catch (e) {
                return null;
            }
        },

        saveConsent: function (consent) {
            consent.timestamp = new Date().toISOString();
            consent.version = CONSENT_VERSION;
            localStorage.setItem(CONSENT_KEY, JSON.stringify(consent));
            this.consent = consent;

            // Sync with backend if user is authenticated
            this.syncWithBackend(consent);
        },

        syncWithBackend: function (consent) {
            const token = localStorage.getItem('authToken');
            if (!token) return;

            fetch('/api/gdpr/consent', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({
                    consent_analytics: consent.analytics
                })
            }).catch(() => {
                // Silently fail - consent is stored locally
            });
        },

        showBanner: function () {
            // Don't show on cookie/privacy policy pages
            if (window.location.pathname.includes('cookie-policy') ||
                window.location.pathname.includes('privacy-policy')) {
                return;
            }

            const banner = document.createElement('div');
            banner.id = 'cookie-consent-banner';
            banner.innerHTML = `
                <div class="cookie-banner-content">
                    <div class="cookie-banner-text">
                        <h3>🍪 Cookie Tecnici</h3>
                        <p>Questo sito utilizza <strong>solo cookie tecnici</strong> necessari per il funzionamento. 
                           Nessun cookie di profilazione o tracciamento.
                           <a href="/cookie-policy.html" style="color: #d4a574; text-decoration: underline;">Maggiori info</a>
                        </p>
                    </div>
                    <div class="cookie-banner-buttons">
                        <button id="cookie-accept-all" class="cookie-btn cookie-btn-primary">✓ Ho capito</button>
                    </div>
                </div>
            `;

            this.addBannerStyles();
            document.body.appendChild(banner);

            // Event listeners
            document.getElementById('cookie-accept-all').addEventListener('click', () => {
                this.saveConsent({ necessary: true, functional: false, analytics: false });
                this.hideBanner();
                this.applyConsent();
            });
        },

        hideBanner: function () {
            const banner = document.getElementById('cookie-consent-banner');
            if (banner) {
                banner.classList.add('cookie-banner-hidden');
                setTimeout(() => banner.remove(), 300);
            }
        },

        showSettings: function () {
            const modal = document.createElement('div');
            modal.id = 'cookie-settings-modal';
            modal.innerHTML = `
                <div class="cookie-modal-overlay"></div>
                <div class="cookie-modal-content">
                    <h2>Preferenze Cookie</h2>
                    <p class="cookie-modal-description">
                        Gestisci le tue preferenze sui cookie. I cookie necessari sono sempre abilitati.
                    </p>
                    
                    <div class="cookie-category">
                        <div class="cookie-category-header">
                            <label>
                                <input type="checkbox" id="consent-necessary" checked disabled>
                                <span class="cookie-category-name">Cookie Necessari</span>
                            </label>
                            <span class="cookie-badge">Sempre attivi</span>
                        </div>
                        <p class="cookie-category-desc">
                            Essenziali per il funzionamento del sito. Includono autenticazione e preferenze.
                        </p>
                    </div>
                    
                    <div class="cookie-category">
                        <div class="cookie-category-header">
                            <label>
                                <input type="checkbox" id="consent-functional" ${this.consent?.functional ? 'checked' : ''}>
                                <span class="cookie-category-name">Cookie Funzionali</span>
                            </label>
                        </div>
                        <p class="cookie-category-desc">
                            Migliorano l'esperienza utente ricordando preferenze come tema e lingua.
                        </p>
                    </div>
                    
                    <div class="cookie-category">
                        <div class="cookie-category-header">
                            <label>
                                <input type="checkbox" id="consent-analytics" ${this.consent?.analytics ? 'checked' : ''}>
                                <span class="cookie-category-name">Cookie Analitici</span>
                            </label>
                        </div>
                        <p class="cookie-category-desc">
                            Ci aiutano a capire come usi il sito per migliorarlo. Dati anonimi.
                        </p>
                    </div>
                    
                    <div class="cookie-modal-footer">
                        <button id="cookie-save-settings" class="cookie-btn cookie-btn-primary">Salva Preferenze</button>
                        <button id="cookie-cancel-settings" class="cookie-btn cookie-btn-secondary">Annulla</button>
                    </div>
                    
                    <div class="cookie-modal-links">
                        <a href="/cookie-policy.html">Cookie Policy</a>
                        <a href="/privacy-policy.html">Privacy Policy</a>
                    </div>
                </div>
            `;

            this.addModalStyles();
            document.body.appendChild(modal);

            // Event listeners
            document.getElementById('cookie-save-settings').addEventListener('click', () => {
                const consent = {
                    necessary: true,
                    functional: document.getElementById('consent-functional').checked,
                    analytics: document.getElementById('consent-analytics').checked
                };
                this.saveConsent(consent);
                this.hideSettings();
                this.hideBanner();
                this.applyConsent();
            });

            document.getElementById('cookie-cancel-settings').addEventListener('click', () => {
                this.hideSettings();
            });

            document.querySelector('.cookie-modal-overlay').addEventListener('click', () => {
                this.hideSettings();
            });
        },

        hideSettings: function () {
            const modal = document.getElementById('cookie-settings-modal');
            if (modal) {
                modal.classList.add('cookie-modal-hidden');
                setTimeout(() => modal.remove(), 300);
            }
        },

        applyConsent: function () {
            // Apply functional cookies
            if (this.consent?.functional) {
                // Enable theme persistence, language preferences, etc.
                document.dispatchEvent(new CustomEvent('cookieConsent:functional', { detail: true }));
            }

            // Apply analytics
            if (this.consent?.analytics) {
                // Enable analytics tracking
                document.dispatchEvent(new CustomEvent('cookieConsent:analytics', { detail: true }));
            }
        },

        hasConsent: function (type) {
            return this.consent ? !!this.consent[type] : false;
        },

        addBannerStyles: function () {
            if (document.getElementById('cookie-consent-styles')) return;

            const styles = document.createElement('style');
            styles.id = 'cookie-consent-styles';
            styles.textContent = `
                #cookie-consent-banner {
                    position: fixed;
                    bottom: 0;
                    left: 0;
                    right: 0;
                    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
                    border-top: 1px solid rgba(212, 165, 116, 0.3);
                    padding: 1.5rem;
                    z-index: 9999;
                    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.3);
                    animation: slideUp 0.3s ease-out;
                }
                
                @keyframes slideUp {
                    from { transform: translateY(100%); opacity: 0; }
                    to { transform: translateY(0); opacity: 1; }
                }
                
                .cookie-banner-hidden {
                    animation: slideDown 0.3s ease-out forwards !important;
                }
                
                @keyframes slideDown {
                    from { transform: translateY(0); opacity: 1; }
                    to { transform: translateY(100%); opacity: 0; }
                }
                
                .cookie-banner-content {
                    max-width: 1200px;
                    margin: 0 auto;
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 2rem;
                    flex-wrap: wrap;
                }
                
                .cookie-banner-text h3 {
                    color: #d4a574;
                    font-family: 'Cinzel', serif;
                    margin-bottom: 0.5rem;
                    font-size: 1.1rem;
                }
                
                .cookie-banner-text p {
                    color: #e2e8f0;
                    font-size: 0.9rem;
                    margin: 0;
                }
                
                .cookie-banner-buttons {
                    display: flex;
                    gap: 0.75rem;
                    flex-wrap: wrap;
                }
                
                .cookie-btn {
                    padding: 0.75rem 1.5rem;
                    border-radius: 0.5rem;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.2s ease;
                    border: none;
                    font-size: 0.9rem;
                }
                
                .cookie-btn-primary {
                    background: #d4a574;
                    color: #0f172a;
                }
                
                .cookie-btn-primary:hover {
                    background: #c49464;
                }
                
                .cookie-btn-secondary {
                    background: rgba(212, 165, 116, 0.1);
                    color: #d4a574;
                    border: 1px solid rgba(212, 165, 116, 0.3);
                }
                
                .cookie-btn-secondary:hover {
                    background: rgba(212, 165, 116, 0.2);
                }
                
                .cookie-btn-link {
                    background: transparent;
                    color: #d4a574;
                    text-decoration: underline;
                    padding: 0.75rem 1rem;
                }
                
                .cookie-btn-link:hover {
                    color: #e2e8f0;
                }
                
                @media (max-width: 768px) {
                    .cookie-banner-content {
                        flex-direction: column;
                        text-align: center;
                    }
                    
                    .cookie-banner-buttons {
                        width: 100%;
                        justify-content: center;
                    }
                }
            `;
            document.head.appendChild(styles);
        },

        addModalStyles: function () {
            if (document.getElementById('cookie-modal-styles')) return;

            const styles = document.createElement('style');
            styles.id = 'cookie-modal-styles';
            styles.textContent = `
                #cookie-settings-modal {
                    position: fixed;
                    inset: 0;
                    z-index: 10000;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    animation: fadeIn 0.2s ease-out;
                }
                
                @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                }
                
                .cookie-modal-hidden {
                    animation: fadeOut 0.2s ease-out forwards !important;
                }
                
                @keyframes fadeOut {
                    from { opacity: 1; }
                    to { opacity: 0; }
                }
                
                .cookie-modal-overlay {
                    position: absolute;
                    inset: 0;
                    background: rgba(0, 0, 0, 0.7);
                    backdrop-filter: blur(4px);
                }
                
                .cookie-modal-content {
                    position: relative;
                    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
                    border: 1px solid rgba(212, 165, 116, 0.3);
                    border-radius: 1rem;
                    padding: 2rem;
                    max-width: 500px;
                    width: 90%;
                    max-height: 90vh;
                    overflow-y: auto;
                    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
                }
                
                .cookie-modal-content h2 {
                    color: #d4a574;
                    font-family: 'Cinzel', serif;
                    margin-bottom: 0.5rem;
                    font-size: 1.5rem;
                }
                
                .cookie-modal-description {
                    color: #94a3b8;
                    font-size: 0.9rem;
                    margin-bottom: 1.5rem;
                }
                
                .cookie-category {
                    background: rgba(0, 0, 0, 0.2);
                    border: 1px solid rgba(212, 165, 116, 0.1);
                    border-radius: 0.5rem;
                    padding: 1rem;
                    margin-bottom: 1rem;
                }
                
                .cookie-category-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 0.5rem;
                }
                
                .cookie-category-header label {
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                    cursor: pointer;
                    color: #e2e8f0;
                }
                
                .cookie-category-header input[type="checkbox"] {
                    width: 1.25rem;
                    height: 1.25rem;
                    accent-color: #d4a574;
                }
                
                .cookie-category-name {
                    font-weight: 600;
                }
                
                .cookie-badge {
                    background: rgba(212, 165, 116, 0.2);
                    color: #d4a574;
                    padding: 0.25rem 0.5rem;
                    border-radius: 0.25rem;
                    font-size: 0.75rem;
                    font-weight: 600;
                }
                
                .cookie-category-desc {
                    color: #94a3b8;
                    font-size: 0.85rem;
                    margin: 0;
                }
                
                .cookie-modal-footer {
                    display: flex;
                    gap: 0.75rem;
                    margin-top: 1.5rem;
                }
                
                .cookie-modal-footer .cookie-btn {
                    flex: 1;
                }
                
                .cookie-modal-links {
                    display: flex;
                    justify-content: center;
                    gap: 1.5rem;
                    margin-top: 1.5rem;
                    padding-top: 1rem;
                    border-top: 1px solid rgba(212, 165, 116, 0.1);
                }
                
                .cookie-modal-links a {
                    color: #d4a574;
                    font-size: 0.85rem;
                    text-decoration: none;
                }
                
                .cookie-modal-links a:hover {
                    text-decoration: underline;
                }
            `;
            document.head.appendChild(styles);
        }
    };

    // Auto-initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => CookieConsent.init());
    } else {
        CookieConsent.init();
    }
})();
