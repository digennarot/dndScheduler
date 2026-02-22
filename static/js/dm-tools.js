/**
 * DM Tools Manager
 * Handles logic for dedicated DM tool pages
 */

class DMToolsManager {
    constructor() {
        this.currentUser = null;
        this.isAuthenticated = false;
        this.init();
    }

    async init() {
        // Authenticate check
        const token = localStorage.getItem('adminToken');
        if (!token) {
            window.location.href = '/admin.html'; // Redirect to admin login if not authenticated
            return;
        }

        // Mock getting user info from token or stored user
        // In a real app we'd validate properly
        this.isAuthenticated = true;
        this.currentUser = { name: 'Admin User' }; // Placeholder

        // Page specific initialization
        this.dispatchPageLoad();
    }

    dispatchPageLoad() {
        const path = window.location.pathname;
        if (path.includes('reminders.html')) {
            this.loadRemindersPage();
        } else if (path.includes('sessions.html')) {
            this.loadSessionsPage();
        } else if (path.includes('calendar.html')) {
            this.loadCalendarPage();
        } else {
            console.log('DM Tools Dashboard');
        }
    }

    // ==========================================
    // SHARED UTILITIES
    // ==========================================

    async fetchAllSessions() {
        try {
            const response = await fetch('/api/polls');
            if (response.ok) {
                return await response.json();
            }
        } catch (error) {
            console.error('Error fetching sessions:', error);
        }
        return [];
    }

    async fetchSessionDetails(sessionId) {
        try {
            const response = await fetch(`/api/polls/${sessionId}`);
            if (response.ok) {
                return await response.json();
            }
        } catch (error) {
            console.error(`Error fetching details for session ${sessionId}:`, error);
        }
        return null;
    }

    formatDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('it-IT', {
            weekday: 'short',
            day: 'numeric',
            month: 'short',
            year: 'numeric'
        });
    }

    showNotification(title, message, type = 'info') {
        const bgColor = type === 'error' ? 'bg-deep-red' :
            type === 'info' ? 'bg-mystic' : 'bg-emerald'; // Changed success to emerald which is existing class

        const notification = document.createElement('div');
        notification.className = `fixed top-20 right-4 ${bgColor} text-white p-4 rounded-lg shadow-lg z-50 max-w-sm transition-all duration-300 transform translate-x-0`;
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
        setTimeout(() => notification.remove(), 5000);
    }

    // ==========================================
    // PAGE HANDLERS
    // ==========================================

    async loadRemindersPage() {
        console.log('Loading Reminders Page');
        const container = document.getElementById('sessions-list');
        if (!container) return;

        const sessions = await this.fetchAllSessions();
        if (sessions.length === 0) {
            container.innerHTML = '<p class="text-gray-400 text-center col-span-full">Nessuna sessione attiva.</p>';
            return;
        }

        container.innerHTML = sessions.map(session => `
            <div class="admin-card p-6 rounded-xl hover:bg-gray-700/50 transition-colors cursor-pointer" onclick="dmTools.selectSessionForReminders('${session.id}')">
                <div class="flex justify-between items-start mb-4">
                    <h3 class="font-cinzel text-xl text-amber font-bold">${session.title}</h3>
                    <span class="text-xs text-gray-500 bg-gray-800 px-2 py-1 rounded">ID: ${session.id}</span>
                </div>
                <p class="text-sm text-gray-400 mb-4 line-clamp-2">${session.description || 'Nessuna descrizione'}</p>
                <div class="flex justify-between items-center text-sm text-gray-400">
                    <span>📅 ${this.formatDate(new Date(session.created_at * 1000))}</span>
                    <button class="action-btn primary text-xs">Seleziona</button>
                </div>
            </div>
        `).join('');
    }

    async selectSessionForReminders(sessionId) {
        // Implementation for selecting session and showing details
        console.log('Selected session:', sessionId);
        const detailsContainer = document.getElementById('reminder-details');
        const placeholder = document.getElementById('reminder-placeholder');

        if (placeholder) placeholder.classList.add('hidden');
        if (detailsContainer) detailsContainer.classList.remove('hidden');

        // Fetch details to get participants info
        const data = await this.fetchSessionDetails(sessionId);
        if (!data) return;

        const participants = data.participants || [];
        const availability = data.availability || [];

        // Calculate pending
        const pendingParticipants = participants.filter(p => !availability.some(a => a.participant_id === p.id));

        document.getElementById('selected-session-title').textContent = data.poll.title;
        document.getElementById('pending-count').textContent = pendingParticipants.length;
        document.getElementById('total-count').textContent = participants.length;

        const pendingList = document.getElementById('pending-participants-list');
        if (pendingList) {
            if (pendingParticipants.length === 0) {
                pendingList.innerHTML = '<p class="text-green-400">Tutti hanno risposto! 🎉</p>';
            } else {
                pendingList.innerHTML = pendingParticipants.map(p => `
                    <div class="flex items-center space-x-2 text-sm text-gray-300">
                        <span class="w-2 h-2 rounded-full bg-amber"></span>
                        <span>${p.name}</span>
                    </div>
                `).join('');
            }
        }

        // Update send button
        const sendBtn = document.getElementById('trigger-reminders-btn');
        if (sendBtn) {
            sendBtn.onclick = () => {
                if (window.reminderManager) {
                    window.reminderManager.sendReminders(sessionId, pendingParticipants.map(p => p.id));
                    this.showNotification('Invio...', `Invio promemoria a ${pendingParticipants.length} utenti`);
                } else {
                    alert('Reminder Manager not loaded');
                }
            };
            // Disable if no pending
            sendBtn.disabled = pendingParticipants.length === 0;
            if (pendingParticipants.length === 0) {
                sendBtn.classList.add('opacity-50', 'cursor-not-allowed');
            } else {
                sendBtn.classList.remove('opacity-50', 'cursor-not-allowed');
            }
        }
    }

    async loadSessionsPage() {
        const tbody = document.getElementById('sessions-table-body');
        if (!tbody) return;

        const sessions = await this.fetchAllSessions();
        if (sessions.length === 0) {
            tbody.innerHTML = '<tr><td colspan="5" class="text-center py-8 text-gray-500">Nessuna sessione trovata.</td></tr>';
            return;
        }

        tbody.innerHTML = sessions.map(session => `
            <tr class="hover:bg-gray-700/30 transition-colors">
                <td class="px-6 py-4">
                    <div class="font-medium text-amber">${session.title}</div>
                    <div class="text-xs text-gray-500">${session.id}</div>
                </td>
                <td class="px-6 py-4 text-gray-300">
                    ${this.formatDate(new Date(session.created_at * 1000))}
                </td>
                <td class="px-6 py-4">
                    <span class="px-2 py-1 bg-emerald/20 text-emerald text-xs rounded-full">Attiva</span>
                </td>
                <td class="px-6 py-4">
                     <div class="flex space-x-2">
                        <button onclick="dmTools.handleSessionAction('view', '${session.id}')" class="text-gray-400 hover:text-white" title="Visualizza">👁️</button>
                        <button onclick="dmTools.handleSessionAction('edit', '${session.id}')" class="text-gray-400 hover:text-white" title="Modifica">✏️</button>
                        <button onclick="dmTools.handleSessionAction('duplicate', '${session.id}')" class="text-gray-400 hover:text-white" title="Duplica">📋</button>
                        <button onclick="dmTools.handleSessionAction('delete', '${session.id}')" class="text-red-400 hover:text-red-300" title="Elimina">🗑️</button>
                     </div>
                </td>
            </tr>
        `).join('');
    }

    async handleSessionAction(action, sessionId) {
        switch (action) {
            case 'duplicate':
                if (window.adminManager) window.adminManager.duplicateSessioneForSession(sessionId); // Reuse if available, else reimplement
                else {
                    // Quick implementation of duplicate logic reusing create-poll page
                    const response = await fetch(`/api/polls/${sessionId}`);
                    if (response.ok) {
                        const data = await response.json();
                        const params = new URLSearchParams({
                            duplicate: sessionId,
                            title: `${data.poll.title} (Copia)`,
                            description: data.poll.description || '',
                            location: data.poll.location || ''
                        });
                        window.location.href = `/create-poll.html?${params.toString()}`;
                    }
                }
                break;
            case 'delete':
                if (confirm('Sei sicuro di voler eliminare questa sessione?')) {
                    await fetch(`/api/polls/${sessionId}`, { method: 'DELETE' });
                    this.showNotification('Eliminata', 'Sessione eliminata con successo');
                    this.loadSessionsPage(); // Refresh
                }
                break;
            case 'view':
                // Redirect to details view? or Modal?
                // For now let's just use the Admin Manager modal re-used or redirect to a comprehensive view page
                alert('Funzionalità Visualizza in arrivo - usa la Dashboard Admin per ora');
                break;
            case 'edit':
                // Could reuse the edit modal from admin manager if we refactored it to be more modular.
                // For now, simple alert or redirect
                alert('Funzionalità Modifica in arrivo');
                break;
        }
    }

    async loadCalendarPage() {
        const calendarGrid = document.getElementById('calendar-grid');
        if (!calendarGrid) return;

        // Mock calendar generation - simple current month view
        // In production use a library like FullCalendar

        const now = new Date();
        const year = now.getFullYear();
        const month = now.getMonth();

        const sessions = await this.fetchAllSessions();
        // Parse dates from all sessions and map to days
        const sessionMap = new Map();

        for (const session of sessions) {
            const details = await this.fetchSessionDetails(session.id); // Need details for dates
            if (details && details.poll.dates) {
                try {
                    const dates = JSON.parse(details.poll.dates);
                    dates.forEach(d => {
                        if (!sessionMap.has(d)) sessionMap.set(d, []);
                        sessionMap.get(d).push(session);
                    });
                } catch (e) { }
            }
        }

        // Render Grid
        // (Simplified for this file size limit, assuming basic structure exists in HTML)

        // ... (Calendar rendering logic would go here)
        console.log('Calendar data prepared', sessionMap);
    }
}

window.dmTools = new DMToolsManager();
