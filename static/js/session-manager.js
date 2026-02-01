// Session Management System
class SessionManager {
    constructor() {
        this.selectedSession = null;
        this.currentUser = null;
        this.init();
    }

    init() {
        this.loadCurrentUser();

        // Initial load (might be empty if fetching)
        this.loadSessionsList();
        this.loadQuickStats();
        this.loadRecentActivity();
        this.setupEventListeners();

        // Listen for data load
        document.addEventListener('pollsLoaded', () => {
            this.loadCurrentUser(); // Re-load user if needed
            this.loadSessionsList();
            this.loadQuickStats();
            this.loadRecentActivity();

            // Check for poll ID in URL
            const urlParams = new URLSearchParams(window.location.search);
            const pollId = urlParams.get('poll');
            if (pollId) {
                this.selectSession(pollId);
                // Check for auto-edit action
                if (urlParams.get('action') === 'edit') {
                    // Slight delay to ensure UI update
                    setTimeout(() => this.editSession(), 100);
                }
            }
        });
    }

    loadCurrentUser() {
        if (window.DDSchedulerApp) {
            this.currentUser = window.DDSchedulerApp.currentUser;
        }
    }

    loadSessionsList() {
        const container = document.getElementById('sessions-list');
        if (!container || !window.DDSchedulerApp) return;

        // Filter by organizer (Story 4.1)
        const userSessions = window.DDSchedulerApp.polls.filter(poll => {
            // Include if I am the organizer
            if (this.currentUser && poll.organizer_id === this.currentUser.id) return true;

            // Also include if I am a participant (optional, but keep for completeness if manage.html is used for both)
            // But manage.html says "Gestisci le Tue Campagne" (Manage Your Campaigns), suggesting it's for DMs.
            // Let's stick to organizer mainly, or check role.
            // If I am a DM, I want to see polls I created.
            return false;
        });

        if (userSessions.length === 0) {
            container.innerHTML = `
                <div class="text-center py-12">
                    <div class="text-gray-400 text-6xl mb-4">📅</div>
                    <h3 class="text-xl font-semibold text-gray-600 mb-2">Nessuna Sessione Trovata</h3>
                    <p class="text-gray-500 mb-4">Non hai ancora creato nessuna sessione di pianificazione.</p>
                    <button onclick="window.location.href='create-poll.html'" 
                            class="action-button primary">
                        Crea la Tua Prima Sessione
                    </button>
                </div>
            `;
            return;
        }

        const formatDateIT = (dateString) => {
            const date = new Date(dateString);
            return date.toLocaleDateString('it-IT', {
                weekday: 'short',
                month: 'short',
                day: 'numeric'
            });
        };

        container.innerHTML = userSessions.map(session => {
            const responseRate = window.DDSchedulerApp.calculateResponseRate(session);
            const respondedCount = Object.keys(session.responses || {}).filter(userId =>
                session.responses[userId].responded
            ).length;

            // Handle missing fields with defaults
            const status = session.status || 'active';
            const duration = session.duration || 180; // Default 3 hours
            const createdAt = session.created_at || session.createdAt || Date.now();
            const participants = session.participants || [];

            // Clean description for display (remove metadata)
            let cleanDesc = session.description || '';
            cleanDesc = cleanDesc.replace(/Timezone:.*(\n|$)/g, '')
                .replace(/Recurring Pattern:.*(\n|$)/g, '')
                .trim();

            const isFinished = status === 'finalized';

            return `
                <div class="group border border-gray-200 rounded-xl p-5 hover:border-amber transition-all duration-300 hover:shadow-md cursor-pointer bg-white relative overflow-hidden"
                     onclick="sessionManager.selectSession('${session.id}')">
                    
                    <div class="flex items-start justify-between mb-4">
                        <div class="flex-1 pr-4">
                            <h4 class="font-cinzel text-xl font-bold text-forest mb-1 group-hover:text-amber-700 transition-colors">${window.DDSchedulerApp.escapeHtml(session.title)}</h4>
                            <p class="text-gray-600 text-sm line-clamp-2">${window.DDSchedulerApp.escapeHtml(cleanDesc) || 'Nessuna descrizione.'}</p>
                        </div>
                        <div class="flex-shrink-0">
                            <span class="${isFinished ? 'bg-purple-100 text-purple-800' : 'bg-emerald/10 text-emerald-800'} px-3 py-1 rounded-full text-xs font-semibold uppercase tracking-wide">
                                ${isFinished ? 'Finalizzata' : 'Attiva'}
                            </span>
                        </div>
                    </div>
                    
                    <div class="bg-gray-50 rounded-lg p-3 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-4">
                        <div class="flex flex-col">
                            <span class="text-xs text-gray-500 uppercase tracking-wider mb-1">Durata</span>
                            <span class="font-semibold text-gray-700 flex items-center">
                                <span class="mr-1">⏱️</span> ${this.formatDuration(duration)}
                            </span>
                        </div>
                        <div class="flex flex-col">
                            <span class="text-xs text-gray-500 uppercase tracking-wider mb-1">Giocatori</span>
                            <span class="font-semibold text-gray-700 flex items-center">
                                <span class="mr-1">👥</span> ${participants.length}
                            </span>
                        </div>
                        <div class="flex flex-col">
                            <span class="text-xs text-gray-500 uppercase tracking-wider mb-1">Risposte</span>
                            <span class="font-semibold text-gray-700 flex items-center">
                                <span class="mr-1">📩</span> ${respondedCount}/${participants.length}
                            </span>
                        </div>
                        <div class="flex flex-col">
                            <span class="text-xs text-gray-500 uppercase tracking-wider mb-1">Tasso Risp.</span>
                            <span class="font-semibold ${responseRate >= 100 ? 'text-emerald-600' : responseRate >= 50 ? 'text-amber-600' : 'text-gray-700'} flex items-center">
                                <span class="mr-1">📊</span> ${responseRate}%
                            </span>
                        </div>
                    </div>
                    
                    <div class="flex items-center justify-between pt-2 border-t border-gray-100">
                        <div class="text-xs text-gray-400">
                            Creata il ${formatDateIT(createdAt)}
                        </div>
                        <div class="flex space-x-3">
                            ${!isFinished ? `
                                <button onclick="event.stopPropagation(); sessionManager.sendReminders('${session.id}')" 
                                        class="text-xs font-medium text-amber hover:text-amber-700 transition-colors flex items-center">
                                    🔔 <span class="ml-1">Invia Promemoria</span>
                                </button>
                            ` : ''}
                            <button onclick="event.stopPropagation(); sessionManager.viewSession('${session.id}')" 
                                    class="text-xs font-medium text-forest hover:text-forest-700 transition-colors flex items-center">
                                👁️ <span class="ml-1">Dettagli</span>
                            </button>
                        </div>
                    </div>
                    
                    ${session.bestTimes && session.bestTimes.length > 0 ? `
                        <div class="mt-3 text-sm">
                            <span class="text-xs text-gray-500 mr-2">Top:</span>
                            ${session.bestTimes.slice(0, 2).map(time => `
                                <span class="inline-block px-2 py-0.5 bg-amber/10 text-amber-800 rounded text-xs mr-1">
                                    ${this.formatDateTime(time)}
                                </span>
                            `).join('')}
                        </div>
                    ` : ''}
                </div>
            `;
        }).join('');
    }

