// Authentication Utilities with Authelia SSO Support
class AuthManager {
    constructor() {
        this.TOKEN_KEY = 'dnd_session_token';
        this.USER_KEY = 'dnd_user_data';
        this.AUTHELIA_CONFIG_KEY = 'authelia_config';
        this._autheliaConfig = null;
    }

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
            return true;
        }

        // Fall back to local token verification
        const token = this.getToken();
        if (!token) {
            return false;
        }

        try {
            const response = await fetch(`/api/auth/me/${token}`);
            if (response.ok) {
                const user = await response.json();
                this.setUser(user);
                return true;
            } else {
                // Session invalid, clear local data
                this.setToken(null);
                this.setUser(null);
                return false;
            }
        } catch (error) {
            console.error('Session verification failed:', error);
            return false;
        }
    }

    /**
     * Require authentication - redirect to login if not authenticated
     * Supports Authelia SSO redirect when enabled
     * @param {string} redirectUrl - Optional URL to redirect back to after login
     */
    async requireAuth(redirectUrl = null) {
        const isValid = await this.verifySession();

        if (!isValid) {
            // Store the intended destination
            const returnUrl = redirectUrl || window.location.pathname + window.location.search;
            sessionStorage.setItem('auth_return_url', returnUrl);

            // Try Authelia redirect first
            const redirected = await this.redirectToAutheliaLogin();
            if (!redirected) {
                // Fall back to local login page
                window.location.href = '/login.html';
            }
            return false;
        }

        return true;
    }

    /**
     * Login with credentials (for non-Authelia authentication)
     */
    async login(email, password) {
        try {
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ email, password })
            });

            if (!response.ok) {
                const error = await response.text();
                throw new Error(error || 'Login failed');
            }

            const data = await response.json();
            this.setToken(data.token);
            this.setUser(data.user);

            return data;
        } catch (error) {
            console.error('Login error:', error);
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
                throw new Error(error || 'Registration failed');
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

    /**
     * Get the return URL after login
     */
    getReturnUrl() {
        const returnUrl = sessionStorage.getItem('auth_return_url');
        sessionStorage.removeItem('auth_return_url');
        return returnUrl || '/dashboard.html';
    }

    /**
     * Update user profile in navbar (with Authelia support)
     */
    async updateNavbar() {
        const user = this.getUser();
        const config = await this.getAutheliaConfig();
        const userMenuContainer = document.getElementById('user-menu');

        if (!userMenuContainer) return;

        if (user) {
            // Show SSO badge if authenticated via Authelia
            const ssoBadge = config.enabled ? '<span class="text-xs text-emerald-600 ml-1">(SSO)</span>' : '';

            userMenuContainer.innerHTML = `
                <div class="flex items-center space-x-4">
                    <span class="text-sm text-gray-700">Welcome, <strong>${this.escapeHtml(user.name)}</strong>${ssoBadge}</span>
                    <button onclick="authManager.logout()" 
                        class="text-sm text-gray-600 hover:text-forest transition-colors">
                        Logout
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
                            Login
                        </a>
                        <a href="/register.html" 
                            class="text-sm bg-forest text-white px-4 py-2 rounded-lg hover:bg-forest/90 transition-colors">
                            Sign Up
                        </a>
                    </div>
                `;
            }
        }
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
                window.location.href = '/login.html';
            }
            throw new Error('Session expired');
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
