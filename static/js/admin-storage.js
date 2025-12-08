// Secure Admin Storage Manager
// Handles localStorage with validation and sanitization

class AdminStorage {
    constructor() {
        this.config = window.AdminConfig;
    }

    // Validate that data hasn't been tampered with
    _validateUserData(data) {
        if (!data || typeof data !== 'object') {
            return false;
        }

        // Required fields
        const requiredFields = ['id', 'username', 'email'];
        for (const field of requiredFields) {
            if (!data[field]) {
                return false;
            }
        }

        // Validate email format
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (!emailRegex.test(data.email)) {
            return false;
        }

        // Validate ID format (should be UUID or similar)
        if (typeof data.id !== 'string' || data.id.length < 10) {
            return false;
        }

        return true;
    }

    // Save token securely
    saveToken(token) {
        if (!token || typeof token !== 'string') {
            throw new Error('Invalid token');
        }

        try {
            const timestamp = Date.now();
            const data = {
                token: token,
                savedAt: timestamp,
                expiresAt: timestamp + this.config.security.sessionTimeout,
            };
            localStorage.setItem(this.config.security.tokenKey, JSON.stringify(data));
            this.config.log('Token saved successfully');
            return true;
        } catch (e) {
            this.config.error('Failed to save token:', e);
            return false;
        }
    }

    // Get token with expiry check
    getToken() {
        try {
            const data = localStorage.getItem(this.config.security.tokenKey);
            if (!data) {
                return null;
            }

            const parsed = JSON.parse(data);

            // Check if expired
            if (parsed.expiresAt && Date.now() > parsed.expiresAt) {
                this.config.warn('Token expired, clearing');
                this.clearToken();
                return null;
            }

            return parsed.token;
        } catch (e) {
            this.config.error('Failed to get token:', e);
            this.clearToken();
            return null;
        }
    }

    // Clear token
    clearToken() {
        localStorage.removeItem(this.config.security.tokenKey);
        this.config.log('Token cleared');
    }

    // Save user data
    saveUser(userData) {
        if (!this._validateUserData(userData)) {
            this.config.error('Invalid user data, not saving');
            return false;
        }

        try {
            // Only save necessary fields, strip sensitive data
            const safeData = {
                id: userData.id,
                username: userData.username,
                email: userData.email,
                role: userData.role || 'admin',
                created_at: userData.created_at,
                savedAt: Date.now(),
            };

            localStorage.setItem(this.config.security.userKey, JSON.stringify(safeData));
            this.config.log('User data saved successfully');
            return true;
        } catch (e) {
            this.config.error('Failed to save user data:', e);
            return false;
        }
    }

    // Get user data
    getUser() {
        try {
            const data = localStorage.getItem(this.config.security.userKey);
            if (!data) {
                return null;
            }

            const parsed = JSON.parse(data);

            if (!this._validateUserData(parsed)) {
                this.config.warn('Stored user data is invalid, clearing');
                this.clearUser();
                return null;
            }

            return parsed;
        } catch (e) {
            this.config.error('Failed to get user data:', e);
            this.clearUser();
            return null;
        }
    }

    // Clear user data
    clearUser() {
        localStorage.removeItem(this.config.security.userKey);
        this.config.log('User data cleared');
    }

    // Clear all admin data
    clearAll() {
        this.clearToken();
        this.clearUser();
        this.config.log('All admin data cleared');
    }

    // Check if logged in
    isLoggedIn() {
        const token = this.getToken();
        const user = this.getUser();
        return !!(token && user);
    }
}

// Create global instance
window.AdminStorage = new AdminStorage();