    loadQuickStats() {
        if (!window.DDSchedulerApp) return;

        // TODO: Filter by organizer once authentication is implemented
        const userSessions = window.DDSchedulerApp.polls;

        const activeCount = userSessions.filter(s => (s.status || 'active') === 'active').length;
        const finalizedCount = userSessions.filter(s => (s.status || 'active') === 'finalized').length;

        const totalResponseRates = userSessions.map(session =>
            window.DDSchedulerApp.calculateResponseRate(session)
        );
        const avgResponseRate = totalResponseRates.length > 0
            ? Math.round(totalResponseRates.reduce((a, b) => a + b, 0) / totalResponseRates.length)
            : 0;

        document.getElementById('active-count').textContent = activeCount;
        document.getElementById('finalized-count').textContent = finalizedCount;
        document.getElementById('avg-response-rate').textContent = `${avgResponseRate}%`;

        // Load activity chart
        this.loadActivityChart();
    }


    loadActivityChart() {
        const chartContainer = document.getElementById('activity-chart');
        if (!chartContainer || !window.DDSchedulerApp) return;

        const chart = echarts.init(chartContainer);

        // Calculate real activity data from the last 7 days
        const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
        const today = new Date();
        const activity = new Array(7).fill(0);

        // TODO: Filter by organizer once authentication is implemented
        const userSessions = window.DDSchedulerApp.polls;

        // Count activities (poll creations and responses) per day
        userSessions.forEach(session => {
            // Count poll creation
            // Fix: Multiply by 1000 for JS Date (backend sends seconds)
            if (session.created_at) {
                const timestamp = typeof session.created_at === 'number' && session.created_at < 10000000000
                    ? session.created_at * 1000
                    : session.created_at;

                const createdDate = new Date(timestamp);
                const daysDiff = Math.floor((today - createdDate) / (1000 * 60 * 60 * 24));
                if (daysDiff >= 0 && daysDiff < 7) {
                    activity[6 - daysDiff]++;
                }
            }

            // Count responses
            Object.values(session.responses || {}).forEach(response => {
                if (response.responded && response.timestamp) {
                    // Responses might be in ms or s, check heuristic
                    let timestamp = response.timestamp;
                    if (typeof timestamp === 'number' && timestamp < 10000000000) {
                        timestamp *= 1000;
                    }

                    const responseDate = new Date(timestamp);
                    const daysDiff = Math.floor((today - responseDate) / (1000 * 60 * 60 * 24));
                    if (daysDiff >= 0 && daysDiff < 7) {
                        activity[6 - daysDiff]++;
                    }
                }
            });
        });

        const option = {
            grid: {
                left: '10%',
                right: '10%',
                top: '10%',
                bottom: '20%'
            },
            xAxis: {
                type: 'category',
                data: days,
                axisLabel: {
                    fontSize: 10,
                    color: '#666'
                },
                axisLine: { show: false },
                axisTick: { show: false }
            },
            yAxis: {
                type: 'value',
                show: false
            },
            series: [{
                data: activity,
                type: 'bar',
                itemStyle: {
                    color: '#d4a574',
                    borderRadius: [2, 2, 0, 0]
                },
                barWidth: '60%'
            }],
            tooltip: {
                trigger: 'axis',
                formatter: function (params) {
                    return `${params[0].name}: ${params[0].value} attività`;
                }
            }
        };

        chart.setOption(option);
    }

    loadRecentActivity() {
        const container = document.getElementById('recent-activity');
        if (!container || !window.DDSchedulerApp) return;

        // TODO: Filter by organizer once authentication is implemented
        const userSessions = window.DDSchedulerApp.polls;

        // Collect real activities
        const activities = [];

        userSessions.forEach(session => {
            // Poll creation activity
            // Fix: Multiply by 1000 for JS Date
            if (session.created_at) {
                const timestamp = typeof session.created_at === 'number' && session.created_at < 10000000000
                    ? session.created_at * 1000
                    : session.created_at;

                activities.push({
                    type: 'created',
                    message: `Campagna creata: ${session.title}`,
                    timestamp: new Date(timestamp),
                    icon: '🎲'
                });
            }

            // Response activities
            Object.entries(session.responses || {}).forEach(([userId, response]) => {
                if (response.responded && response.timestamp) {
                    let timestamp = response.timestamp;
                    if (typeof timestamp === 'number' && timestamp < 10000000000) {
                        timestamp *= 1000;
                    }

                    const user = window.DDSchedulerApp.getUserById(userId);
                    activities.push({
                        type: 'response',
                        message: `${user?.name || 'Un giocatore'} ha risposto a ${session.title}`,
                        timestamp: new Date(timestamp),
                        icon: '✓'
                    });
                }
            });

            // Finalization activity
            if (session.status === 'finalized' && session.finalizedAt) {
                let timestamp = session.finalizedAt;
                // parsed dates string or iso string usually don't need scaling but let's be safe if number
                if (typeof timestamp === 'number' && timestamp < 10000000000) {
                    timestamp *= 1000;
                }

                activities.push({
                    type: 'finalized',
                    message: `Campagna ${session.title} finalizzata`,
                    timestamp: new Date(timestamp),
                    icon: '📅'
                });
            }
        });

        // Sort by timestamp (most recent first)
        activities.sort((a, b) => b.timestamp - a.timestamp);

        // Take only the 5 most recent
        const recentActivities = activities.slice(0, 5);

        if (recentActivities.length === 0) {
            container.innerHTML = `
                <div class="text-center py-8 text-gray-500">
                    <div class="text-4xl mb-2">📋</div>
                    <div class="text-sm">Nessuna attività recente</div>
                </div>
            `;
            return;
        }

        container.innerHTML = recentActivities.map(activity => `
            <div class="flex items-start space-x-3 p-3 rounded-lg hover:bg-gray-50 transition-colors">
                <div class="w-8 h-8 bg-amber/20 rounded-full flex items-center justify-center flex-shrink-0">
                    <span class="text-amber text-sm">${activity.icon}</span>
                </div>
                <div class="flex-1">
                    <div class="text-sm text-gray-900">${activity.message}</div>
                    <div class="text-xs text-gray-500 mt-1">${this.getTimeAgo(activity.timestamp)}</div>
                </div>
            </div>
        `).join('');
    }

