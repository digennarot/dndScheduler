// Dashboard-specific functionality
class DashboardManager {
    constructor(app) {
        this.app = app;
        this.init();
    }

    init() {
        this.renderActivePolls();
        this.renderActivityFeed();
        this.initializeCharts();
        this.updateStatistics();
        this.startRealTimeUpdates();

        document.addEventListener('pollsLoaded', () => {
            this.renderActivePolls();
            this.renderActivityFeed();
            this.updateStatistics();
        });
    }

    updateStatistics() {
        if (!this.app || !this.app.polls) return;

        const polls = this.app.polls;
        const hasPolls = polls.length > 0;

        // Calculate Success Rate
        if (hasPolls) {
            const successfulPolls = polls.filter(poll => {
                // Consider a poll successful if it's finalized or has >75% response rate
                if (poll.status === 'finalized') return true;
                const responseRate = this.app.calculateResponseRate(poll);
                return responseRate >= 75;
            });
            const successRate = Math.round((successfulPolls.length / polls.length) * 100);
            document.getElementById('success-rate').textContent = `${successRate}%`;
        } else {
            document.getElementById('success-rate').textContent = 'N/D';
        }

        // Calculate Average Response Time
        if (!hasPolls) {
            document.getElementById('avg-response-time').textContent = 'N/D';
        } else {
            let totalResponseTime = 0;
            let pollsWithResponses = 0;

            polls.forEach(poll => {
                const pollCreated = new Date(poll.created_at);
                let firstResponseTime = null;

                // Find the earliest response time
                Object.values(poll.responses || {}).forEach(response => {
                    if (response.responded && response.timestamp) {
                        const responseTime = new Date(response.timestamp);
                        if (!firstResponseTime || responseTime < firstResponseTime) {
                            firstResponseTime = responseTime;
                        }
                    }
                });

                if (firstResponseTime) {
                    const timeDiff = firstResponseTime - pollCreated;
                    const daysDiff = timeDiff / (1000 * 60 * 60 * 24);
                    totalResponseTime += daysDiff;
                    pollsWithResponses++;
                }
            });

            if (pollsWithResponses > 0) {
                const avgDays = (totalResponseTime / pollsWithResponses).toFixed(1);
                document.getElementById('avg-response-time').textContent = `${avgDays} giorni`;
            } else {
                document.getElementById('avg-response-time').textContent = 'In Attesa';
            }
        }

        // Update Hero Stats
        const activeCampaigns = polls.filter(p => p.status === 'active').length;
        const sessionsScheduled = polls.filter(p => p.status === 'finalized').length;

        // Calculate pending responses for the current user
        let pendingResponses = 0;
        if (this.app.currentUser) {
            pendingResponses = polls.filter(poll => {
                // Check if user is a participant but hasn't responded
                const isParticipant = poll.participants.includes(this.app.currentUser.id);
                const userResponse = poll.responses ? poll.responses[this.app.currentUser.id] : null;
                return isParticipant && (!userResponse || !userResponse.responded);
            }).length;
        }

        const activeEl = document.getElementById('hero-active-campaigns');
        const pendingEl = document.getElementById('hero-pending-responses');
        const scheduledEl = document.getElementById('hero-sessions-scheduled');

        if (activeEl) activeEl.textContent = `${activeCampaigns} Campagn${activeCampaigns !== 1 ? 'e' : 'a'} Attiv${activeCampaigns !== 1 ? 'e' : 'a'}`;
        if (pendingEl) pendingEl.textContent = `${pendingResponses} Rispost${pendingResponses !== 1 ? 'e' : 'a'} in Attesa`;
        if (scheduledEl) scheduledEl.textContent = `${sessionsScheduled} Session${sessionsScheduled !== 1 ? 'i' : 'e'} Pianificat${sessionsScheduled !== 1 ? 'e' : 'a'}`;
    }

