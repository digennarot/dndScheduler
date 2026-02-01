// Authentication Utilities with Authelia SSO Support
// Updated for backward compatibility with auth.js
class AuthManager {
    constructor() {
        // Use standard keys for compatibility with availability-manager.js and other scripts
        this.TOKEN_KEY = 'authToken';
        this.USER_KEY = 'currentUser';
        this.AUTHELIA_CONFIG_KEY = 'authelia_config';
        this._autheliaConfig = null;
    }

    // --- Legacy Compatibility Getters/Methods ---
    get user() {
        return this.getUser();
    }

    getCurrentUser() {
        return this.getUser();
    }

    isLoggedIn() {
        return this.isAuthenticated();
    }
    // ---------------------------------------------

    /**
     * Get Authelia SSO configuration from server
     */
    async getAutheliaConfig() {
        // Return cached config if available
        if (this._autheliaConfig) {
            return this._autheliaConfig;
        }

        try {
            const response = await fetch('/api/auth/authelia/config');
            if (response.ok) {
                this._autheliaConfig = await response.json();
                return this._autheliaConfig;
            }
        } catch (error) {
            console.warn('Failed to fetch Authelia config:', error);
        }

        return { enabled: false, login_url: null, logout_url: null };
    }

    /**
     * Get Google Auth configuration from server
     */
    async getGoogleConfig() {
        try {
            const response = await fetch('/api/auth/google/config');
            if (response.ok) {
                return await response.json();
            }
        } catch (error) {
            console.warn('Failed to fetch Google config:', error);
        }
        return { client_id: null };
    }

    /**
     * Initialize Google Sign-In Button
     */
    async initGoogleLogin(containerId) {
        const config = await this.getGoogleConfig();
        const container = document.getElementById(containerId);

        if (!config.client_id) {
            console.log('Google Auth not configured (no client_id)');
            if (container) {
                container.innerHTML = '<span class="text-orange-500 text-sm font-bold">Debug: Google Auth Disabled (Backend returned no client_id)</span>';
            }
            return;
        }

        console.log('Initializing Google Auth with Client ID:', config.client_id);

        let attempts = 0;
        const maxAttempts = 50; // 5 seconds

        const render = () => {
            if (window.google && window.google.accounts) {
                try {
                    window.google.accounts.id.initialize({
                        client_id: config.client_id,
                        callback: window.handleGoogleLogin,
                        auto_select: false,
                        cancel_on_tap_outside: true
                    });

                    window.google.accounts.id.renderButton(
                        container,
                        { theme: "outline", size: "large", width: "100%" }
                    );
                    console.log('Google Sign-In initialized');

                    // Remove loading text if it exists (Google button replaces content, but just in case)
                    // The renderButton method replaces the contents of the container, 
                    // so the "Caricamento..." text will be removed automatically if successful.

                } catch (e) {
                    console.error('Google Auth Init Error:', e);
                    if (container) {
                        container.innerHTML = `<span class="text-red-500 text-sm">Errore Google: ${e.message}</span>`;
                    }
                }
            } else {
                attempts++;
                if (attempts > maxAttempts) {
                    console.error('Google Identity Services script timeout');
                    if (container) {
                        container.innerHTML = '<span class="text-red-500 text-sm">Timeout: Impossibile caricare Google Script. Controlla la connessione o disabilita AdBlock.</span>';
                    }
                    return;
                }
                console.log(`Waiting for Google Identity Services... (${attempts}/${maxAttempts})`);
                setTimeout(render, 100);
            }
        };

        render();
    }

    /**
     * Check if user is authenticated via Authelia SSO
     * This works when behind Caddy ForwardAuth
     */
    async checkAutheliaSession() {
        const config = await this.getAutheliaConfig();
        if (!config.enabled) {
            return null;
        }

        try {
            const response = await fetch('/api/auth/authelia/session');
            if (response.ok) {
                const user = await response.json();
                // Store user data locally for quick access
                this.setUser(user);
                return user;
            }
        } catch (error) {
            console.debug('No Authelia session:', error);
        }

        return null;
    }

    /**
     * Redirect to Authelia login page
     */
    async redirectToAutheliaLogin() {
        const config = await this.getAutheliaConfig();
        if (config.enabled && config.login_url) {
            // Store current URL for post-login redirect
            const returnUrl = window.location.href;
            // Authelia uses 'rd' parameter for redirect
            window.location.href = `${config.login_url}?rd=${encodeURIComponent(returnUrl)}`;
            return true;
        }
        return false;
    }

    /**
     * Logout via Authelia (if enabled) or local logout
     */
    async logout() {
        const token = this.getToken();
        const config = await this.getAutheliaConfig();

        // Clear local session first
        if (token) {
            try {
                await fetch(`/api/auth/logout/${token}`, { method: 'POST' });
            } catch (error) {
                console.error('Logout error:', error);
            }
        }

        // Clear local data
        this.setToken(null);
        this.setUser(null);
        localStorage.removeItem('rememberMe'); // Clear legacy flag if present

        // If Authelia is enabled, redirect to Authelia logout
        if (config.enabled && config.logout_url) {
            window.location.href = config.logout_url;
        } else {
            window.location.href = '/login.html';
        }
    }