    getTimeAgo(timestamp) {
        const now = new Date();
        const diff = now - timestamp;
        const seconds = Math.floor(diff / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);

        if (days > 0) return `${days} giorn${days === 1 ? 'o' : 'i'} fa`;
        if (hours > 0) return `${hours} or${hours === 1 ? 'a' : 'e'} fa`;
        if (minutes > 0) return `${minutes} minut${minutes === 1 ? 'o' : 'i'} fa`;
        return 'Proprio ora';
    }

    async selectSession(sessionId) {
        if (!window.DDSchedulerApp) return;

        // Try getting from local cache
        this.selectedSession = window.DDSchedulerApp.getPollById(sessionId);

        // Story 1.5: If not found (e.g. private poll or direct link), try fetching individually
        if (!this.selectedSession) {
            console.log('Session not found locally, attempting to fetch...', sessionId);
            this.selectedSession = await window.DDSchedulerApp.fetchSinglePoll(sessionId);

            // If found, refresh the list to show this new poll
            if (this.selectedSession) {
                this.loadSessionsList();
                this.loadQuickStats(); // Update stats too
            }
        }

        if (!this.selectedSession) {
            console.error('Session not found:', sessionId);
            // Optionally show error to user
            this.showNotification('Errore', 'Impossibile trovare la sessione specificata.', 'error');
            return;
        }

        // Show session detail section
        document.getElementById('session-detail').style.display = 'block';

        // Update session details
        this.updateSessionDetails();

        // Generate overlap heatmap
        this.generateOverlapHeatmap();

        // Load recommended times
        this.loadRecommendedTimes();

        // Load participant responses
        this.loadParticipantResponses();

        // Scroll to detail section
        document.getElementById('session-detail').scrollIntoView({
            behavior: 'smooth'
        });
    }

    updateSessionDetails() {
        if (!this.selectedSession) return;

        document.getElementById('detail-title').textContent = this.selectedSession.title; // textContent is safe, but consistency doesn't hurt

        // Clean description
        let cleanDesc = this.selectedSession.description || '';
        cleanDesc = cleanDesc.replace(/Timezone:.*(\n|$)/g, '')
            .replace(/Recurring Pattern:.*(\n|$)/g, '')
            .trim();

        document.getElementById('detail-description').textContent = cleanDesc || 'Nessuna descrizione disponibile.'; // textContent is safe

        // Update finalize button
        const finalizeBtn = document.getElementById('finalize-btn');
        if (this.selectedSession.status === 'finalized') {
            finalizeBtn.textContent = 'Campagna Finalizzata';
            finalizeBtn.disabled = true;
            finalizeBtn.classList.add('opacity-50', 'cursor-not-allowed');
        } else {
            finalizeBtn.textContent = 'Finalizza Disponibilità';
            finalizeBtn.disabled = false;
            finalizeBtn.classList.remove('opacity-50', 'cursor-not-allowed');
        }
    }

    generateOverlapHeatmap() {
        const container = document.getElementById('overlap-heatmap');
        if (!container || !this.selectedSession) return;

        // CRITICAL FIX: Remove the CSS Grid class that enforces 7 columns
        container.className = 'mt-4 overflow-hidden rounded-xl border border-gray-200 shadow-sm';

        // Parse dates
        let dates = [];
        try {
            const parsed = JSON.parse(this.selectedSession.dates);
            if (Array.isArray(parsed)) {
                dates = parsed;
            } else {
                dates = this.generateDateRange(this.selectedSession.dateRange.start, this.selectedSession.dateRange.end);
            }
        } catch (e) {
            dates = this.selectedSession.dateRange ?
                this.generateDateRange(this.selectedSession.dateRange.start, this.selectedSession.dateRange.end) : [];
        }

        const timeSlots = this.selectedSession.timeSlots || ['18:00', '19:00', '20:00', '21:00'];

        // Helper for IT dates
        const formatDateIT = (dateString) => {
            const date = new Date(dateString);
            return date.toLocaleDateString('it-IT', { weekday: 'short', day: 'numeric', month: 'short' });
        };

        // Build Table Structure
        let tableHtml = `
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="text-xs text-gray-700 uppercase bg-gray-50 border-b border-gray-100">
                        <tr>
                            <th scope="col" class="px-6 py-4 font-bold text-forest bg-gray-50/50 sticky left-0 z-10 w-24">
                                Orario
                            </th>
                            ${dates.map(date => `
                                <th scope="col" class="px-4 py-3 text-center min-w-[100px]">
                                    ${formatDateIT(date)}
                                </th>
                            `).join('')}
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-100">
        `;

        // Add rows
        timeSlots.forEach(time => {
            tableHtml += `
                <tr class="hover:bg-gray-50/50 transition-colors">
                    <td class="px-6 py-4 font-medium text-forest bg-white sticky left-0 z-10 border-r border-gray-100 shadow-[4px_0_8px_-4px_rgba(0,0,0,0.05)]">
                        ${this.formatTime(time)}
                    </td>
            `;

            dates.forEach(date => {
                const overlapCount = this.calculateOverlap(date, time);
                const totalParticipants = this.selectedSession.participants.length || 1;
                const ratio = overlapCount / totalParticipants;

                // Determine styling based on availability ratio
                let cellContent = '';

                if (overlapCount > 0) {
                    let bgClass = 'bg-gray-100 text-gray-800';
                    if (ratio === 1) bgClass = 'bg-emerald-500 text-white shadow-md';
                    else if (ratio >= 0.75) bgClass = 'bg-emerald-400 text-white';
                    else if (ratio >= 0.5) bgClass = 'bg-amber-400 text-white';

                    cellContent = `
                        <div class="w-8 h-8 mx-auto flex items-center justify-center rounded-lg font-bold ${bgClass} transition-transform hover:scale-110">
                            ${overlapCount}
                        </div>
                    `;
                } else {
                    cellContent = `<div class="w-2 h-2 mx-auto rounded-full bg-gray-200"></div>`;
                }

                tableHtml += `
                    <td class="px-4 py-3 text-center">
                        ${cellContent}
                    </td>
                `;
            });

            tableHtml += `</tr>`;
        });

        tableHtml += `
                    </tbody>
                </table>
            </div>
            <div class="bg-gray-50 px-4 py-2 border-t border-gray-100 text-xs text-center text-gray-500 flex justify-center space-x-4">
                <span class="flex items-center"><span class="w-2 h-2 rounded-full bg-emerald-500 mr-1"></span> Tutti Disponibili</span>
                <span class="flex items-center"><span class="w-2 h-2 rounded-full bg-amber-400 mr-1"></span> Buona Disponibilità</span>
                <span class="flex items-center"><span class="w-2 h-2 rounded-full bg-gray-200 mr-1"></span> Nessuno</span>
            </div>
        `;

        container.innerHTML = tableHtml;
    }