    renderActivePolls() {
        const container = document.getElementById('active-polls');
        if (!container) return;

        // Check if user is logged in - hide section if not
        if (!this.app.currentUser) {
            // Hide the entire Active Campaigns section
            const section = container.closest('section');
            if (section) {
                section.style.display = 'none';
            }
            return;
        }

        // Show the section when logged in
        const section = container.closest('section');
        if (section) {
            section.style.display = 'block';
        }

        if (!this.app || !this.app.polls || this.app.polls.length === 0) {
            this.renderEmptyState(container);
            return;
        }

        // Filter polls where the current user is a participant
        const userPolls = this.app.polls.filter(poll =>
            poll.participants.includes(this.app.currentUser.id) ||
            poll.participants.includes(this.app.currentUser.email)
        );

        const activePolls = userPolls.filter(poll => poll.status === 'active');

        if (activePolls.length === 0) {
            this.renderEmptyState(container);
            return;
        }

        container.innerHTML = `<div class="grid relative grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            ${activePolls.map(poll => {
            const datesCount = poll.dates ? JSON.parse(poll.dates).length : 0;
            const totalParticipants = poll.participants.length;
            const respondedCount = poll.responses ? Object.keys(poll.responses).filter(k => poll.responses[k].responded).length : 0;
            const responseRate = totalParticipants > 0 ? Math.round((respondedCount / totalParticipants) * 100) : 0;
            const organizer = this.app.users.find(u => u.id === poll.organizer_id);

            return `
                <div class="bg-white rounded-xl shadow-lg p-6 hover:shadow-xl transition-all card-hover border-2 border-transparent hover:border-amber/30">
                    <div class="flex items-start justify-between mb-4">
                        <h4 class="font-cinzel text-xl font-bold text-forest">${this.app.escapeHtml(poll.title)}</h4>
                        <span class="px-3 py-1 bg-emerald/10 text-emerald rounded-full text-xs font-semibold">
                            Attiva
                        </span>
                    </div>

                    <p class="text-gray-600 mb-4 line-clamp-2">${this.app.escapeHtml(poll.description) || 'Nessuna descrizione'}</p>

                    <div class="space-y-3 mb-4">
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-gray-500">Organizzatore:</span>
                            <span class="font-medium">${organizer ? organizer.name : 'Sconosciuto'}</span>
                        </div>
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-gray-500">Date Proposte:</span>
                            <span class="font-medium">${datesCount} ${datesCount === 1 ? 'data' : 'date'}</span>
                        </div>
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-gray-500">Risposte:</span>
                            <span class="font-medium">${respondedCount}/${totalParticipants}</span>
                        </div>
                    </div>

                    <div class="mb-4">
                        <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
                            <span>Progresso</span>
                            <span>${responseRate}%</span>
                        </div>
                        <div class="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
                            <div class="h-full bg-gradient-to-r from-emerald to-forest rounded-full transition-all"
                                style="width: ${responseRate}%"></div>
                        </div>
                    </div>

                    <div class="flex gap-2">
                        <a href="participate.html?poll=${poll.id}"
                            class="flex-1 text-center bg-forest text-white px-4 py-2 rounded-lg font-semibold hover:bg-forest/90 transition-all text-sm">
                            Partecipa
                        </a>
                        <a href="manage.html?poll=${poll.id}"
                            class="flex-1 text-center border-2 border-forest text-forest px-4 py-2 rounded-lg font-semibold hover:bg-forest hover:text-white transition-all text-sm">
                            Gestisci
                        </a>
                    </div>
                </div>
                `;
        }).join('')}
        </div>`;
    }

    renderEmptyState(container) {
        container.innerHTML = `
                <div class="col-span-full text-center py-12">
                <div class="text-gray-400 text-6xl mb-4">🎲</div>
                <h3 class="text-xl font-semibold text-gray-600 mb-2">Nessuna Campagna Attiva</h3>
                <p class="text-gray-500 mb-6">Inizia creando la tua prima campagna o unisciti a una esistente!</p>
                <div class="flex justify-center space-x-4">
                    <a href="create-poll.html" class="bg-forest text-white px-6 py-3 rounded-lg font-semibold hover:bg-forest/90 transition-all">
                        Crea Campagna
                    </a>
                    <a href="participate.html" class="bg-amber text-white px-6 py-3 rounded-lg font-semibold hover:bg-amber/90 transition-all">
                        Unisciti
                    </a>
                </div>
            </div>
                `;
    }

    showNotification(title, message) {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = 'fixed top-20 right-4 bg-forest text-white p-4 rounded-lg shadow-lg z-50 max-w-sm mystical-glow';
        notification.innerHTML = `
                <div class="flex items-start space-x-3">
                <div class="w-8 h-8 bg-amber rounded-full flex items-center justify-center flex-shrink-0">
                    <span class="text-white text-sm">📅</span>
                </div>
                <div class="flex-1">
                    <div class="font-semibold text-sm">${title}</div>
                    <div class="text-xs opacity-90 mt-1">${message}</div>
                </div>
                <button onclick="this.parentElement.parentElement.remove()" class="text-white/60 hover:text-white text-sm">×</button>
            </div>
                `;

        document.body.appendChild(notification);

        // Animate in
        anime({
            targets: notification,
            opacity: [0, 1],
            translateX: [100, 0],
            duration: 300,
            easing: 'easeOutQuart'
        });

        // Auto remove after 5 seconds
        setTimeout(() => {
            anime({
                targets: notification,
                opacity: [1, 0],
                translateX: [0, 100],
                duration: 300,
                easing: 'easeInQuart',
                complete: () => notification.remove()
            });
        }, 5000);
    }
}

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    if (window.DDSchedulerApp) {
        new DashboardManager(window.DDSchedulerApp);
    }
});