    /**
     * Get the current session token
     */
    getToken() {
        return localStorage.getItem(this.TOKEN_KEY);
    }

    /**
     * Set the session token
     */
    setToken(token) {
        if (token) {
            localStorage.setItem(this.TOKEN_KEY, token);
        } else {
            localStorage.removeItem(this.TOKEN_KEY);
        }
    }

    /**
     * Get the current user data
     */
    getUser() {
        const userData = localStorage.getItem(this.USER_KEY);
        return userData ? JSON.parse(userData) : null;
    }

    /**
     * Set the current user data
     */
    setUser(user) {
        if (user) {
            localStorage.setItem(this.USER_KEY, JSON.stringify(user));
        } else {
            localStorage.removeItem(this.USER_KEY);
        }
    }

    /**
     * Check if user is authenticated (local token or Authelia)
     */
    isAuthenticated() {
        return !!this.getToken() || !!this.getUser();
    }

    /**
     * Verify session with the server (supports both local and Authelia)
     */
    async verifySession() {
        // First check for Authelia session (if enabled and behind ForwardAuth)
        const autheliaUser = await this.checkAutheliaSession();
        if (autheliaUser) {
            console.log('✅ Authelia session verified');
            return true;
        }

        // Fall back to local token verification
        const token = this.getToken();
        if (!token) {
            console.log('❌ No token found in localStorage');
            return false;
        }

        console.log('🔍 Verifying session with token:', token.substring(0, 8) + '...');

        try {
            const response = await fetch(`/api/auth/me/${token}`);
            console.log('📡 Session verification response status:', response.status);

            if (response.ok) {
                const user = await response.json();
                this.setUser(user);
                console.log('✅ Session verified for user:', user.name);
                return true;
            } else {
                // Session invalid, clear local data
                const errorText = await response.text();
                console.error('❌ Session invalid:', response.status, errorText);
                this.setToken(null);
                this.setUser(null);
                return false;
            }
        } catch (error) {
            console.error('❌ Session verification failed:', error);
            return false;
        }
    }

    /**
     * Require authentication - redirect to login if not authenticated
     * Supports Authelia SSO redirect when enabled
     * @param {string} redirectUrl - Optional URL to redirect back to after login
     */
    async requireAuth(redirectUrl = null) {
        console.log('🔐 [requireAuth] Starting authentication check...');
        console.log('🔐 [requireAuth] Current page:', window.location.href);

        const isValid = await this.verifySession();

        console.log('🔐 [requireAuth] verifySession result:', isValid);

        if (!isValid) {
            console.log('❌ [requireAuth] Session invalid, preparing redirect...');

            // Store the intended destination
            const returnUrl = redirectUrl || window.location.pathname + window.location.search;
            console.log('🔐 [requireAuth] Return URL:', returnUrl);

            sessionStorage.setItem('auth_return_url', returnUrl);

            // Try Authelia redirect first
            console.log('🔐 [requireAuth] Checking Authelia...');
            const redirected = await this.redirectToAutheliaLogin();

            if (!redirected) {
                console.log('🔐 [requireAuth] No Authelia, redirecting to login page...');
                // Fall back to local login page
                window.location.href = `/login.html?return=${encodeURIComponent(returnUrl)}`;
            } else {
                console.log('🔐 [requireAuth] Redirecting via Authelia...');
            }
            return false;
        }

        console.log('✅ [requireAuth] Authentication successful!');
        return true;
    }