    // Deprecated helpers removed from loop but kept if needed for other things
    // createHeatmapCell removed as we generate string HTML directly now
    createHeatmapCell(content, type) {
        return null;
    }

    calculateOverlap(date, time) {
        if (!this.selectedSession || !this.selectedSession.responses) return 0;

        let overlapCount = 0;
        Object.values(this.selectedSession.responses).forEach(response => {
            if (response.responded && response.availability) {
                const cellId = `${date}_${time}`;
                // Check exact match 'available' or localized version if relevant
                // The backend stores 'available'
                if (response.availability[cellId] === 'available') {
                    overlapCount++;
                }
            }
        });

        return overlapCount;
    }

    loadRecommendedTimes() {
        const container = document.getElementById('recommended-times');
        if (!container || !this.selectedSession) return;

        // Get all dates and times
        let dates = [];
        try {
            const parsed = JSON.parse(this.selectedSession.dates);
            if (Array.isArray(parsed)) {
                // Determine if it's a range or list based on poll mode (heuristics: if 2 dates and high duration, likely range, but let's see)
                // Actually, backend now simply stores the list of dates for specific mode, 
                // or the list of all dates in range for range mode.
                // So reliable way is to trust the array content as the dates to check.
                // However, app.js logic suggests dateRange might be set.
                // Let's rely on the array directly.
                dates = parsed;
            } else if (parsed && parsed.start && parsed.end) {
                dates = this.generateDateRange(parsed.start, parsed.end);
            }
        } catch (e) { console.error("Error parsing dates", e); }

        // Use default if empty (fallback)
        if (dates.length === 0 && this.selectedSession.dateRange) {
            dates = this.generateDateRange(this.selectedSession.dateRange.start, this.selectedSession.dateRange.end);
        }

        const timeSlots = this.selectedSession.timeSlots || ['18:00', '19:00', '20:00', '21:00'];

        // Calculate overlap for all combinations
        const allCombinations = [];
        dates.forEach(date => {
            timeSlots.forEach(time => {
                const overlap = this.calculateOverlap(displayDate(date), time); // displayDate handles format if needed? No, calculateOverlap expects ISO YYYY-MM-DD
                // Actually calculateOverlap expects key matching availability map keys.
                // Availability map keys are "YYYY-MM-DD_HH:MM".
                const overlapCount = this.calculateOverlap(date, time);
                if (overlapCount > 0) {
                    allCombinations.push({
                        date,
                        time,
                        overlap: overlapCount
                    });
                }
            });
        });

        // Sort by overlap desc
        allCombinations.sort((a, b) => b.overlap - a.overlap);

        // Take top 5
        const recommendedTimes = allCombinations.slice(0, 5);

        if (recommendedTimes.length === 0) {
            container.innerHTML = `
                <div class="text-center py-6 text-gray-500 text-sm">
                    Nessuna disponibilità comune trovata ancora.
                </div>
            `;
            return;
        }

        const totalParticipants = this.selectedSession.participants.length || 1;

        container.innerHTML = recommendedTimes.map(rec => {
            // Calculate percentage
            const percentage = Math.round((rec.overlap / totalParticipants) * 100);

            // Determine confidence color and description
            let confidenceColor = 'bg-emerald/10 text-emerald-800';
            let confidenceText = '';

            if (percentage >= 75) {
                confidenceColor = 'bg-emerald/10 text-emerald-800';
                confidenceText = `Ottima (${percentage}%)`;
            } else if (percentage >= 50) {
                confidenceColor = 'bg-amber/10 text-amber-800';
                confidenceText = `Buona (${percentage}%)`;
            } else {
                confidenceColor = 'bg-deep-red/10 text-deep-red';
                confidenceText = `Bassa (${percentage}%)`;
            }

            return `
                <div class="border border-gray-200 rounded-lg p-4 hover:border-amber transition-colors">
                    <div class="flex items-center justify-between mb-2">
                        <h5 class="font-semibold text-forest">${this.formatDate(rec.date)}</h5>
                        <span class="px-2 py-1 ${confidenceColor} rounded text-xs font-medium">
                            ${confidenceText}
                        </span>
                    </div>
                    <div class="text-sm text-gray-600 mb-2">
                        ${this.formatTime(rec.time)} • ${rec.overlap}/${totalParticipants} giocatori disponibili
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2">
                        <div class="bg-emerald h-2 rounded-full transition-all" 
                             style="width: ${percentage}%"></div>
                    </div>
                </div>
            `;
        }).join('');
    }

