// Authentication State Management

class AuthManager {
    constructor() {
        this.token = localStorage.getItem('authToken');
        this.user = this.getCurrentUser();
    }

    // Get current user from localStorage
    getCurrentUser() {
        const userStr = localStorage.getItem('currentUser');
        if (!userStr) return null;

        try {
            return JSON.parse(userStr);
        } catch (e) {
            console.error('Failed to parse user data:', e);
            return null;
        }
    }

    // Check if user is logged in
    isLoggedIn() {
        return !!this.token && !!this.user;
    }

    // Get auth token
    getToken() {
        return this.token;
    }

    // Logout
    async logout() {
        if (this.token) {
            try {
                await fetch(`/api/auth/logout/${this.token}`, {
                    method: 'POST'
                });
            } catch (e) {
                console.error('Logout error:', e);
            }
        }

        localStorage.removeItem('authToken');
        localStorage.removeItem('currentUser');
        localStorage.removeItem('rememberMe');

        this.token = null;
        this.user = null;

        window.location.href = '/login.html';
    }

    // Verify session is still valid
    async verifySession() {
        if (!this.token) return false;

        try {
            const response = await fetch(`/api/auth/me/${this.token}`);

            if (!response.ok) {
                // Session invalid, clear and redirect
                this.logout();
                return false;
            }

            const user = await response.json();
            localStorage.setItem('currentUser', JSON.stringify(user));
            this.user = user;
            return true;

        } catch (e) {
            console.error('Session verification error:', e);
            return false;
        }
    }

    // Require authentication (redirect to login if not logged in)
    requireAuth(returnUrl = null) {
        if (!this.isLoggedIn()) {
            const url = returnUrl || window.location.pathname;
            window.location.href = `/login.html?return=${encodeURIComponent(url)}`;
            return false;
        }
        return true;
    }

    // Update user display in navigation
    updateUserDisplay() {
        const userDisplay = document.getElementById('user-display');
        const loginLink = document.getElementById('login-link');
        const logoutBtn = document.getElementById('logout-btn');

        if (this.isLoggedIn()) {
            if (userDisplay) {
                userDisplay.innerHTML = `
                    <div class="flex items-center space-x-2">
                        <div class="w-8 h-8 bg-gradient-to-br from-emerald to-mystic rounded-full flex items-center justify-center">
                            <span class="text-white font-semibold text-sm">${this.user.name.charAt(0).toUpperCase()}</span>
                        </div>
                        <div class="text-sm">
                            <div class="font-semibold text-forest">${this.user.name}</div>
                            <div class="text-xs text-gray-500">${this.user.email}</div>
                        </div>
                    </div>
                `;
                userDisplay.style.display = 'flex';
            }

            if (loginLink) loginLink.style.display = 'none';
            if (logoutBtn) {
                logoutBtn.style.display = 'block';
                logoutBtn.onclick = () => this.logout();
            }
        } else {
            if (userDisplay) userDisplay.style.display = 'none';
            if (loginLink) loginLink.style.display = 'block';
            if (logoutBtn) logoutBtn.style.display = 'none';
        }
    }
}

// Global auth manager instance
window.authManager = new AuthManager();

// Auto-verify session on page load
document.addEventListener('DOMContentLoaded', async () => {
    if (window.authManager.isLoggedIn()) {
        await window.authManager.verifySession();
        window.authManager.updateUserDisplay();
    }
});