    /**
     * Login with credentials (for non-Authelia authentication)
     */
    async login(email, password) {
        try {
            console.log('🔐 Attempting login for:', email);
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ email, password })
            });

            if (!response.ok) {
                const error = await response.text();
                console.error('❌ Login failed:', response.status, error);
                throw new Error(error || 'Login fallito');
            }

            const data = await response.json();
            console.log('✅ Login successful, received token:', data.token.substring(0, 8) + '...');
            console.log('👤 User data:', data.user);

            this.setToken(data.token);
            this.setUser(data.user);

            console.log('💾 Token and user saved to localStorage');

            return data;
        } catch (error) {
            console.error('❌ Login error:', error);
            throw error;
        }
    }

    /**
     * Register a new user (for non-Authelia registration)
     */
    async register(name, email, password) {
        try {
            const response = await fetch('/api/auth/register', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ name, email, password })
            });

            if (!response.ok) {
                const error = await response.text();
                throw new Error(error || 'Registrazione fallita');
            }

            const data = await response.json();
            this.setToken(data.token);
            this.setUser(data.user);

            return data;
        } catch (error) {
            console.error('Registration error:', error);
            throw error;
        }
    }

    // Google Login
    async loginWithGoogle(credential) {
        try {
            console.log('🔐 Attempting Google login...');
            const response = await fetch('/api/auth/google/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ credential })
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Google login failed');
            }

            const data = await response.json();
            console.log('✅ Google Login successful');

            this.setToken(data.token);
            this.setUser(data.user);

            return data;
        } catch (error) {
            console.error('Google login error:', error);
            throw error;
        }
    }

    /**
     * Get the return URL after login
     */
    getReturnUrl() {
        const returnUrl = sessionStorage.getItem('auth_return_url');
        sessionStorage.removeItem('auth_return_url');
        return returnUrl || '/dashboard.html';
    }

    /**
     * Escape HTML to prevent XSS
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Update user profile in navbar (with Authelia support)
     */
    async updateNavbar() {
        const user = this.getUser();
        const config = await this.getAutheliaConfig();

        // Support for dashboard.html style navigation (user-display)
        const userDisplay = document.getElementById('user-display');
        const logoutBtn = document.getElementById('logout-btn');

        if (userDisplay) {
            if (user) {
                userDisplay.innerHTML = `
                <div class="flex items-center space-x-2">
                    <div class="w-8 h-8 bg-gradient-to-br from-emerald to-mystic rounded-full flex items-center justify-center">
                        <span class="text-white font-semibold text-sm">${user.name.charAt(0).toUpperCase()}</span>
                    </div>
                    <div class="text-sm">
                        <div class="font-semibold text-forest">${this.escapeHtml(user.name)}</div>
                         <!-- <div class="text-xs text-gray-500">${this.escapeHtml(user.email)}</div> -->
                    </div>
                </div>`;
                userDisplay.style.display = 'flex';
                if (logoutBtn) {
                    logoutBtn.style.display = 'block';
                    logoutBtn.onclick = () => this.logout();
                }
            } else {
                userDisplay.style.display = 'none';
                if (logoutBtn) logoutBtn.style.display = 'none';
            }
        }

        // Support for main site navigation (user-menu)
        const userMenuContainer = document.getElementById('user-menu');
        if (userMenuContainer) {
            if (user) {
                // Show SSO badge if authenticated via Authelia
                const ssoBadge = config.enabled ? '<span class="text-xs text-emerald-600 ml-1">(SSO)</span>' : '';

                userMenuContainer.innerHTML = `
                    <div class="flex items-center space-x-4">
                        <span class="text-sm text-gray-700">Benvenuto, <strong>${this.escapeHtml(user.name)}</strong>${ssoBadge}</span>
                        <button onclick="authManager.logout()" 
                            class="text-sm text-gray-600 hover:text-forest transition-colors">
                            Esci
                        </button>
                    </div>
                `;
            } else {
                // If Authelia is enabled, show SSO login button
                if (config.enabled) {
                    userMenuContainer.innerHTML = `
                        <div class="flex items-center space-x-4">
                            <button onclick="authManager.redirectToAutheliaLogin()"
                                class="text-sm bg-forest text-white px-4 py-2 rounded-lg hover:bg-forest/90 transition-colors flex items-center">
                                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                                        d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                                </svg>
                                Accedi con SSO
                            </button>
                        </div>
                    `;
                } else {
                    userMenuContainer.innerHTML = `
                        <div class="flex items-center space-x-4">
                            <a href="/login.html" class="text-sm text-gray-600 hover:text-forest transition-colors">
                                Accedi
                            </a>
                            <a href="/register.html" 
                                class="text-sm bg-forest text-white px-4 py-2 rounded-lg hover:bg-forest/90 transition-colors">
                                Registrati
                            </a>
                        </div>
                    `;
                }
            }
        }
    }

    /**
     * Make an authenticated API request
     */
    async authenticatedFetch(url, options = {}) {
        const token = this.getToken();

        // Even without a token, behind Authelia the request may be authenticated via headers
        const headers = {
            ...options.headers,
        };

        // Add Bearer token if available
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }

        const response = await fetch(url, {
            ...options,
            headers
        });

        // If unauthorized, check if Authelia can help, otherwise logout
        if (response.status === 401) {
            const redirected = await this.redirectToAutheliaLogin();
            if (!redirected) {
                this.setToken(null);
                this.setUser(null);
                // Only redirect if we are not already on the login page
                if (!window.location.pathname.includes('login.html')) {
                    window.location.href = '/login.html';
                }
            }
            throw new Error('Sessione scaduta');
        }

        return response;
    }
}

// Create global instance
window.authManager = new AuthManager();

// Auto-update navbar on page load
document.addEventListener('DOMContentLoaded', () => {
    window.authManager.updateNavbar();
});

// Google Login Callback (must be global)
window.handleGoogleLogin = async (response) => {
    try {
        await window.authManager.loginWithGoogle(response.credential);

        // Get return URL
        const returnUrl = window.authManager.getReturnUrl() ||
            new URLSearchParams(window.location.search).get('return') ||
            '/dashboard.html';

        window.location.href = returnUrl;
    } catch (error) {
        console.error('Google login handle error:', error);
        // Try to show error on login page
        const errorDiv = document.getElementById('error-message');
        if (errorDiv) {
            errorDiv.textContent = error.message;
            errorDiv.classList.remove('hidden');
        } else {
            alert('Login fallito: ' + error.message);
        }
    }
};