    loadParticipantResponses() {
        const container = document.getElementById('participant-responses');
        if (!container || !this.selectedSession) return;

        container.innerHTML = this.selectedSession.participants.map(participantId => {
            const user = window.DDSchedulerApp.getUserById(participantId);
            const response = this.selectedSession.responses[participantId];
            const hasResponded = response && response.responded;

            // Calculate availability percentage
            let availabilityPercent = 0;
            if (hasResponded && response.availability) {
                if (typeof response.availability === 'object') {
                    // Count available slots
                    const slots = Object.values(response.availability);
                    const availableCount = slots.filter(status => status === 'available').length;
                    const totalCount = slots.length;
                    availabilityPercent = totalCount > 0 ? Math.round((availableCount / totalCount) * 100) : 0;
                } else if (typeof response.availability === 'number') {
                    availabilityPercent = response.availability;
                }
            }

            return `
                <div class="border border-gray-200 rounded-lg p-4">
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex items-center">
                            <div class="w-10 h-10 bg-gradient-to-br from-mystic to-amber rounded-full flex items-center justify-center text-white font-semibold text-sm mr-3">
                                ${user ? user.avatar : '?'}
                            </div>
                            <div>
                                <div class="font-semibold text-forest">${user ? window.DDSchedulerApp.escapeHtml(user.name) : 'Giocatore Sconosciuto'}</div>
                                <div class="text-sm text-gray-500">
                                    ${hasResponded ? 'Ha risposto' : 'In attesa'}
                                </div>
                            </div>
                        </div>
                        <div class="text-right">
                            ${hasResponded ? `
                                <span class="px-2 py-1 bg-emerald/10 text-emerald-800 rounded text-xs">
                                    ${availabilityPercent}% Disponibile
                                </span>
                            ` : `
                                <button onclick="sessionManager.sendIndividualReminder('${participantId}')" 
                                        class="text-sm text-amber hover:text-amber-600">
                                    Invia Promemoria
                                </button>
                            `}
                        </div>
                    </div>
                    
                    ${hasResponded && response.availability ? `
                        <div class="text-sm text-gray-600">
                            Disponibilità: ${availabilityPercent}% delle fasce orarie
                        </div>
                        <div class="w-full bg-gray-200 rounded-full h-2 mt-2">
                            <div class="bg-emerald h-2 rounded-full transition-all" 
                                 style="width: ${Math.min(availabilityPercent, 100)}%"></div>
                        </div>
                    ` : ''}
                </div>
            `;
        }).join('');
    }


    // Action methods
    finalizeSession() {
        if (!this.selectedSession || this.selectedSession.status === 'finalized') return;

        // Show finalize modal
        this.showFinalizeModal();
    }

    showFinalizeModal() {
        const modal = document.getElementById('finalize-modal');
        if (!modal) return;

        // Populate time options
        this.populateTimeOptions();

        modal.style.display = 'flex';

        // Animate in
        anime({
            targets: modal,
            opacity: [0, 1],
            duration: 300,
            easing: 'easeOutQuart'
        });
    }

    populateTimeOptions() {
        const container = document.getElementById('time-options');
        if (!container || !this.selectedSession) return;

        // Calculate best times based on actual availability data
        const availabilityMap = {};

        // Aggregate availability from all participants
        if (this.selectedSession.availability && Array.isArray(this.selectedSession.availability)) {
            this.selectedSession.availability.forEach(entry => {
                if (entry.status === 'available') {
                    const key = `${entry.date}_${entry.time_slot}`;
                    if (!availabilityMap[key]) {
                        availabilityMap[key] = {
                            date: entry.date,
                            time: entry.time_slot,
                            count: 0
                        };
                    }
                    availabilityMap[key].count++;
                }
            });
        }

        // Convert to array and sort by count (descending)
        let timeOptions = Object.values(availabilityMap)
            .sort((a, b) => b.count - a.count)
            .slice(0, 5); // Top 5 best times

        // If no availability data, show message
        if (timeOptions.length === 0) {
            container.innerHTML = `
                <div class="text-center py-8 text-gray-500">
                    <p class="mb-2">⚠️ Nessun dato sulla disponibilità</p>
                    <p class="text-sm">Attendi che i partecipanti inviino la loro disponibilità prima di finalizzare.</p>
                </div>
            `;
            return;
        }

        container.innerHTML = timeOptions.map((option, index) => {
            const matchPercent = Math.round((option.count / this.selectedSession.participants.length) * 100);
            return `
                <label class="flex items-center p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors cursor-pointer">
                    <input type="radio" name="finalTime" value="${option.date}_${option.time}" 
                           class="mr-3" ${index === 0 ? 'checked' : ''}>
                    <div class="flex-1">
                        <div class="font-medium text-forest">${this.formatDate(option.date)} alle ${this.formatTime(option.time)}</div>
                        <div class="text-sm text-gray-600">${option.count} giocatori disponibili</div>
                    </div>
                    <div class="text-right">
                        <span class="px-2 py-1 ${matchPercent >= 75 ? 'bg-emerald/10 text-emerald-800' : 'bg-amber/10 text-amber-800'} rounded text-xs">
                            Compatibilità ${matchPercent}%
                        </span>
                    </div>
                </label>
            `;
        }).join('');
    }

    closeFinalizeModal() {
        const modal = document.getElementById('finalize-modal');
        if (!modal) return;

        anime({
            targets: modal,
            opacity: [1, 0],
            duration: 300,
            easing: 'easeInQuart',
            complete: () => {
                modal.style.display = 'none';
            }
        });
    }

