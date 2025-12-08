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

        // TODO: Filter by organizer once authentication is implemented
        // const userSessions = window.DDSchedulerApp.polls.filter(poll =>
        //     poll.organizer === this.currentUser?.id
        // );
        const userSessions = window.DDSchedulerApp.polls;

        if (userSessions.length === 0) {
            container.innerHTML = `
                <div class="text-center py-12">
                    <div class="text-gray-400 text-6xl mb-4">📅</div>
                    <h3 class="text-xl font-semibold text-gray-600 mb-2">No Sessions Found</h3>
                    <p class="text-gray-500 mb-4">You haven't created any scheduling sessions yet.</p>
                    <button onclick="window.location.href='create-poll.html'" 
                            class="action-button primary">
                        Create Your First Session
                    </button>
                </div>
            `;
            return;
        }

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

            return `
                <div class="border border-gray-200 rounded-lg p-4 hover:border-amber transition-colors cursor-pointer mystical-glow"
                     onclick="sessionManager.selectSession('${session.id}')">
                    <div class="flex items-start justify-between mb-3">
                        <div class="flex-1">
                            <h4 class="font-cinzel text-lg font-semibold text-forest mb-1">${session.title}</h4>
                            <p class="text-gray-600 text-sm line-clamp-2">${session.description}</p>
                        </div>
                        <div class="ml-4">
                            <span class="status-badge status-${status}">${status}</span>
                        </div>
                    </div>
                    
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-4">
                        <div>
                            <span class="text-gray-500">Duration:</span>
                            <span class="font-medium ml-2">${this.formatDuration(duration)}</span>
                        </div>
                        <div>
                            <span class="text-gray-500">Players:</span>
                            <span class="font-medium ml-2">${participants.length}</span>
                        </div>
                        <div>
                            <span class="text-gray-500">Responses:</span>
                            <span class="font-medium ml-2">${respondedCount}/${participants.length}</span>
                        </div>
                        <div>
                            <span class="text-gray-500">Response Rate:</span>
                            <span class="font-medium ml-2">${responseRate}%</span>
                        </div>
                    </div>
                    
                    <div class="flex items-center justify-between">
                        <div class="text-sm text-gray-500">
                            Created ${this.formatDate(createdAt)}
                        </div>
                        <div class="flex space-x-2">
                            ${session.status === 'active' ? `
                                <button onclick="event.stopPropagation(); sessionManager.sendReminders('${session.id}')" 
                                        class="text-sm text-amber hover:text-amber-600">
                                    Send Reminders
                                </button>
                            ` : ''}
                            <button onclick="event.stopPropagation(); sessionManager.viewSession('${session.id}')" 
                                    class="text-sm text-forest hover:text-forest-600">
                                View Details
                            </button>
                        </div>
                    </div>
                    
                    ${session.bestTimes && session.bestTimes.length > 0 ? `
                        <div class="mt-3 pt-3 border-t border-gray-100">
                            <div class="text-sm text-gray-600 mb-2">Best Times:</div>
                            <div class="flex flex-wrap gap-2">
                                ${session.bestTimes.slice(0, 3).map(time => `
                                    <span class="px-2 py-1 bg-amber/10 text-amber-800 rounded text-xs">
                                        ${this.formatDateTime(time)}
                                    </span>
                                `).join('')}
                            </div>
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
            if (session.created_at) {
                const createdDate = new Date(session.created_at);
                const daysDiff = Math.floor((today - createdDate) / (1000 * 60 * 60 * 24));
                if (daysDiff >= 0 && daysDiff < 7) {
                    activity[6 - daysDiff]++;
                }
            }

            // Count responses
            Object.values(session.responses || {}).forEach(response => {
                if (response.responded && response.timestamp) {
                    const responseDate = new Date(response.timestamp);
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
                    return `${params[0].name}: ${params[0].value} activities`;
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
            if (session.created_at) {
                activities.push({
                    type: 'created',
                    message: `Created session: ${session.title}`,
                    timestamp: new Date(session.created_at),
                    icon: '🎲'
                });
            }

            // Response activities
            Object.entries(session.responses || {}).forEach(([userId, response]) => {
                if (response.responded && response.timestamp) {
                    const user = window.DDSchedulerApp.getUserById(userId);
                    activities.push({
                        type: 'response',
                        message: `${user?.name || 'A player'} responded to ${session.title}`,
                        timestamp: new Date(response.timestamp),
                        icon: '✓'
                    });
                }
            });

            // Finalization activity
            if (session.status === 'finalized' && session.finalizedAt) {
                activities.push({
                    type: 'finalized',
                    message: `${session.title} session finalized`,
                    timestamp: new Date(session.finalizedAt),
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
                    <div class="text-sm">No recent activity</div>
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

        if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
        if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
        if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
        return 'Just now';
    }

    selectSession(sessionId) {
        if (!window.DDSchedulerApp) return;

        this.selectedSession = window.DDSchedulerApp.getPollById(sessionId);
        if (!this.selectedSession) return;

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

        document.getElementById('detail-title').textContent = this.selectedSession.title;
        document.getElementById('detail-description').textContent = this.selectedSession.description;

        // Update finalize button
        const finalizeBtn = document.getElementById('finalize-btn');
        if (this.selectedSession.status === 'finalized') {
            finalizeBtn.textContent = 'Session Finalized';
            finalizeBtn.disabled = true;
            finalizeBtn.classList.add('opacity-50', 'cursor-not-allowed');
        } else {
            finalizeBtn.textContent = 'Finalize Time';
            finalizeBtn.disabled = false;
            finalizeBtn.classList.remove('opacity-50', 'cursor-not-allowed');
        }
    }

    generateOverlapHeatmap() {
        const container = document.getElementById('overlap-heatmap');
        if (!container || !this.selectedSession) return;

        // Generate date range and time slots
        const dates = this.generateDateRange(this.selectedSession.dateRange.start, this.selectedSession.dateRange.end);
        const timeSlots = this.selectedSession.timeSlots || ['18:00', '19:00', '20:00', '21:00'];

        // Clear existing heatmap
        container.innerHTML = '';

        // Add header row
        container.appendChild(this.createHeatmapCell('Time', 'header'));
        dates.forEach(date => {
            container.appendChild(this.createHeatmapCell(this.formatDate(date), 'header'));
        });

        // Add time rows with overlap data
        timeSlots.forEach(time => {
            // Time label
            container.appendChild(this.createHeatmapCell(this.formatTime(time), 'time-label'));

            // Date cells with overlap counts
            dates.forEach(date => {
                const overlapCount = this.calculateOverlap(date, time);
                const cell = this.createHeatmapCell(overlapCount > 0 ? `${overlapCount}` : '', 'data');

                if (overlapCount > 0) {
                    // Add overlap indicator
                    const indicator = document.createElement('div');
                    indicator.className = 'overlap-indicator';
                    indicator.textContent = overlapCount;
                    cell.appendChild(indicator);

                    // Color code based on overlap
                    const intensity = Math.min(overlapCount / this.selectedSession.participants.length, 1);
                    cell.style.backgroundColor = `rgba(74, 124, 89, ${intensity * 0.8})`;
                    cell.style.color = intensity > 0.5 ? 'white' : '#1a3d2e';
                }

                container.appendChild(cell);
            });
        });
    }

    createHeatmapCell(content, type) {
        const cell = document.createElement('div');
        cell.className = `heatmap-cell ${type}`;
        cell.textContent = content;
        return cell;
    }

    calculateOverlap(date, time) {
        // Simulate overlap calculation based on responses
        if (!this.selectedSession.responses) return 0;

        let overlapCount = 0;
        Object.values(this.selectedSession.responses).forEach(response => {
            if (response.responded && response.availability) {
                const cellId = `${date}_${time}`;
                if (response.availability[cellId] === 'available') {
                    overlapCount++;
                }
            }
        });

        return Math.floor(Math.random() * this.selectedSession.participants.length); // Mock data
    }

    loadRecommendedTimes() {
        const container = document.getElementById('recommended-times');
        if (!container || !this.selectedSession) return;

        // Generate recommended times based on overlap analysis
        const recommendedTimes = [
            {
                date: '2025-01-18',
                time: '19:00',
                overlap: 4,
                confidence: 'High'
            },
            {
                date: '2025-01-25',
                time: '18:30',
                overlap: 3,
                confidence: 'Medium'
            }
        ];

        container.innerHTML = recommendedTimes.map(rec => {
            // Calculate percentage (capped at 100%)
            const percentage = Math.min(Math.round((rec.overlap / this.selectedSession.participants.length) * 100), 100);

            // Determine confidence color and description
            let confidenceColor = 'bg-emerald/10 text-emerald-800';
            let confidenceText = '';

            if (percentage >= 75) {
                confidenceColor = 'bg-emerald/10 text-emerald-800';
                confidenceText = `${percentage}% disponibili`;
            } else if (percentage >= 50) {
                confidenceColor = 'bg-amber/10 text-amber-800';
                confidenceText = `${percentage}% disponibili`;
            } else {
                confidenceColor = 'bg-deep-red/10 text-deep-red';
                confidenceText = `Solo ${percentage}%`;
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
                        ${this.formatTime(rec.time)} • ${rec.overlap}/${this.selectedSession.participants.length} giocatori disponibili
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2">
                        <div class="bg-emerald h-2 rounded-full transition-all" 
                             style="width: ${percentage}%"></div>
                    </div>
                    <div class="text-xs text-gray-500 mt-1 text-right">
                        ${percentage}%
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
                                <div class="font-semibold text-forest">${user ? user.name : 'Unknown Player'}</div>
                                <div class="text-sm text-gray-500">
                                    ${hasResponded ? 'Responded' : 'Pending Response'}
                                </div>
                            </div>
                        </div>
                        <div class="text-right">
                            ${hasResponded ? `
                                <span class="px-2 py-1 bg-emerald/10 text-emerald-800 rounded text-xs">
                                    ${availabilityPercent}% Available
                                </span>
                            ` : `
                                <button onclick="sessionManager.sendIndividualReminder('${participantId}')" 
                                        class="text-sm text-amber hover:text-amber-600">
                                    Send Reminder
                                </button>
                            `}
                        </div>
                    </div>
                    
                    ${hasResponded && response.availability ? `
                        <div class="text-sm text-gray-600">
                            Availability: ${availabilityPercent}% of time slots
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
                    <p class="mb-2">⚠️ No availability data yet</p>
                    <p class="text-sm">Wait for participants to submit their availability before finalizing.</p>
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
                        <div class="font-medium text-forest">${this.formatDate(option.date)} at ${this.formatTime(option.time)}</div>
                        <div class="text-sm text-gray-600">${option.count} player${option.count !== 1 ? 's' : ''} available</div>
                    </div>
                    <div class="text-right">
                        <span class="px-2 py-1 ${matchPercent >= 75 ? 'bg-emerald/10 text-emerald-800' : 'bg-amber/10 text-amber-800'} rounded text-xs">
                            ${matchPercent}% match
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

    confirmFinalize() {
        const selectedTime = document.querySelector('input[name="finalTime"]:checked');
        if (!selectedTime) {
            alert('Please select a time to finalize the session.');
            return;
        }

        const sessionNotes = document.getElementById('session-notes').value;

        // Update session status
        if (this.selectedSession) {
            this.selectedSession.status = 'finalized';
            this.selectedSession.finalizedTime = selectedTime.value;
            this.selectedSession.notes = sessionNotes;
            this.selectedSession.finalizedAt = new Date().toISOString();
        }

        // Close modal
        this.closeFinalizeModal();

        // Show success message
        this.showSuccessMessage('Session Finalized', 'Players have been notified of the final session time.');

        // Refresh the view
        setTimeout(() => {
            this.loadSessionsList();
            this.updateSessionDetails();
        }, 1000);
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

        this.showSuccessMessage('Reminders Sent', `Sent reminders to ${pendingCount} pending players.`);
    }

    sendIndividualReminder(participantId) {
        const user = window.DDSchedulerApp.getUserById(participantId);
        this.showSuccessMessage('Reminder Sent', `Sent reminder to ${user ? user.name : 'player'}.`);
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
                    <h3 class="font-cinzel text-2xl font-bold text-forest">Edit Session</h3>
                    <button onclick="sessionManager.closeEditModal()" class="text-gray-400 hover:text-gray-600 text-2xl">×</button>
                </div>

                <form id="edit-session-form" class="p-6 space-y-6">
                    <!-- Basic Information -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Basic Information</h4>
                        
                        <div>
                            <label for="edit-title" class="block text-sm font-semibold text-gray-700 mb-2">
                                Campaign Name *
                            </label>
                            <input type="text" id="edit-title" required
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="e.g., Tomb of Annihilation">
                        </div>

                        <div>
                            <label for="edit-description" class="block text-sm font-semibold text-gray-700 mb-2">
                                Description
                            </label>
                            <textarea id="edit-description" rows="4"
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="Brief description of your campaign..."></textarea>
                        </div>

                        <div>
                            <label for="edit-location" class="block text-sm font-semibold text-gray-700 mb-2">
                                Location
                            </label>
                            <input type="text" id="edit-location"
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="e.g., Online, Discord, Roll20">
                        </div>
                    </div>

                    <!-- Dates -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Available Dates</h4>
                        <div>
                            <label for="edit-dates" class="block text-sm font-semibold text-gray-700 mb-2">
                                Select Dates *
                            </label>
                            <input type="text" id="edit-dates" required
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="Select dates...">
                            <p class="text-sm text-gray-500 mt-1">Click to select multiple dates</p>
                        </div>
                    </div>

                    <!-- Time Preferences -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Time Preferences</h4>
                        <div id="edit-time-preferences-container">
                            <!-- Will be populated dynamically -->
                        </div>
                    </div>

                    <!-- Participants -->
                    <div class="space-y-4">
                        <h4 class="font-semibold text-lg text-forest border-b pb-2">Participants</h4>
                        <div>
                            <label for="edit-participants" class="block text-sm font-semibold text-gray-700 mb-2">
                                Player Emails
                            </label>
                            <textarea id="edit-participants" rows="3"
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent"
                                placeholder="Enter email addresses separated by commas or line breaks"></textarea>
                            <p class="text-sm text-gray-500 mt-1">Current participants will be preserved</p>
                        </div>
                    </div>

                    <!-- Action Buttons -->
                    <div class="flex items-center justify-end space-x-4 pt-4 border-t border-gray-200">
                        <button type="button" onclick="sessionManager.closeEditModal()"
                            class="px-6 py-3 border border-gray-300 text-gray-700 rounded-lg font-semibold hover:bg-gray-50 transition-colors">
                            Cancel
                        </button>
                        <button type="submit"
                            class="px-6 py-3 bg-forest text-white rounded-lg font-semibold hover:bg-forest/90 transition-colors">
                            Save Changes
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
            alert('Please select at least one time slot for the first date');
            return;
        }

        this.editSelectedDates.forEach(date => {
            this.editTimePreferences[date] = [...firstTimes];
        });

        this.renderEditTimePreferences();
        this.showSuccessMessage('Times Applied', `Applied ${firstTimes.length} time slot(s) to all ${this.editSelectedDates.length} dates`);
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
            alert('Failed to update session: ' + error.message);
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

        return date.toLocaleDateString('en-US', {
            weekday: 'short',
            month: 'short',
            day: 'numeric'
        });
    }

    formatTime(timeString) {
        const [hours, minutes] = timeString.split(':');
        const hour12 = hours > 12 ? hours - 12 : hours;
        const ampm = hours >= 12 ? 'PM' : 'AM';
        return `${hour12}:${minutes || '00'} ${ampm}`;
    }

    formatDateTime(dateTimeString) {
        const [date, time] = dateTimeString.split('_');
        return `${this.formatDate(date)} at ${this.formatTime(time)}`;
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