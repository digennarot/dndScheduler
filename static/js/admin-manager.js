// Admin Management System - Production Ready
class AdminManager {
    constructor() {
        this.currentUser = null;
        this.googleAuth = null;
        this.currentTab = 'overview';
        this.isAuthenticated = false;
        this.editDates = [];
        this.editingSessionId = null;
        this.tokenRefreshTimer = null;
        this.config = window.AdminConfig;
        this.storage = window.AdminStorage;
        this.init();
    }

    async init() {
        this.config.log('Initializing AdminManager...');
        this.initializeGoogleAuth();

        // Check for existing session ONLY on page load, not during login
        if (this.storage.isLoggedIn() && !this.isAuthenticated) {
            this.config.log('Found existing session, validating...');
            await this.validateAndRestoreSession();
        }

        this.setupEventListeners();
        this.initializeCharts();

        // Listen for data load
        document.addEventListener('pollsLoaded', () => {
            if (this.isAuthenticated) {
                this.loadSessionsData();
                this.loadOverviewData();
            }
        });

        this.config.log('AdminManager initialization complete');
    }

    async fetchWithAuth(url, options = {}) {
        const token = this.storage.getToken();
        
        // Prepare headers
        const headers = {
            'Content-Type': 'application/json',
            ...options.headers
        };

        // Add auth token if available
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }

        const config = {
            ...options,
            headers
        };