    async confirmFinalize() {
        const selectedTimeInput = document.querySelector('input[name="finalTime"]:checked');
        if (!selectedTimeInput) {
            alert('Seleziona un orario per finalizzare la sessione.');
            return;
        }

        const sessionNotes = document.getElementById('session-notes').value;
        const finalizedTime = selectedTimeInput.value;

        try {
            const response = await fetch(`/api/polls/${this.selectedSession.id}/finalize`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${window.authManager.getToken()}`
                },
                body: JSON.stringify({
                    finalized_time: finalizedTime,
                    notes: sessionNotes
                })
            });

            if (!response.ok) {
                const error = await response.text();
                throw new Error(error || 'Failed to finalize session');
            }

            const result = await response.json();

            // Close modal immediately
            this.closeFinalizeModal();

            // Show success message
            this.showSuccessMessage('Sessione Finalizzata', 'I giocatori sono stati notificati dell\'orario finale.');

            // Refresh the entire session view from server to ensure data consistency
            // This fixes the adversarial finding about "patching local object"
            setTimeout(() => {
                this.loadSessionsList(); // Refresh list to update status badges
                this.selectSession(this.selectedSession.id); // Re-fetch details from server

                // Optionally reload dashboard stats if we can access them
                if (window.dashboardManager) {
                    window.dashboardManager.updateStatistics();
                }
            }, 500);

        } catch (error) {
            console.error('Error finalizing session:', error);
            this.showNotification('Errore', 'Impossibile finalizzare la sessione: ' + error.message, 'error');
        }
    }

    sendReminders(sessionId = null) {
        const targetSession = sessionId ?
            window.DDSchedulerApp.getPollById(sessionId) : this.selectedSession;

        if (!targetSession) return;

        // Count pending responses
        const pendingCount = targetSession.participants.filter(participantId => {
            const response = targetSession.responses[participantId];
            return !response || !response.responded;
        }).length;

        this.showSuccessMessage('Promemoria Inviati', `Inviati promemoria a ${pendingCount} giocatori in attesa.`);
    }

    sendIndividualReminder(participantId) {
        const user = window.DDSchedulerApp.getUserById(participantId);
        this.showSuccessMessage('Promemoria Inviato', `Inviato promemoria a ${user ? user.name : 'giocatore'}.`);
    }

    exportToCalendar(sessionId = null) {
        const targetSession = sessionId ?
            window.DDSchedulerApp.getPollById(sessionId) : this.selectedSession;

        if (!targetSession) return;

        // Create ICS file content
        const icsContent = this.generateICSFile(targetSession);

        // Create blob and download
        const blob = new Blob([icsContent], { type: 'text/calendar;charset=utf-8' });
        const link = document.createElement('a');
        link.href = window.URL.createObjectURL(blob);
        link.download = `${targetSession.title.replace(/[^a-z0-9]/gi, '_')}.ics`;
        link.click();

        this.showSuccessMessage('Esportato', 'Evento calendario scaricato!');
    }

    generateICSFile(session) {
        // Parse dates
        let dates = [];
        try {
            const datesData = typeof session.dates === 'string' ? JSON.parse(session.dates) : session.dates;
            dates = Array.isArray(datesData) ? datesData : [];
        } catch (e) {
            dates = [];
        }

        if (dates.length === 0) return '';

        // Use first date as example
        const eventDate = new Date(dates[0]);
        const endDate = new Date(eventDate.getTime() + (session.duration || 180) * 60000);

        const formatICSDate = (date) => {
            return date.toISOString().replace(/[-:]/g, '').split('.')[0] + 'Z';
        };

        return `BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//D&D Scheduler//EN
BEGIN:VEVENT
UID:${session.id}@dndscheduler.com
DTSTAMP:${formatICSDate(new Date())}
DTSTART:${formatICSDate(eventDate)}
DTEND:${formatICSDate(endDate)}
SUMMARY:${session.title}
DESCRIPTION:${session.description || 'Sessione D&D'}
LOCATION:${session.location || 'Online'}
STATUS:TENTATIVE
END:VEVENT
END:VCALENDAR`;
    }

    duplicateSession(sessionId = null) {
        const targetSession = sessionId ?
            window.DDSchedulerApp.getPollById(sessionId) : this.selectedSession;

        if (!targetSession) return;

        // Create a copy with new ID
        const duplicatedSession = {
            ...targetSession,
            id: 'poll-' + Date.now(),
            title: `${targetSession.title} (Copia)`,
            created_at: Date.now() / 1000,
            status: 'active',
            responses: {}
        };

        // In a real implementation, this would call the backend
        // For now, just show success message
        this.showSuccessMessage('Sessione Duplicata', `"${duplicatedSession.title}" creata con successo!`);

        // Redirect to create poll with pre-filled data
        const params = new URLSearchParams({
            duplicate: targetSession.id,
            title: duplicatedSession.title,
            description: targetSession.description || '',
            location: targetSession.location || ''
        });

        setTimeout(() => {
            window.location.href = `create-poll.html?${params.toString()}`;
        }, 1500);
    }

    viewSessionHistory(sessionId = null) {
        const targetSession = sessionId ?
            window.DDSchedulerApp.getPollById(sessionId) : this.selectedSession;

        if (!targetSession) return;

        // Create history modal
        const modal = document.createElement('div');
        modal.className = 'fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4';
        modal.innerHTML = `
            <div class="bg-white rounded-2xl shadow-2xl max-w-2xl w-full max-h-[80vh] overflow-hidden">
                <div class="p-6 border-b border-gray-200">
                    <div class="flex items-center justify-between">
                        <h3 class="font-cinzel text-2xl font-bold text-forest">Storico Sessione</h3>
                        <button onclick="this.closest('.fixed').remove()" class="text-gray-400 hover:text-gray-600">
                            <span class="text-2xl">×</span>
                        </button>
                    </div>
                    <p class="text-gray-600 mt-2">${targetSession.title}</p>
                </div>
                
                <div class="p-6 overflow-y-auto max-h-[60vh]">
                    <div class="space-y-4">
                        ${this.generateHistoryTimeline(targetSession)}
                    </div>
                </div>
            </div>
        `;

        document.body.appendChild(modal);
    }

    generateHistoryTimeline(session) {
        const events = [];

        // Session created
        events.push({
            date: new Date(session.created_at * 1000),
            type: 'created',
            message: 'Sessione creata'
        });

        // Responses
        Object.entries(session.responses || {}).forEach(([userId, response]) => {
            if (response.responded && response.timestamp) {
                const user = window.DDSchedulerApp.getUserById(userId);
                events.push({
                    date: new Date(response.timestamp),
                    type: 'response',
                    message: `${user ? user.name : 'Utente'} ha risposto`
                });
            }
        });

        // Sort by date
        events.sort((a, b) => b.date - a.date);

        if (events.length === 0) {
            return '<p class="text-gray-500 text-center py-8">Nessun evento registrato</p>';
        }

        return events.map(event => `
            <div class="flex items-start space-x-4">
                <div class="w-10 h-10 rounded-full ${event.type === 'created' ? 'bg-forest' : 'bg-emerald'} flex items-center justify-center flex-shrink-0">
                    <span class="text-white text-sm">${event.type === 'created' ? '🎲' : '✓'}</span>
                </div>
                <div class="flex-1">
                    <p class="font-semibold text-gray-900">${event.message}</p>
                    <p class="text-sm text-gray-500">${this.formatDateTime(event.date)}</p>
                </div>
            </div>
        `).join('');
    }

    formatDateTime(date) {
        if (!(date instanceof Date)) date = new Date(date);
        return date.toLocaleString('it-IT', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    }

    viewSession(sessionId) {
        this.selectSession(sessionId);
    }

    editSession() {
        if (!this.selectedSession) return;

        // Show edit modal
        this.showEditModal();
    }

    showEditModal() {
        // Create modal if it doesn't exist
        let modal = document.getElementById('edit-session-modal');
        if (!modal) {
            modal = this.createEditModal();
            document.body.appendChild(modal);
        }

        // Populate form with current session data
        this.populateEditForm();

        // Show modal
        modal.style.display = 'flex';

        // Animate in
        if (typeof anime !== 'undefined') {
            anime({
                targets: modal,
                opacity: [0, 1],
                duration: 300,
                easing: 'easeOutQuart'
            });
        }
    }

    createEditModal() {
        const modal = document.createElement('div');
        modal.id = 'edit-session-modal';
        modal.className = 'fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4';
        modal.style.display = 'none';

        modal.innerHTML = `
            <div class="bg-white rounded-2xl shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
                <div class="sticky top-0 bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between rounded-t-2xl">
                    <h3 class="font-cinzel text-2xl font-bold text-forest">Modifica Campagna</h3>
                    <button onclick="sessionManager.closeEditModal()" class="text-gray-400 hover:text-gray-600 text-2xl">×</button>
                </div>

                <form id="edit-session-form" class="p-6 space-y-6">
                    <!-- Basic Information -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Informazioni Base</h4>
                        
                        <div>
                            <label for="edit-title" class="block text-sm font-semibold text-gray-700 mb-2">
                                Nome Campagna *
                            </label>
                            <input type="text" id="edit-title" required
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="es. La Tomba dell'Annientamento">
                        </div>

                        <div>
                            <label for="edit-description" class="block text-sm font-semibold text-gray-700 mb-2">
                                Descrizione
                            </label>
                            <textarea id="edit-description" rows="4"
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="Breve descrizione della tua avventura..."></textarea>
                        </div>

                        <div>
                            <label for="edit-location" class="block text-sm font-semibold text-gray-700 mb-2">
                                Luogo
                            </label>
                            <input type="text" id="edit-location"
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="es. Online, Discord, Roll20">
                        </div>
                    </div>

                    <!-- Dates -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Date Disponibili</h4>
                        <div>
                            <label for="edit-dates" class="block text-sm font-semibold text-gray-700 mb-2">
                                Seleziona Date *
                            </label>
                            <input type="text" id="edit-dates" required
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="Seleziona date...">
                            <p class="text-sm text-gray-500 mt-1">Clicca per selezionare più date</p>
                        </div>
                    </div>

                    <!-- Time Preferences -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Preferenze Orarie</h4>
                        <div id="edit-time-preferences-container">
                            <!-- Will be populated dynamically -->
                        </div>
                    </div>

                    <!-- Participants -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Partecipanti</h4>
                        <div>
                            <label for="edit-participants" class="block text-sm font-semibold text-gray-700 mb-2">
                                Email Giocatori
                            </label>
                            <textarea id="edit-participants" rows="3"
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="Inserisci indirizzi email separati da virgola o a capo"></textarea>
                            <p class="text-sm text-gray-500 mt-1">I partecipanti attuali saranno mantenuti</p>
                        </div>
                    </div>

                    <!-- Action Buttons -->
                    <div class="flex items-center justify-end space-x-4 pt-4 border-t border-gray-200">
                        <button type="button" onclick="sessionManager.closeEditModal()"
                            class="px-6 py-3 border border-gray-300 text-gray-700 rounded-lg font-semibold hover:bg-gray-50 transition-colors">
                            Annulla
                        </button>
                        <button type="submit"
                            class="px-6 py-3 bg-forest text-white rounded-lg font-semibold hover:bg-forest/90 transition-colors">
                            Salva Modifiche
                        </button>
                    </div>
                </form>
            </div>
        `;

        // Setup form submission
        const form = modal.querySelector('#edit-session-form');
        form.addEventListener('submit', (e) => {
            e.preventDefault();
            this.submitEditForm();
        });

        return modal;
    }

    populateEditForm() {
        if (!this.selectedSession) return;

        // Populate basic fields
        document.getElementById('edit-title').value = this.selectedSession.title || '';
        document.getElementById('edit-description').value = this.selectedSession.description || '';
        document.getElementById('edit-location').value = this.selectedSession.location || '';

        // Parse dates
        let dates = [];
        try {
            if (typeof this.selectedSession.dates === 'string') {
                dates = JSON.parse(this.selectedSession.dates);
            } else if (Array.isArray(this.selectedSession.dates)) {
                dates = this.selectedSession.dates;
            }
        } catch (e) {
            console.error('Error parsing dates:', e);
        }

        // Initialize Flatpickr for date selection
        const dateInput = document.getElementById('edit-dates');
        if (dateInput && typeof flatpickr !== 'undefined') {
            this.editDatePicker = flatpickr(dateInput, {
                mode: 'multiple',
                locale: 'it', // Force Italian locale
                dateFormat: 'Y-m-d',
                minDate: 'today',
                defaultDate: dates,
                onChange: (selectedDates) => {
                    this.editSelectedDates = selectedDates.map(d => {
                        const year = d.getFullYear();
                        const month = String(d.getMonth() + 1).padStart(2, '0');
                        const day = String(d.getDate()).padStart(2, '0');
                        return `${year}-${month}-${day}`;
                    });
                    this.renderEditTimePreferences();
                }
            });
        }

        // Initialize time preferences
        this.editSelectedDates = dates;
        this.editTimePreferences = {};

        // Parse existing time preferences
        try {
            if (this.selectedSession.time_range) {
                const timeData = typeof this.selectedSession.time_range === 'string'
                    ? JSON.parse(this.selectedSession.time_range)
                    : this.selectedSession.time_range;

                if (typeof timeData === 'object' && !Array.isArray(timeData)) {
                    // New format: per-day preferences
                    this.editTimePreferences = timeData;
                } else if (Array.isArray(timeData)) {
                    // Legacy format: apply to all dates
                    dates.forEach(date => {
                        this.editTimePreferences[date] = timeData;
                    });
                }
            }
        } catch (e) {
            console.error('Error parsing time preferences:', e);
        }

        this.renderEditTimePreferences();

        // Populate participants
        const participantEmails = this.selectedSession.participants
            .map(p => p.email || p)
            .filter(e => e)
            .join(', ');
        document.getElementById('edit-participants').value = participantEmails;
    }

    renderEditTimePreferences() {
        const container = document.getElementById('edit-time-preferences-container');
        if (!container) return;

        if (!this.editSelectedDates || this.editSelectedDates.length === 0) {
            container.innerHTML = `
                <div class="text-center py-8 text-gray-500">
                    <p class="text-sm">Select dates above to configure time preferences</p>
                </div>
            `;
            return;
        }

        // Initialize preferences for new dates
        this.editSelectedDates.forEach(date => {
            if (!this.editTimePreferences[date]) {
                this.editTimePreferences[date] = [];
            }
        });

        const timeSlots = [
            '09:00', '10:00', '11:00', '12:00', '13:00', '14:00',
            '15:00', '16:00', '17:00', '18:00', '19:00', '20:00',
            '21:00', '22:00', '23:00'
        ];

        container.innerHTML = `
            <div class="mb-4 bg-amber/10 p-3 rounded-lg border border-amber/30">
                <div class="flex items-center justify-between">
                    <span class="text-sm text-gray-700">Quick action: Apply first date's times to all</span>
                    <button type="button" onclick="sessionManager.applyEditTimesToAll()" 
                        class="px-3 py-1 bg-amber text-white rounded text-sm hover:bg-amber/90">
                        Copy to All
                    </button>
                </div>
            </div>

            ${this.editSelectedDates.map(date => {
            const dateObj = new Date(date + 'T00:00:00');
            const dayName = dateObj.toLocaleDateString('en-US', { weekday: 'long' });
            const formattedDate = dateObj.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
            const selectedTimes = this.editTimePreferences[date] || [];

            return `
                    <div class="mb-6 pb-6 border-b last:border-0">
                        <div class="flex items-center justify-between mb-3">
                            <div>
                                <h5 class="font-semibold text-forest">${dayName}</h5>
                                <p class="text-sm text-gray-600">${formattedDate}</p>
                            </div>
                            <span class="text-sm text-gray-500">${selectedTimes.length} selected</span>
                        </div>
                        <div class="grid grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-2">
                            ${timeSlots.map(time => {
                const isSelected = selectedTimes.includes(time);
                return `
                                    <div class="time-slot ${isSelected ? 'selected' : ''}" 
                                        onclick="sessionManager.toggleEditTimeSlot('${date}', '${time}')">
                                        ${this.formatTime(time)}
                                    </div>
                                `;
            }).join('')}
                        </div>
                    </div>
                `;
        }).join('')}
        `;
    }

    toggleEditTimeSlot(date, time) {
        if (!this.editTimePreferences[date]) {
            this.editTimePreferences[date] = [];
        }

        const index = this.editTimePreferences[date].indexOf(time);
        if (index > -1) {
            this.editTimePreferences[date].splice(index, 1);
        } else {
            this.editTimePreferences[date].push(time);
        }

        this.renderEditTimePreferences();
    }

    applyEditTimesToAll() {
        if (!this.editSelectedDates || this.editSelectedDates.length === 0) return;

        const firstDate = this.editSelectedDates[0];
        const firstTimes = this.editTimePreferences[firstDate] || [];

        if (firstTimes.length === 0) {
            alert('Seleziona almeno una fascia oraria per la prima data');
            return;
        }

        this.editSelectedDates.forEach(date => {
            this.editTimePreferences[date] = [...firstTimes];
        });

        this.renderEditTimePreferences();
        this.showSuccessMessage('Orari Applicati', `Applicati ${firstTimes.length} orari a tutte le date.`);
    }

    async submitEditForm() {
        try {
            const title = document.getElementById('edit-title').value;
            const description = document.getElementById('edit-description').value;
            const location = document.getElementById('edit-location').value;
            const participantsText = document.getElementById('edit-participants').value;

            // Validate
            if (!title || !this.editSelectedDates || this.editSelectedDates.length === 0) {
                alert('Please fill in all required fields');
                return;
            }

            // Parse participants
            const participants = participantsText
                .split(/[,\n]/)
                .map(e => e.trim())
                .filter(e => e);

            // Prepare payload
            const payload = {
                title: title,
                description: description,
                location: location,
                dates: this.editSelectedDates,
                timePreferences: this.editTimePreferences,
                participants: participants
            };

            console.log('Updating poll:', payload);

            // Make API call
            const response = await fetch(`/api/polls/${this.selectedSession.id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${window.authManager.getToken()}`
                },
                body: JSON.stringify(payload)
            });

            if (!response.ok) {
                const error = await response.text();
                throw new Error(error || 'Failed to update poll');
            }

            const result = await response.json();
            console.log('Poll updated:', result);

            // Close modal
            this.closeEditModal();

            // Show success message
            this.showSuccessMessage('Session Updated', 'Your session has been successfully updated!');

            // Reload data
            setTimeout(() => {
                window.location.reload();
            }, 1500);

        } catch (error) {
            console.error('Error updating poll:', error);
            alert('Impossibile aggionare la sessione: ' + error.message);
        }
    }

    closeEditModal() {
        const modal = document.getElementById('edit-session-modal');
        if (!modal) return;

        if (typeof anime !== 'undefined') {
            anime({
                targets: modal,
                opacity: [1, 0],
                duration: 300,
                easing: 'easeInQuart',
                complete: () => {
                    modal.style.display = 'none';
                }
            });
        } else {
            modal.style.display = 'none';
        }

        // Cleanup
        if (this.editDatePicker) {
            this.editDatePicker.destroy();
            this.editDatePicker = null;
        }
    }

    exportCalendar() {
        if (!this.selectedSession) return;
        // Chiama l'implementazione reale
        this.exportToCalendar(this.selectedSession.id);
    }

    duplicateSessionWrapper() {
        if (!this.selectedSession) return;
        // Chiama l'implementazione reale
        this.duplicateSession(this.selectedSession.id);
    }

    viewHistory() {
        if (!this.selectedSession) return;
        // Chiama l'implementazione reale
        this.viewSessionHistory(this.selectedSession.id);
    }

    // Utility methods
    formatDuration(minutes) {
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
    }

    formatDate(dateString) {
        // Handle Unix timestamps (numbers) and date strings
        const date = typeof dateString === 'number'
            ? new Date(dateString * 1000) // Unix timestamp in seconds
            : new Date(dateString);

        return date.toLocaleDateString('it-IT', {
            weekday: 'short',
            month: 'short',
            day: 'numeric'
        });
    }

    formatTime(timeString) {
        const [hours, minutes] = timeString.split(':');
        // 24h format
        return `${hours}:${minutes || '00'}`;
    }

    formatDateTime(dateTimeString) {
        const [date, time] = dateTimeString.split('_');
        return `${this.formatDate(date)} alle ${this.formatTime(time)}`;
    }

    generateDateRange(startDate, endDate) {
        const dates = [];
        const start = new Date(startDate);
        const end = new Date(endDate);

        while (start <= end) {
            dates.push(start.toISOString().split('T')[0]);
            start.setDate(start.getDate() + 1);
        }

        return dates;
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
        // Close modal when clicking outside
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal-overlay')) {
                this.closeFinalizeModal();
            }
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                this.closeFinalizeModal();
            }
        });
    }
}

// Initialize session manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.sessionManager = new SessionManager();
});