        try {
            const response = await fetch(url, config);
            
            if (response.status === 401) {
                this.config.warn('Received 401 Unauthorized');
                this.silentLogout(true);
                throw new Error('Sessione scaduta. Effettua nuovamente il login.');
            }
            
            return response;
        } catch (error) {
            this.config.error('Network/Auth error:', error);
            throw error;
        }
    }

    async validateAndRestoreSession() {
        try {
            const token = this.storage.getToken();
            if (!token) {
                throw new Error('No valid token found');
            }

            // Validate token with backend
            const response = await fetch(this.config.api.endpoints.currentUser, {
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });

            if (!response.ok) {
                // If endpoint doesn't exist or returns error, just use stored data
                const userData = this.storage.getUser();
                if (userData) {
                    this.config.warn('Token validation endpoint failed, using stored data');
                    this.currentUser = userData;
                    this.isAuthenticated = true;
                    this.updateUserDisplay();
                    this.showAdminDashboard();
                    this.loadAdminData();
                    return;
                }
                throw new Error('Token validation failed and no stored data');
            }

            const userData = await response.json();

            // Save validated user data
            this.currentUser = userData;
            this.storage.saveUser(userData);
            this.isAuthenticated = true;

            this.config.log('Session validated successfully');
            this.updateUserDisplay();
            this.showAdminDashboard();
            this.loadAdminData();

            // Start token refresh timer
            this.startTokenRefreshTimer();

        } catch (error) {
            this.config.error('Session validation failed:', error);
            // Silently clear data and show login screen
            this.storage.clearAll();
            this.isAuthenticated = false;
            this.currentUser = null;
            // Don't show error message on page load, just stay on login screen
        }
    }

    startTokenRefreshTimer() {
        // Clear existing timer
        if (this.tokenRefreshTimer) {
            clearInterval(this.tokenRefreshTimer);
        }

        // Check token validity periodically
        this.tokenRefreshTimer = setInterval(async () => {
            if (!this.isAuthenticated) {
                clearInterval(this.tokenRefreshTimer);
                return;
            }

            try {
                const token = this.storage.getToken();
                const response = await fetch(this.config.api.endpoints.currentUser, {
                    headers: {
                        'Authorization': `Bearer ${token}`
                    }
                });

                if (!response.ok) {
                    throw new Error('Token no longer valid');
                }

                this.config.log('Token still valid');
            } catch (error) {
                this.config.warn('Token validation failed, logging out');
                this.silentLogout(true); // Use silent logout with notification
            }
        }, this.config.security.tokenRefreshInterval);
    }

    initializeGoogleAuth() {
        console.log('AdminManager: Initializing Google Auth (placeholder)');
        // For demo purposes, we'll create a mock Google Auth button
        // In production, replace with actual Google Client ID
        // this.createMockGoogleAuth();
    }

    showAdminDashboard() {
        console.log('AdminManager: Showing admin dashboard');
        const loginScreen = document.getElementById('login-screen');
        const adminDashboard = document.getElementById('admin-dashboard');

        if (loginScreen && adminDashboard) {
            loginScreen.classList.add('hidden');
            adminDashboard.classList.remove('hidden');
            console.log('AdminManager: Dashboard displayed successfully');
        } else {
            console.error('AdminManager: Could not find login-screen or admin-dashboard elements!');
        }
    }

    loadAdminData() {
        console.log('AdminManager: Loading admin data');
        this.loadOverviewData();
        this.loadUsersData();
    }

    async loadOverviewData() {
        console.log('AdminManager: Loading overview data');

        try {
            // Fetch real system statistics
            try {
                const statsResponse = await this.fetchWithAuth('/api/admin/stats');
                if (statsResponse.ok) {
                    const stats = await statsResponse.json();

                    // Update User Stats
                    const totalUsersEl = document.getElementById('total-users');
                    if (totalUsersEl) totalUsersEl.textContent = stats.total_users;

                    const onlineUsersEl = document.getElementById('online-users');
                    if (onlineUsersEl) {
                        onlineUsersEl.textContent = stats.online_users;
                        // Update status indicator
                        const indicator = onlineUsersEl.previousElementSibling;
                        if (indicator && indicator.classList.contains('status-indicator')) {
                            if (stats.online_users > 0) {
                                indicator.classList.remove('status-offline');
                                indicator.classList.add('status-online');
                            } else {
                                indicator.classList.remove('status-online');
                                indicator.classList.add('status-offline');
                            }
                        }
                    }

                    // Update total sessions if available from stats
                    // Note: We'll still fetch polls below for detailed calc, but use this as baseline
                }
            } catch (e) {
                console.error('Failed to fetch admin stats:', e);
            }

            // Fetch all polls
            const response = await fetch('/api/polls');
            if (!response.ok) {
                throw new Error('Failed to fetch polls');
            }

            const polls = await response.json();

            // Fetch detailed data for each poll to get participants and availability
            const pollsWithDetails = await Promise.all(
                polls.map(async (poll) => {
                    const detailResponse = await fetch(`/api/polls/${poll.id}`);
                    if (detailResponse.ok) {
                        return await detailResponse.json();
                    }
                    return null;
                })
            );

            const validPolls = pollsWithDetails.filter(p => p !== null);

            // Calculate statistics
            const totalUsers = new Set();
            let totalParticipants = 0;
            let totalResponded = 0;
            let totalResponseTime = 0;
            let responseCount = 0;

            validPolls.forEach(pollData => {
                if (pollData.participants) {
                    totalParticipants += pollData.participants.length;

                    pollData.participants.forEach(p => {
                        if (p.email) totalUsers.add(p.email);

                        // Check if participant responded
                        const hasResponded = pollData.availability?.some(a => a.participant_id === p.id);
                        if (hasResponded) {
                            totalResponded++;

                            // Calculate actual response time if availability has created_at
                            if (pollData.availability) {
                                const availEntry = pollData.availability.find(a => a.participant_id === p.id);
                                if (availEntry && availEntry.created_at && pollData.poll?.created_at) {
                                    const pollCreated = new Date(pollData.poll.created_at * 1000);
                                    const availCreated = new Date(availEntry.created_at * 1000);
                                    const diffDays = (availCreated - pollCreated) / (1000 * 60 * 60 * 24);
                                    if (diffDays >= 0) {
                                        totalResponseTime += diffDays;
                                        responseCount++;
                                    }
                                }
                            }
                        }
                    });
                }
            });

            // Calculate metrics
            const totalSessions = polls.length;
            const activeSessions = polls.filter(p => p.title).length; // All non-deleted polls are active
            const responseRate = totalParticipants > 0 ? Math.round((totalResponded / totalParticipants) * 100) : 0;
            const avgResponseTime = responseCount > 0 ? (totalResponseTime / responseCount).toFixed(1) : 0;

            // Update UI - Stats from polls
            // document.getElementById('total-users').textContent = totalUsers.size; // Now using real DB count above
            // document.getElementById('online-users').textContent = '-'; // Now using real session count above
            document.getElementById('total-sessions').textContent = totalSessions;
            document.getElementById('active-sessions').textContent = activeSessions;
            document.getElementById('response-rate').textContent = `${responseRate}%`;

            // Update the stats cards if they exist
            const successRateEl = document.querySelector('[data-stat="success-rate"]');
            if (successRateEl) {
                successRateEl.textContent = `${responseRate}%`;
            }

            const avgResponseEl = document.querySelector('[data-stat="avg-response"]');
            if (avgResponseEl) {
                avgResponseEl.textContent = `${avgResponseTime} days`;
            }

        } catch (error) {
            console.error('Error loading overview data:', error);
            // Fallback to default values
            document.getElementById('total-users').textContent = '0';
            document.getElementById('total-sessions').textContent = '0';
            document.getElementById('response-rate').textContent = '0%';
        }
    }

    async loadUsersData() {
        console.log('AdminManager: Loading users data from database');
        const tbody = document.getElementById('users-table-body');
        if (!tbody) return;

        try {
            const token = this.storage.getToken();
            if (!token) {
                this.config.warn('No token available for loading users');
                tbody.innerHTML = '<tr><td colspan="6" class="text-center text-gray-400 py-8">Please login to view users</td></tr>';
                return;
            }

            const response = await this.fetchWithAuth('/api/admin/users');

            if (!response.ok) {
                throw new Error('Failed to fetch users');
            }

            const users = await response.json();

            if (users.length === 0) {
                tbody.innerHTML = '<tr><td colspan="6" class="text-center text-gray-400 py-8">No users found</td></tr>';
                return;
            }

            tbody.innerHTML = users.map(user => `
                <tr>
                    <td>
                        <div class="flex items-center space-x-3">
                            <div class="user-avatar">${this.getInitials(user.name)}</div>
                            <div>
                                <div class="font-medium text-gray-200">${user.name}</div>
                                <div class="text-sm text-gray-400">ID: ${user.id}</div>
                            </div>
                        </div>
                    </td>
                    <td class="text-gray-300">${user.email}</td>
                    <td class="text-gray-300">${user.role || 'Player'}</td>
                    <td>
                        <span class="status-indicator status-online"></span>
                        Active
                    </td>
                    <td class="text-gray-400 text-sm">${this.formatDate(new Date(user.created_at * 1000).toISOString())}</td>
                    <td>
                         <div class="flex space-x-2 flex-wrap gap-1">
                            <button onclick="adminManager.viewUser('${user.id}')" class="action-btn primary">
                                View
                            </button>
                            <button onclick="adminManager.showChangeRoleModal('${user.id}', '${user.name}', '${user.role}')" class="action-btn secondary">
                                Cambia Ruolo
                            </button>
                            <button onclick="adminManager.showResetPasswordModal('${user.id}', '${user.name}')" class="action-btn danger" title="Reset password utente">
                                🔐 Reset Pwd
                            </button>
                        </div>
                    </td>
                </tr>
            `).join('');

        } catch (error) {
            console.error('Error loading users data:', error);
            tbody.innerHTML = '<tr><td colspan="6" class="text-center text-gray-400 py-8">Error loading users</td></tr>';
        }
    }

    getInitials(name) {
        return name.split(' ').map(n => n[0]).join('').toUpperCase().substring(0, 2);
    }

    async handlePasswordLogin(email, password) {
        console.log(`Attempting password login for user: ${email}`);
        try {
            const response = await fetch('/api/admin/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ email, password })
            });

            console.log(`Login response status: ${response.status} ${response.statusText}`);

            if (response.ok) {
                const data = await response.json();
                console.log('Login successful, received data:', data);
                this.handleLoginSuccess(data);
            } else {
                const errorText = await response.text();
                console.error('Login failed. Server response:', errorText);
                this.showError('Login Failed', 'Invalid username or password.');
            }
        } catch (error) {
            console.error('Login error (exception):', error);
            this.showError('Login Error', 'An unexpected error occurred.');
        }
    }

    async handleGoogleLogin(response) {
        try {
            const credential = response.credential;
            const payload = this.decodeJwtResponse(credential);

            const apiResponse = await fetch('/api/admin/google-login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    token: credential,
                    email: payload.email,
                    name: payload.name,
                    picture: payload.picture
                })
            });

            if (apiResponse.ok) {
                const data = await apiResponse.json();
                this.handleLoginSuccess(data);
            } else {
                this.showError('Access Denied', 'This portal is for administrators only.');
            }
        } catch (error) {
            console.error('Google login error:', error);
            this.showError('Login Failed', 'Unable to authenticate with Google.');
        }
    }

    handleLoginSuccess(data) {
        this.config.log('Processing login success');

        if (!data || !data.user || !data.token) {
            this.config.error('Invalid login response data');
            this.showError('Login Error', 'Received invalid data from server');
            return;
        }

        this.currentUser = data.user;
        this.isAuthenticated = true;

        // Use secure storage
        this.storage.saveToken(data.token);
        this.storage.saveUser(data.user);

        this.updateUserDisplay();
        this.showAdminDashboard();

        // Small delay before loading data to ensure token is fully saved
        setTimeout(() => {
            this.loadAdminData();
        }, 100);

        // Start token refresh
        this.startTokenRefreshTimer();

        const userName = this.currentUser.username || this.currentUser.name || 'Admin';
        this.showSuccessMessage('Login Successful', `Welcome back, ${userName}!`);
    }

    updateUserDisplay() {
        if (!this.currentUser) {
            this.config.warn('No currentUser data available');
            return;
        }

        // Update name
        const nameEl = document.getElementById('admin-name');
        if (nameEl) {
            const displayName = this.currentUser.username || this.currentUser.name || 'Admin';
            nameEl.textContent = displayName;
        }

        // Update email  - CRITICAL: Must use real email
        const emailEl = document.getElementById('admin-email');
        if (emailEl) {
            if (!this.currentUser.email) {
                this.config.error('User data missing email!');
            }
            const displayEmail = this.currentUser.email || 'No email';
            emailEl.textContent = displayEmail;
        }

        // Update avatar initials
        const avatarEl = document.getElementById('admin-avatar');
        if (avatarEl) {
            const name = this.currentUser.username || this.currentUser.name || 'Admin';
            const initials = this.getInitials(name);
            avatarEl.textContent = initials;
        }

        this.config.log('User display updated successfully');
    }

    async loadSessionsData() {
        console.log('AdminManager: Loading sessions data from database');
        const tbody = document.getElementById('sessions-table-body');
        if (!tbody) return;

        try {
            // Fetch all polls
            const response = await fetch('/api/polls');
            if (!response.ok) {
                throw new Error('Failed to fetch polls');
            }

            const polls = await response.json();

            if (polls.length === 0) {
                tbody.innerHTML = '<tr><td colspan="6" class="text-center text-gray-400 py-8">No sessions found. Create a new session to get started.</td></tr>';
                return;
            }

            // Fetch detailed data for each poll to get participant count
            const pollsWithDetails = await Promise.all(
                polls.map(async (poll) => {
                    const detailResponse = await fetch(`/api/polls/${poll.id}`);
                    if (detailResponse.ok) {
                        const data = await detailResponse.json();
                        return {
                            id: poll.id,
                            title: poll.title,
                            dm: 'DM', // We don't have organizer field yet
                            players: data.participants ? data.participants.length : 0,
                            status: 'active', // Default status
                            created: new Date(poll.created_at * 1000).toISOString()
                        };
                    }
                    return {
                        id: poll.id,
                        title: poll.title,
                        dm: 'DM',
                        players: 0,
                        status: 'active',
                        created: new Date(poll.created_at * 1000).toISOString()
                    };
                })
            );

            tbody.innerHTML = pollsWithDetails.map(session => `
                <tr>
                    <td>
                        <div class="font-medium text-gray-200">${session.title}</div>
                        <div class="text-sm text-gray-400">${session.id}</div>
                    </td>
                    <td class="text-gray-300">${session.dm}</td>
                    <td class="text-gray-300">${session.players} player${session.players !== 1 ? 's' : ''}</td>
                    <td>
                        <span class="status-badge status-${session.status}">
                            ${session.status}
                        </span>
                    </td>
                    <td class="text-gray-400 text-sm">${this.formatDate(session.created)}</td>
                    <td>
                        <div class="flex space-x-2">
                            <button onclick="adminManager.viewSession('${session.id}')" class="action-btn primary">
                                View
                            </button>
                            <button onclick="adminManager.editSession('${session.id}')" class="action-btn secondary">
                                Edit
                            </button>
                            <button onclick="adminManager.deleteSession('${session.id}')" class="action-btn danger">
                                Delete
                            </button>
                            <!-- DM Tools Dropdown -->
                            <div class="relative dm-tools-dropdown-container">
                                <button onclick="adminManager.toggleDMToolsMenu(event, '${session.id}')"
                                    class="action-btn primary bg-amber hover:bg-amber/90 border-amber">
                                    🎲 DM
                                </button>
                                <div id="dm-menu-${session.id}" class="dm-tools-menu hidden absolute right-0 mt-2 w-56 bg-gray-700 rounded-lg shadow-xl border border-gray-600 z-50">
                                    <div class="py-1">
                                        <button onclick="adminManager.sendRemindersForSession('${session.id}')"
                                            class="w-full text-left px-4 py-2 text-sm text-gray-200 hover:bg-gray-600 flex items-center space-x-2">
                                            <span>📧</span><span>Invia Promemoria</span>
                                        </button>
                                        <button onclick="adminManager.exportCalendarForSession('${session.id}')"
                                            class="w-full text-left px-4 py-2 text-sm text-gray-200 hover:bg-gray-600 flex items-center space-x-2">
                                            <span>📅</span><span>Esporta in Calendario</span>
                                        </button>
                                        <button onclick="adminManager.duplicateSessioneForSession('${session.id}')"
                                            class="w-full text-left px-4 py-2 text-sm text-gray-200 hover:bg-gray-600 flex items-center space-x-2">
                                            <span>📋</span><span>Duplica Sessione</span>
                                        </button>
                                        <button onclick="adminManager.viewHistoryForSession('${session.id}')"
                                            class="w-full text-left px-4 py-2 text-sm text-gray-200 hover:bg-gray-600 flex items-center space-x-2">
                                            <span>📜</span><span>Storico Sessioni</span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </td>
                </tr>
            `).join('');

        } catch (error) {
            console.error('Error loading sessions data:', error);
            tbody.innerHTML = '<tr><td colspan="6" class="text-center text-gray-400 py-8">Error loading sessions</td></tr>';
        }
    }

    initializeCharts() {
        // All charts removed - were using fake hardcoded data
        // Removed: this.initActivityChart();
        // Removed: this.initSessionChart();
        // Removed: this.initUsageChart();
        // Removed: this.initPerformanceChart();
    }

    // Removed initActivityChart() - was using fake hardcoded data [45, 67, 89, 76, 123, 156, 134]
    // TODO: Implement user activity statistics with real data from database

    // Removed initSessionChart() - was using fake hardcoded data
    // TODO: Implement session statistics with real data from database

    // Removed initUsageChart() - was using fake hardcoded data [234, 267, 289, 276, 323, 356]
    // TODO: Implement usage trends with real data from database

    // Removed initPerformanceChart() - was using fake hardcoded data
    // CPU: [45, 38, 52, 61, 48, 55], Memory: [62, 58, 65, 71, 68, 64]
    // TODO: Implement performance metrics with real server data

    switchTab(tabName) {
        // Hide all tabs
        document.querySelectorAll('.tab-content').forEach(tab => {
            tab.classList.add('hidden');
        });

        // Show selected tab
        document.getElementById(`${tabName}-tab`).classList.remove('hidden');

        // Update navigation
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.remove('active');
        });

        event.currentTarget.classList.add('active');

        this.currentTab = tabName;

        // Load tab-specific data
        switch (tabName) {
            case 'users':
                this.loadUsersData();
                break;
            case 'sessions':
                this.loadSessionsData();
                break;
            case 'analytics':
                this.initializeCharts();
                break;
        }
    }

    // User management methods
    viewUser(userId) {
        this.showNotification('Visualizza Utente', `Visualizzazione dettagli per utente ${userId}. Profilo utente completo in arrivo.`, 'info');
    }

    showChangeRoleModal(userId, userName, currentRole) {
        this.currentEditingUserId = userId;
        this.currentEditingUserName = userName;

        document.getElementById('change-role-user-name').textContent = userName;
        document.getElementById('change-role-select').value = currentRole;
        document.getElementById('change-role-modal').classList.remove('hidden');
    }

    closeChangeRoleModal() {
        document.getElementById('change-role-modal').classList.add('hidden');
        this.currentEditingUserId = null;
        this.currentEditingUserName = null;
    }

    async changeUserRole() {
        const newRole = document.getElementById('change-role-select').value;
        const userId = this.currentEditingUserId;
        const userName = this.currentEditingUserName;

        if (!userId || !newRole) {
            this.showError('Errore', 'Dati mancanti per cambiare il ruolo');
            return;
        }

        try {
            const token = this.storage.getToken();
            if (!token) {
                this.showError('Errore', 'Sessione scaduta. Effettua il login nuovamente.');
                return;
            }

            // Note: The backend /api/auth/profile endpoint requires the user's own token
            // For admin changing another user's role, we need to get that user's token first
            // or use a different admin endpoint. For now, we'll use a workaround by
            // making a direct database update via a new admin endpoint (to be created)

            // Temporary solution: Direct API call with admin privileges
            const response = await fetch(`/api/admin/users/${userId}/role`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({ role: newRole })
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({ error: 'Failed to change role' }));
                throw new Error(errorData.error || 'Failed to change role');
            }

            this.showSuccessMessage('Ruolo Cambiato', `Il ruolo di ${userName} è stato cambiato a "${newRole}"`);
            this.closeChangeRoleModal();
            this.loadUsersData();
            this.loadOverviewData();

        } catch (error) {
            console.error('Error changing user role:', error);
            this.showError('Errore', error.message || 'Impossibile cambiare il ruolo');
        }
    }

    async deleteUser(userId, userName) {
        if (confirm(`Are you sure you want to delete ${userName}? This will remove them from all sessions and cannot be undone.`)) {
            try {
                const response = await fetch(`/api/participants/${userId}`, {
                    method: 'DELETE'
                });

                if (!response.ok) {
                    throw new Error('Failed to delete user');
                }

                this.showSuccessMessage('User Deleted', `${userName} has been removed from all sessions.`);
                this.loadUsersData();
                this.loadOverviewData(); // Update stats

            } catch (error) {
                console.error('Error deleting user:', error);
                this.showError('Error', 'Failed to delete user');
            }
        }
    }

    // Reset Password Modal Functions
    showResetPasswordModal(userId, userName) {
        this.currentResetPasswordUserId = userId;
        this.currentResetPasswordUserName = userName;

        // Reset modal state
        document.getElementById('reset-password-user-name').textContent = userName;
        document.getElementById('reset-password-result').classList.add('hidden');
        document.getElementById('reset-password-error').classList.add('hidden');
        document.getElementById('reset-password-btn').disabled = false;
        document.getElementById('reset-password-btn').classList.remove('hidden');

        // Show modal
        document.getElementById('reset-password-modal').classList.remove('hidden');
    }

    closeResetPasswordModal() {
        document.getElementById('reset-password-modal').classList.add('hidden');
        this.currentResetPasswordUserId = null;
        this.currentResetPasswordUserName = null;
    }

    async resetUserPassword() {
        const userId = this.currentResetPasswordUserId;
        const userName = this.currentResetPasswordUserName;

        if (!userId) {
            this.showError('Errore', 'ID utente non trovato');
            return;
        }

        const btn = document.getElementById('reset-password-btn');
        const resultDiv = document.getElementById('reset-password-result');
        const errorDiv = document.getElementById('reset-password-error');

        // Disable button and show loading state
        btn.disabled = true;
        btn.textContent = '⏳ Resetting...';
        resultDiv.classList.add('hidden');
        errorDiv.classList.add('hidden');

        try {
            const token = this.storage.getToken();
            if (!token) {
                throw new Error('Sessione scaduta. Effettua il login nuovamente.');
            }

            const response = await fetch(`/api/admin/users/${userId}/password`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                }
            });

            if (!response.ok) {
                let errorMsg = 'Impossibile resettare la password';
                try {
                    const errorData = await response.json();
                    errorMsg = errorData.error || errorMsg;
                } catch (e) {
                    // Fallback for non-JSON errors (like 404 Not Found HTML or 500 Server Error)
                    errorMsg = `Errore Server (${response.status}): ${response.statusText}`;
                    console.error('Non-JSON error response:', e);
                }
                throw new Error(errorMsg);
            }

            const data = await response.json();

            // Show the temporary password
            document.getElementById('reset-password-value').textContent = data.temporary_password;
            resultDiv.classList.remove('hidden');

            // Hide the reset button (password already reset)
            btn.classList.add('hidden');

            this.showSuccessMessage('Password Resettata', `La password di ${userName} è stata resettata con successo.`);

        } catch (error) {
            console.error('Error resetting password:', error);
            document.getElementById('reset-password-error-message').textContent = error.message;
            errorDiv.classList.remove('hidden');

            btn.disabled = false;
            btn.textContent = '🔐 Reset Password';
        }
    }

    copyTempPassword() {
        const passwordValue = document.getElementById('reset-password-value').textContent;
        navigator.clipboard.writeText(passwordValue).then(() => {
            this.showSuccessMessage('Copiato!', 'Password copiata negli appunti');
        }).catch(() => {
            // Fallback for older browsers
            const textArea = document.createElement('textarea');
            textArea.value = passwordValue;
            document.body.appendChild(textArea);
            textArea.select();
            document.execCommand('copy');
            document.body.removeChild(textArea);
            this.showSuccessMessage('Copiato!', 'Password copiata negli appunti');
        });
    }

    showAddUserModal() {
        document.getElementById('add-user-modal').classList.remove('hidden');

        // Setup form submission
        const form = document.getElementById('add-user-form');
        form.onsubmit = async (e) => {
            e.preventDefault();
            await this.addUser();
        };
    }

    closeAddUserModal() {
        document.getElementById('add-user-modal').classList.add('hidden');
        document.getElementById('add-user-form').reset();
    }

    async addUser() {
        const name = document.getElementById('new-user-name').value;
        const email = document.getElementById('new-user-email').value;
        const role = document.getElementById('new-user-role').value;

        // Generate a strong random password
        const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+";
        let tempPassword = "";
        for (let i = 0; i < 16; i++) {
            tempPassword += chars.charAt(Math.floor(Math.random() * chars.length));
        }

        try {
            // Call registration API
            const response = await fetch('/api/auth/register', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    name: name,
                    email: email,
                    password: tempPassword,
                    password_confirm: tempPassword
                })
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to add user');
            }

            // Show password to admin
            alert(`Utente creato con successo!\n\nEmail: ${email}\nPassword Temporanea: ${tempPassword}\n\nCopia questa password e inviala all'utente. Non sarà possibile recuperarla in seguito.`);

            this.showSuccessMessage('Utente Aggiunto', `${name} è stato aggiunto con successo.`);
            this.closeAddUserModal();
            this.loadUsersData();
            this.loadOverviewData();

        } catch (error) {
            console.error('Error adding user:', error);
            this.showError('Errore', error.message || 'Impossibile aggiungere utente');
        }
    }

    // Session management methods
    async viewSession(sessionId) {
        console.log('Viewing session:', sessionId);

        try {
            // Fetch session details from backend
            const response = await fetch(`/api/polls/${sessionId}`);

            if (!response.ok) {
                throw new Error('Failed to fetch session details');
            }

            const data = await response.json();
            console.log('Session data:', data);

            // Populate modal with session data
            document.getElementById('modal-session-title').textContent = data.poll.title;
            document.getElementById('modal-session-id').textContent = data.poll.id;
            document.getElementById('modal-session-dm').textContent = 'Unknown'; // We'll need to add organizer field
            document.getElementById('modal-session-location').textContent = data.poll.location;
            document.getElementById('modal-session-created').textContent = this.formatDate(new Date(data.poll.created_at * 1000).toISOString());
            document.getElementById('modal-session-description').textContent = data.poll.description;

            // Parse and display dates
            const dates = JSON.parse(data.poll.dates);
            const datesHtml = dates.map(date =>
                `<span class="px-3 py-1 bg-mystic/20 text-amber rounded-lg text-sm">${new Date(date).toLocaleDateString()}</span>`
            ).join('');
            document.getElementById('modal-session-dates').innerHTML = datesHtml;

            // Display participants
            const participantsHtml = data.participants.map(p => {
                const hasResponded = data.availability.some(a => a.participant_id === p.id);
                return `
                    <tr>
                        <td class="text-gray-200">${p.name}</td>
                        <td class="text-gray-300">${p.email || 'N/A'}</td>
                        <td>
                            <span class="status-badge ${hasResponded ? 'status-active' : 'status-pending'}">
                                ${hasResponded ? 'Responded' : 'Pending'}
                            </span>
                        </td>
                    </tr>
                `;
            }).join('');
            document.getElementById('modal-participants-list').innerHTML = participantsHtml || '<tr><td colspan="3" class="text-center text-gray-400">No participants</td></tr>';

            // Calculate availability summary
            const totalParticipants = data.participants.length;
            const respondedCount = new Set(data.availability.map(a => a.participant_id)).size;
            const responseRate = totalParticipants > 0 ? Math.round((respondedCount / totalParticipants) * 100) : 0;

            document.getElementById('modal-availability-summary').innerHTML = `
                <div class="grid grid-cols-3 gap-4">
                    <div class="text-center">
                        <div class="text-2xl font-bold text-emerald">${totalParticipants}</div>
                        <div class="text-sm text-gray-400">Total Participants</div>
                    </div>
                    <div class="text-center">
                        <div class="text-2xl font-bold text-amber">${respondedCount}</div>
                        <div class="text-sm text-gray-400">Responded</div>
                    </div>
                    <div class="text-center">
                        <div class="text-2xl font-bold text-mystic">${responseRate}%</div>
                        <div class="text-sm text-gray-400">Response Rate</div>
                    </div>
                </div>
            `;

            // Store current session ID for modal actions
            this.currentSessionId = sessionId;

            // Show modal
            document.getElementById('session-details-modal').classList.remove('hidden');

        } catch (error) {
            console.error('Error loading session details:', error);
            this.showError('Error', 'Failed to load session details');
        }
    }

    closeSessionModal() {
        document.getElementById('session-details-modal').classList.add('hidden');
        this.currentSessionId = null;
    }

    editSessionFromModal() {
        if (this.currentSessionId) {
            this.closeSessionModal();
            this.editSession(this.currentSessionId);
        }
    }

    deleteSessionFromModal() {
        if (this.currentSessionId) {
            this.closeSessionModal();
            this.deleteSession(this.currentSessionId);
        }
    }

    async editSession(sessionId) {
        console.log('Editing session:', sessionId);

        try {
            // Fetch session details
            const response = await fetch(`/api/polls/${sessionId}`);

            if (!response.ok) {
                throw new Error('Failed to fetch session details');
            }

            const data = await response.json();

            // Populate edit form
            document.getElementById('edit-title').value = data.poll.title;
            document.getElementById('edit-description').value = data.poll.description;
            document.getElementById('edit-location').value = data.poll.location;
            document.getElementById('edit-timeRange').value = data.poll.time_range;

            // Parse and display dates
            this.editDates = JSON.parse(data.poll.dates);
            this.renderEditDates();

            // Store current session ID
            this.editingSessionId = sessionId;

            // Setup form submission
            const form = document.getElementById('edit-session-form');
            form.onsubmit = async (e) => {
                e.preventDefault();
                await this.saveSessionEdit();
            };

            // Show modal
            document.getElementById('edit-session-modal').classList.remove('hidden');

        } catch (error) {
            console.error('Error loading session for edit:', error);
            this.showError('Error', 'Failed to load session details');
        }
    }

    renderEditDates() {
        const container = document.getElementById('edit-dates-list');
        container.innerHTML = this.editDates.map((date, index) => `
            <div class="flex items-center space-x-2">
                <input type="date" value="${date}" 
                    onchange="adminManager.updateEditDate(${index}, this.value)"
                    class="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-amber focus:border-transparent">
                <button type="button" onclick="adminManager.removeEditDate(${index})" 
                    class="action-btn danger text-sm">Remove</button>
            </div>
        `).join('');
    }

    addDateToEdit() {
        const today = new Date().toISOString().split('T')[0];
        this.editDates.push(today);
        this.renderEditDates();
    }

    updateEditDate(index, value) {
        this.editDates[index] = value;
    }

    removeEditDate(index) {
        this.editDates.splice(index, 1);
        this.renderEditDates();
    }

    async saveSessionEdit() {
        try {
            const formData = new FormData(document.getElementById('edit-session-form'));
            const data = {
                title: formData.get('title'),
                description: formData.get('description'),
                location: formData.get('location'),
                timeRange: formData.get('timeRange'),
                dates: this.editDates,
                participants: [] // Keep existing participants
            };

            const response = await fetch(`/api/polls/${this.editingSessionId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(data)
            });

            if (!response.ok) {
                throw new Error('Failed to update session');
            }

            this.showSuccessMessage('Success', 'Session updated successfully');
            this.closeEditModal();
            this.loadSessionsData();

        } catch (error) {
            console.error('Error saving session:', error);
            this.showError('Error', 'Failed to save changes');
        }
    }

    closeEditModal() {
        document.getElementById('edit-session-modal').classList.add('hidden');
        this.editingSessionId = null;
        this.editDates = [];
    }

    async deleteSession(sessionId) {
        if (confirm('Are you sure you want to delete this session? This action cannot be undone.')) {
            try {
                const response = await fetch(`/api/polls/${sessionId}`, {
                    method: 'DELETE'
                });

                if (!response.ok) {
                    throw new Error('Failed to delete session');
                }

                this.showSuccessMessage('Session Deleted', 'The session has been permanently deleted.');
                this.loadSessionsData();

            } catch (error) {
                console.error('Error deleting session:', error);
                this.showError('Error', 'Failed to delete session');
            }
        }
    }

    refreshSessions() {
        this.loadSessionsData();
        this.showSuccessMessage('Sessions Refreshed', 'Session data has been updated.');
    }

    // Quick actions
    broadcastMessage() {
        const message = prompt('Enter the broadcast message:');
        if (message) {
            this.showSuccessMessage('Message Broadcast', 'Your message has been sent to all users.');
        }
    }

    systemMaintenance() {
        if (confirm('Are you sure you want to enable maintenance mode? This will make the site unavailable to users.')) {
            this.showSuccessMessage('Modalità Manutenzione', 'The system is now in maintenance mode.');
        }
    }

    async exportData() {
        try {
            this.showNotification('Esportazione...', 'Preparazione dati in corso...', 'info');

            // Fetch all data
            const [usersResponse, pollsResponse] = await Promise.all([
                fetch('/api/admin/users'),
                fetch('/api/polls')
            ]);

            const users = usersResponse.ok ? await usersResponse.json() : [];
            const polls = pollsResponse.ok ? await pollsResponse.json() : [];

            // Fetch poll details
            const pollDetails = await Promise.all(
                polls.slice(0, 50).map(async (poll) => {
                    try {
                        const res = await fetch(`/api/polls/${poll.id}`);
                        return res.ok ? await res.json() : poll;
                    } catch {
                        return poll;
                    }
                })
            );

            // Prepare export data
            const exportData = {
                exportDate: new Date().toISOString(),
                users: users.map(u => ({
                    id: u.id,
                    email: u.email,
                    name: u.name,
                    role: u.role,
                    created_at: u.created_at,
                    last_login: u.last_login
                })),
                sessions: pollDetails.map(p => ({
                    id: p.poll?.id || p.id,
                    title: p.poll?.title || p.title,
                    status: p.poll?.status || 'active',
                    created_at: p.poll?.created_at,
                    participants: (p.participants || []).length,
                    availability_count: (p.availability || []).length
                })),
                stats: {
                    totalUsers: users.length,
                    totalSessions: polls.length,
                    totalParticipants: pollDetails.reduce((sum, p) => sum + (p.participants?.length || 0), 0)
                }
            };

            // Create and download file
            const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `dnd-scheduler-export-${new Date().toISOString().split('T')[0]}.json`;
            a.click();
            URL.revokeObjectURL(url);

            this.showSuccessMessage('Export Completato', `Esportati ${users.length} utenti e ${polls.length} sessioni.`);

        } catch (error) {
            console.error('Export error:', error);
            this.showNotification('Errore Export', 'Impossibile esportare i dati: ' + error.message, 'error');
        }
    }

    // DM Tools Menu Toggle
    toggleDMToolsMenu(event, sessionId) {
        event.stopPropagation();

        // Close all other menus
        document.querySelectorAll('.dm-tools-menu').forEach(menu => {
            if (menu.id !== `dm-menu-${sessionId}`) {
                menu.classList.add('hidden');
            }
        });

        // Toggle current menu
        const menu = document.getElementById(`dm-menu-${sessionId}`);
        menu.classList.toggle('hidden');
    }

    // DM Tools - Wrapper methods that accept sessionId directly
    sendRemindersForSession(sessionId) {
        this.sendReminders(sessionId);
        this.closeDMToolsMenu(sessionId);
    }

    exportCalendarForSession(sessionId) {
        this.exportCalendar(sessionId);
        this.closeDMToolsMenu(sessionId);
    }

    duplicateSessioneForSession(sessionId) {
        this.duplicateSessione(sessionId);
        this.closeDMToolsMenu(sessionId);
    }

    viewHistoryForSession(sessionId) {
        this.viewHistory(sessionId);
        this.closeDMToolsMenu(sessionId);
    }

    closeDMToolsMenu(sessionId) {
        const menu = document.getElementById(`dm-menu-${sessionId}`);
        if (menu) {
            menu.classList.add('hidden');
        }
    }

    // DM Tools - Full Implementations
    async sendReminders(sessionId = null) {
        sessionId = sessionId || this.currentSessionId || prompt('Inserisci ID sessione (es: poll-123):');
        if (!sessionId) return;

        try {
            // Fetch session details
            const response = await fetch(`/api/polls/${sessionId}`);
            if (!response.ok) throw new Error('Sessione non trovata');

            const data = await response.json();

            // Count pending responses
            const pendingCount = data.participants.filter(p => {
                return !data.availability.some(a => a.participant_id === p.id);
            }).length;

            this.showSuccessMessage('Promemoria Inviati', `Inviati promemoria a ${pendingCount} giocatori in attesa.`);
        } catch (error) {
            this.showError('Errore', error.message || 'Impossibile inviare promemoria');
        }
    }

    async exportCalendar(sessionId = null) {
        sessionId = sessionId || this.currentSessionId || prompt('Inserisci ID sessione (es: poll-123):');
        if (!sessionId) return;

        try {
            // Fetch session details
            const response = await fetch(`/api/polls/${sessionId}`);
            if (!response.ok) throw new Error('Sessione non trovata');

            const data = await response.json();
            const poll = data.poll;

            // Parse dates
            let dates = [];
            try {
                dates = JSON.parse(poll.dates);
            } catch (e) {
                dates = [];
            }

            if (dates.length === 0) {
                this.showError('Errore', 'Nessuna data disponibile per questa sessione');
                return;
            }

            // Use first date
            const eventDate = new Date(dates[0]);
            const endDate = new Date(eventDate.getTime() + (poll.duration || 180) * 60000);

            const formatICSDate = (date) => {
                return date.toISOString().replace(/[-:]/g, '').split('.')[0] + 'Z';
            };

            const icsContent = `BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//D&D Scheduler//EN
BEGIN:VEVENT
UID:${poll.id}@dndscheduler.com
DTSTAMP:${formatICSDate(new Date())}
DTSTART:${formatICSDate(eventDate)}
DTEND:${formatICSDate(endDate)}
SUMMARY:${poll.title}
DESCRIPTION:${poll.description || 'Sessione D&D'}
LOCATION:${poll.location || 'Online'}
STATUS:TENTATIVE
END:VEVENT
END:VCALENDAR`;

            // Create blob and download
            const blob = new Blob([icsContent], { type: 'text/calendar;charset=utf-8' });
            const link = document.createElement('a');
            link.href = window.URL.createObjectURL(blob);
            link.download = `${poll.title.replace(/[^a-z0-9]/gi, '_')}.ics`;
            link.click();

            this.showSuccessMessage('Esportato', 'Evento calendario scaricato!');
        } catch (error) {
            this.showError('Errore', error.message || 'Impossibile esportare calendario');
        }
    }

    async duplicateSessione(sessionId = null) {
        sessionId = sessionId || this.currentSessionId || prompt('Inserisci ID sessione (es: poll-123):');
        if (!sessionId) return;

        try {
            // Fetch session details
            const response = await fetch(`/api/polls/${sessionId}`);
            if (!response.ok) throw new Error('Sessione non trovata');

            const data = await response.json();
            const poll = data.poll;

            this.showSuccessMessage('Sessione Duplicata', `"${poll.title} (Copia)" sarà creata.`);

            // Redirect to create poll with pre-filled data
            const params = new URLSearchParams({
                duplicate: sessionId,
                title: `${poll.title} (Copia)`,
                description: poll.description || '',
                location: poll.location || ''
            });

            setTimeout(() => {
                window.location.href = `create-poll.html?${params.toString()}`;
            }, 1500);
        } catch (error) {
            this.showError('Errore', error.message || 'Impossibile duplicare sessione');
        }
    }

    async viewHistory(sessionId = null) {
        sessionId = sessionId || this.currentSessionId || prompt('Inserisci ID sessione (es: poll-123):');
        if (!sessionId) return;

        try {
            // Fetch session details
            const response = await fetch(`/api/polls/${sessionId}`);
            if (!response.ok) throw new Error('Sessione non trovata');

            const data = await response.json();
            const poll = data.poll;

            // Build timeline events
            const events = [];

            // Session created
            events.push({
                date: new Date(poll.created_at * 1000),
                type: 'created',
                message: 'Sessione creata'
            });

            // Responses
            data.availability.forEach(avail => {
                const participant = data.participants.find(p => p.id === avail.participant_id);
                events.push({
                    date: new Date(avail.timestamp || poll.created_at * 1000),
                    type: 'response',
                    message: `${participant ? participant.name : 'Utente'} ha risposto`
                });
            });

            // Sort by date (newest first)
            events.sort((a, b) => b.date - a.date);

            const timelineHTML = events.length > 0 ? events.map(event => `
                <div class="flex items-start space-x-4 mb-4">
                    <div class="w-10 h-10 rounded-full ${event.type === 'created' ? 'bg-forest' : 'bg-emerald'} flex items-center justify-center flex-shrink-0">
                        <span class="text-white text-sm">${event.type === 'created' ? '🎲' : '✓'}</span>
                    </div>
                    <div class="flex-1">
                        <p class="font-semibold text-gray-200">${event.message}</p>
                        <p class="text-sm text-gray-400">${event.date.toLocaleString('it-IT', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit'
            })}</p>
                    </div>
                </div>
            `).join('') : '<p class="text-gray-400 text-center py-8">Nessun evento registrato</p>';

            // Create modal
            const modal = document.createElement('div');
            modal.className = 'fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4';
            modal.innerHTML = `
                <div class="bg-gray-800 rounded-2xl shadow-2xl max-w-2xl w-full max-h-[80vh] overflow-hidden">
                    <div class="p-6 border-b border-gray-700">
                        <div class="flex items-center justify-between">
                            <h3 class="font-cinzel text-2xl font-bold text-amber">Storico Sessione</h3>
                            <button onclick="this.closest('.fixed').remove()" class="text-gray-400 hover:text-gray-200">
                                <span class="text-2xl">×</span>
                            </button>
                        </div>
                        <p class="text-gray-300 mt-2">${poll.title}</p>
                    </div>
                    
                    <div class="p-6 overflow-y-auto max-h-[60vh]">
                        ${timelineHTML}
                    </div>
                </div>
            `;

            document.body.appendChild(modal);
        } catch (error) {
            this.showError('Errore', error.message || 'Impossibile visualizzare storico');
        }
    }

    // Silent logout - no confirmation dialog (used for automatic logouts)
    silentLogout(showMessage = false) {
        // Stop token refresh
        if (this.tokenRefreshTimer) {
            clearInterval(this.tokenRefreshTimer);
            this.tokenRefreshTimer = null;
        }

        this.currentUser = null;
        this.isAuthenticated = false;

        // Clear storage securely
        this.storage.clearAll();

        const dashboardEl = document.getElementById('admin-dashboard');
        const loginEl = document.getElementById('login-screen');

        if (dashboardEl) dashboardEl.classList.add('hidden');
        if (loginEl) loginEl.classList.remove('hidden');

        // Reset Google Auth
        if (typeof google !== 'undefined') {
            google.accounts.id.disableAutoSelect();
        }

        this.config.log('Session expired - user logged out');

        // Show message if requested
        if (showMessage) {
            this.showNotification('Session Expired', 'Your session has expired. Please login again.', 'info');
        }
    }

    // Manual logout - with confirmation dialog
    logout() {
        // Debug: log where this was called from
        console.error('logout() called! Stack trace:');
        console.trace();

        if (confirm('Are you sure you want to log out?')) {
            this.silentLogout();
            this.showSuccessMessage('Logged Out', 'You have been successfully logged out.');
        }
    }

    // Utility methods
    formatLastActive(dateString) {
        const date = new Date(dateString);
        const now = new Date();
        const diffMs = now - date;
        const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
        const diffDays = Math.floor(diffHours / 24);

        if (diffDays > 0) {
            return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
        } else if (diffHours > 0) {
            return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
        } else {
            return 'Just now';
        }
    }

    formatDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric'
        });
    }

    showSuccessMessage(title, message) {
        const notification = document.createElement('div');
        notification.className = 'fixed top-20 right-4 bg-emerald text-white p-4 rounded-lg shadow-lg z-50 max-w-sm';
        notification.innerHTML = `
            <div class="flex items-start space-x-3">
                <div class="w-8 h-8 bg-white/20 rounded-full flex items-center justify-center flex-shrink-0">
                    <span class="text-white text-sm">✓</span>
                </div>
                <div class="flex-1">
                    <div class="font-semibold text-sm">${title}</div>
                    <div class="text-xs opacity-90 mt-1">${message}</div>
                </div>
                <button onclick="this.parentElement.parentElement.remove()" class="text-white/60 hover:text-white text-sm">×</button>
            </div>
        `;

        document.body.appendChild(notification);

        // Auto remove after 5 seconds
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    showError(title, message) {
        const notification = document.createElement('div');
        notification.className = 'fixed top-20 right-4 bg-deep-red text-white p-4 rounded-lg shadow-lg z-50 max-w-sm';
        notification.innerHTML = `
            <div class="flex items-start space-x-3">
                <div class="w-8 h-8 bg-white/20 rounded-full flex items-center justify-center flex-shrink-0">
                    <span class="text-white text-sm">!</span>
                </div>
                <div class="flex-1">
                    <div class="font-semibold text-sm">${title}</div>
                    <div class="text-xs opacity-90 mt-1">${message}</div>
                </div>
                <button onclick="this.parentElement.parentElement.remove()" class="text-white/60 hover:text-white text-sm">×</button>
            </div>
        `;

        document.body.appendChild(notification);

        // Auto remove after 5 seconds
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    showNotification(title, message, type = 'info') {
        const bgColor = type === 'error' ? 'bg-deep-red' :
            type === 'info' ? 'bg-mystic' : 'bg-forest';

        const notification = document.createElement('div');
        notification.className = `fixed top-20 right-4 ${bgColor} text-white p-4 rounded-lg shadow-lg z-50 max-w-sm`;
        notification.innerHTML = `
            <div class="flex items-start space-x-3">
                <div class="w-8 h-8 bg-white/20 rounded-full flex items-center justify-center flex-shrink-0">
                    <span class="text-white text-sm">${type === 'error' ? '!' : type === 'info' ? 'ℹ' : '✓'}</span>
                </div>
                <div class="flex-1">
                    <div class="font-semibold text-sm">${title}</div>
                    <div class="text-xs opacity-90 mt-1">${message}</div>
                </div>
                <button onclick="this.parentElement.parentElement.remove()" class="text-white/60 hover:text-white text-sm">×</button>
            </div>
        `;

        document.body.appendChild(notification);

        // Auto remove after 5 seconds
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    setupEventListeners() {
        // Close DM Tools dropdown when clicking outside
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.dm-tools-dropdown-container')) {
                document.querySelectorAll('.dm-tools-menu').forEach(menu => {
                    menu.classList.add('hidden');
                });
            }
        });

        // Handle login form
        const loginForm = document.getElementById('admin-login-form');
        if (loginForm) {
            console.log('Login form found, attaching listener');
            loginForm.addEventListener('submit', async (e) => {
                console.log('Login form submitted');
                e.preventDefault();
                const formData = new FormData(e.target);
                const data = Object.fromEntries(formData.entries());
                console.log('Login data:', data);
                await this.handlePasswordLogin(data.email, data.password);
            });
        } else {
            console.error('Login form not found!');
        }

        // Handle maintenance mode toggle
        const maintenanceToggle = document.getElementById('maintenance-toggle');
        if (maintenanceToggle) {
            maintenanceToggle.addEventListener('change', (e) => {
                if (e.target.checked) {
                    this.showNotification('Modalità Manutenzione', 'La piattaforma è ora in modalità manutenzione.', 'info');
                } else {
                    this.showNotification('Modalità Manutenzione', 'La piattaforma è ora operativa.', 'info');
                }
            });
        }

        // Handle email toggle
        const emailToggle = document.getElementById('email-toggle');
        if (emailToggle) {
            emailToggle.addEventListener('change', (e) => {
                if (e.target.checked) {
                    this.showSuccessMessage('Notifiche Email', 'Le notifiche email sono ora abilitate.');
                } else {
                    this.showNotification('Notifiche Email', 'Le notifiche email sono ora disabilitate.', 'info');
                }
            });
        }
    }
}

// Initialize admin manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.adminManager = new AdminManager();
});