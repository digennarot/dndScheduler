// Availability Management System
class AvailabilityManager {
    constructor() {
        this.currentPoll = null;
        this.availabilityData = {};
        this.selectedSession = null;

        // Load current user from local storage if available
        const savedUser = localStorage.getItem('currentUser');
        this.currentUser = savedUser ? JSON.parse(savedUser) : null;

        this.init();
    }

    init() {
        this.loadAvailableSessions();
        this.setupEventListeners();
        this.updateUserDisplay(); // Show user info if logged in

        // Listen for data load
        document.addEventListener('pollsLoaded', () => {
            this.loadAvailableSessions();
        });
    }

    updateUserDisplay() {
        const userInfoDisplay = document.getElementById('user-info-display');
        if (!userInfoDisplay) return;

        if (this.currentUser) {
            document.getElementById('user-display-name').textContent = this.currentUser.name;
            document.getElementById('user-display-email').textContent = this.currentUser.email;
            document.getElementById('user-initial').textContent = this.currentUser.name.charAt(0).toUpperCase();
            userInfoDisplay.style.display = 'flex';
        } else {
            userInfoDisplay.style.display = 'none';
        }
    }

    loadAvailableSessions() {
        const container = document.getElementById('session-list');
        if (!container || !window.DDSchedulerApp) return;

        const availablePolls = window.DDSchedulerApp.polls || [];

        if (availablePolls.length === 0) {
            container.innerHTML = `
                <div class="col-span-full text-center py-12">
                    <div class="text-gray-400 text-6xl mb-4">🎲</div>
                    <h3 class="text-xl font-semibold text-gray-600 mb-2">No Active Sessions</h3>
                    <p class="text-gray-500">Check back later or ask your DM to create a new scheduling poll.</p>
                </div>
            `;
            return;
        }

        container.innerHTML = availablePolls.map(poll => {
            const organizer = window.DDSchedulerApp.getUserById(poll.organizer);
            const responseRate = window.DDSchedulerApp.calculateResponseRate(poll);
            const respondedCount = Object.keys(poll.responses || {}).filter(userId =>
                poll.responses[userId].responded
            ).length;

            return `
                <div class="border border-gray-200 rounded-lg p-4 hover:border-amber transition-colors cursor-pointer mystical-glow"
                     onclick="availabilityManager.selectSession('${poll.id}')">
                    <div class="flex items-start justify-between mb-3">
                        <h4 class="font-cinzel text-lg font-semibold text-forest">${poll.title}</h4>
                        <span class="status-indicator ${poll.status === 'active' ? 'status-pending' : 'status-confirmed'}"></span>
                    </div>
                    
                    <p class="text-gray-600 text-sm mb-3 line-clamp-2">${poll.description || 'No description'}</p>
                    
                    <div class="space-y-2 text-sm">
                        <div class="flex items-center justify-between">
                            <span class="text-gray-500">DM:</span>
                            <span class="font-medium">${organizer ? organizer.name : 'Unknown'}</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-500">Duration:</span>
                            <span class="font-medium">${this.formatDuration(poll.duration || 180)}</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-500">Responses:</span>
                            <span class="font-medium">${respondedCount}/${poll.participants.length}</span>
                        </div>
                    </div>
                    
                    <div class="mt-3 pt-3 border-t border-gray-100">
                        <div class="flex items-center justify-between">
                            <span class="text-xs text-gray-500">${poll.status === 'active' ? 'Active' : 'Finalized'}</span>
                            <div class="w-16 h-1 bg-gray-200 rounded-full">
                                <div class="h-1 bg-emerald rounded-full" style="width: ${responseRate}%"></div>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    }

    selectSession(pollId) {
        if (!window.DDSchedulerApp) return;

        this.selectedSession = window.DDSchedulerApp.getPollById(pollId);
        if (!this.selectedSession) return;

        // Check if user is identified
        if (!this.currentUser) {
            this.promptUserIdentification();
            return;
        }

        // Check if user is a participant
        if (!this.selectedSession.participants.includes(this.currentUser.id)) {
            this.joinSession(this.selectedSession.id);
            return;
        }

        this.showAvailabilityInterface();
    }

    promptUserIdentification() {
        const modal = document.createElement('div');
        modal.className = 'fixed inset-0 bg-black/50 flex items-center justify-center z-50';
        modal.innerHTML = `
            <div class="bg-white p-8 rounded-2xl shadow-2xl max-w-md w-full mx-4">
                <h3 class="font-cinzel text-2xl font-bold text-forest mb-4">Unisciti alla Sessione</h3>
                <p class="text-gray-600 mb-6">Accedi o inserisci i tuoi dati per partecipare.</p>
                
                <!-- Tab Switcher -->
                <div class="flex border-b border-gray-200 mb-6">
                    <button id="login-tab" class="flex-1 py-2 font-semibold text-forest border-b-2 border-forest">
                        Accedi
                    </button>
                    <button id="guest-tab" class="flex-1 py-2 font-semibold text-gray-500">
                        Ospite
                    </button>
                </div>
                
                <!-- Login Form -->
                <form id="login-form" class="space-y-4">
                    <div>
                        <label class="block text-sm font-semibold text-gray-700 mb-1">Email</label>
                        <input type="email" id="login-email" required class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent">
                    </div>
                    <div>
                        <label class="block text-sm font-semibold text-gray-700 mb-1">Password</label>
                        <input type="password" id="login-password" required class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent">
                    </div>
                    <button type="submit" class="w-full bg-forest text-white py-3 rounded-lg font-semibold mystical-glow mt-4">
                        Accedi e Continua
                    </button>
                </form>
                
                <!-- Guest Form (hidden by default) -->
                <form id="guest-form" class="space-y-4 hidden">
                    <div>
                        <label class="block text-sm font-semibold text-gray-700 mb-1">Nome</label>
                        <input type="text" id="join-name" required class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent">
                    </div>
                    <div>
                        <label class="block text-sm font-semibold text-gray-700 mb-1">Email</label>
                        <input type="email" id="join-email" required class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent">
                    </div>
                    <button type="submit" class="w-full bg-forest text-white py-3 rounded-lg font-semibold mystical-glow mt-4">
                        Continua come Ospite
                    </button>
                </form>
            </div>
        `;

        document.body.appendChild(modal);

        // Tab switching
        const loginTab = modal.querySelector('#login-tab');
        const guestTab = modal.querySelector('#guest-tab');
        const loginForm = modal.querySelector('#login-form');
        const guestForm = modal.querySelector('#guest-form');

        loginTab.addEventListener('click', () => {
            loginTab.classList.add('text-forest', 'border-b-2', 'border-forest');
            loginTab.classList.remove('text-gray-500');
            guestTab.classList.remove('text-forest', 'border-b-2', 'border-forest');
            guestTab.classList.add('text-gray-500');
            loginForm.classList.remove('hidden');
            guestForm.classList.add('hidden');
        });

        guestTab.addEventListener('click', () => {
            guestTab.classList.add('text-forest', 'border-b-2', 'border-forest');
            guestTab.classList.remove('text-gray-500');
            loginTab.classList.remove('text-forest', 'border-b-2', 'border-forest');
            loginTab.classList.add('text-gray-500');
            guestForm.classList.remove('hidden');
            loginForm.classList.add('hidden');
        });

        // Login form submission
        loginForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const email = document.getElementById('login-email').value;
            const password = document.getElementById('login-password').value;

            try {
                const response = await fetch('/api/auth/login', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({ email, password })
                });

                const data = await response.json();

                if (!response.ok) {
                    throw new Error(data.error || data.message || 'Login fallito');
                }

                // Store user and token
                this.currentUser = data.user;
                localStorage.setItem('authToken', data.token);
                localStorage.setItem('currentUser', JSON.stringify(data.user));
                this.updateUserDisplay();

                modal.remove();

                if (this.selectedSession) {
                    // Check if already a participant
                    const existingParticipant = this.selectedSession.participants?.find(p =>
                        p.email === email || p.id === this.currentUser.id
                    );

                    if (existingParticipant) {
                        this.showAvailabilityInterface();
                    } else {
                        await this.joinSession(this.selectedSession.id);
                    }
                }
            } catch (error) {
                this.showNotification('Errore Login', error.message, 'error');
            }
        });

        // Guest form submission
        guestForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const name = document.getElementById('join-name').value;
            const email = document.getElementById('join-email').value;

            // Store temporarily but don't update UI yet
            const tempUser = {
                id: email.replace(/[^a-zA-Z0-9]/g, '-').toLowerCase(),
                name: name,
                email: email
            };

            modal.remove();

            if (this.selectedSession) {
                // Check if already a participant
                const existingParticipant = this.selectedSession.participants?.find(p =>
                    p.email === email || p.id === tempUser.id
                );

                if (existingParticipant) {
                    // Already joined, just set user and show interface
                    this.currentUser = tempUser;
                    localStorage.setItem('currentUser', JSON.stringify(this.currentUser));
                    this.updateUserDisplay();
                    this.showAvailabilityInterface();
                } else {
                    // Need to join - set tempUser for join request
                    this.currentUser = tempUser;
                    await this.joinSession(this.selectedSession.id);
                }
            }
        });
    }

    async joinSession(pollId) {
        try {
            const response = await fetch(`/api/polls/${pollId}/join`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    name: this.currentUser.name,
                    email: this.currentUser.email
                })
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                throw new Error(errorData.message || 'Failed to join session');
            }

            const data = await response.json();
            this.currentUser.id = data.id;
            this.currentUser.accessToken = data.access_token; // Store access token
            localStorage.setItem('currentUser', JSON.stringify(this.currentUser));
            this.updateUserDisplay(); // Update UI to show user info

            if (this.selectedSession) {
                this.selectedSession.participants.push(this.currentUser.id);
                if (!window.DDSchedulerApp.users.find(u => u.id === this.currentUser.id)) {
                    window.DDSchedulerApp.users.push(this.currentUser);
                }
            }

            this.showAvailabilityInterface();
            this.showNotification('Joined Session', 'You have successfully joined the session!');

            if (window.DDSchedulerApp) {
                await window.DDSchedulerApp.fetchPolls();
            }

        } catch (error) {
            console.error('Error joining session:', error);

            // Clear user data on failed join
            this.currentUser = null;
            localStorage.removeItem('currentUser');
            this.updateUserDisplay(); // Hide user info

            this.showNotification('Error', error.message || 'Failed to join session', 'error');
        }
    }

    showAvailabilityInterface() {
        document.getElementById('availability-section').style.display = 'block';
        this.updateSessionInfo();
        this.generateAvailabilityGrid();
        this.loadGroupOverview();
        this.initializeOverlapChart();

        document.getElementById('availability-section').scrollIntoView({
            behavior: 'smooth'
        });
    }

    updateSessionInfo() {
        if (!this.selectedSession) return;

        const organizer = window.DDSchedulerApp.getUserById(this.selectedSession.organizer);

        document.getElementById('campaign-title').textContent = this.selectedSession.title;
        document.getElementById('campaign-description').textContent = this.selectedSession.description || 'No description';
        document.getElementById('dm-name').textContent = organizer ? organizer.name : 'Unknown';
        document.getElementById('session-duration').textContent = this.formatDuration(this.selectedSession.duration || 180);
        document.getElementById('session-timezone').textContent = this.selectedSession.timezone || 'Not specified';
        document.getElementById('response-deadline').textContent = this.selectedSession.settings?.deadline || 'No deadline';
    }

    generateAvailabilityGrid() {
        const container = document.getElementById('availability-grid');
        if (!container || !this.selectedSession) return;

        console.log('Generating grid for session:', this.selectedSession);

        // Parse dates from the poll
        let dates = [];
        try {
            const datesData = typeof this.selectedSession.dates === 'string'
                ? JSON.parse(this.selectedSession.dates)
                : this.selectedSession.dates;

            if (Array.isArray(datesData)) {
                dates = datesData;
            }
        } catch (e) {
            console.error('Error parsing dates:', e);
            dates = [];
        }

        console.log('Parsed dates:', dates);

        // Parse time slots
        let timeSlots = [];
        try {
            const timeSlotsData = typeof this.selectedSession.time_range === 'string'
                ? JSON.parse(this.selectedSession.time_range)
                : this.selectedSession.time_range;

            if (Array.isArray(timeSlotsData)) {
                timeSlots = timeSlotsData;
            } else {
                timeSlots = ['18:00', '19:00', '20:00', '21:00'];
            }
        } catch (e) {
            console.error('Error parsing time slots:', e);
            timeSlots = ['18:00', '19:00', '20:00', '21:00'];
        }

        console.log('Parsed time slots:', timeSlots);

        if (dates.length === 0) {
            container.innerHTML = '<div class="col-span-full text-center text-gray-500 py-8">No dates available for this session</div>';
            return;
        }

        container.innerHTML = '';

        // Set grid columns dynamically based on number of dates
        // 1 column for time labels + N columns for dates
        container.style.gridTemplateColumns = `80px repeat(${dates.length}, 1fr)`;
        console.log('Grid columns set to:', container.style.gridTemplateColumns);

        // Header row
        container.appendChild(this.createGridCell('Time', 'header'));
        dates.forEach(date => {
            const cell = this.createGridCell(this.formatDate(date), 'header');
            container.appendChild(cell);
        });

        // Time rows
        timeSlots.forEach(time => {
            container.appendChild(this.createGridCell(this.formatTime(time), 'time-label'));

            dates.forEach(date => {
                const cellId = `${date}_${time}`;
                const cell = this.createGridCell('', 'clickable', cellId);
                cell.onclick = () => this.toggleAvailability(cellId);
                container.appendChild(cell);
            });
        });
    }

    createGridCell(content, type, id = null) {
        const cell = document.createElement('div');
        cell.className = `grid-cell ${type}`;
        cell.textContent = content;
        if (id) cell.id = id;
        return cell;
    }

    toggleAvailability(cellId) {
        console.log('toggleAvailability called with cellId:', cellId);
        const cell = document.getElementById(cellId);
        console.log('Found cell:', cell);

        if (!cell) {
            console.error('Cell not found for id:', cellId);
            return;
        }

        if (!this.availabilityData[cellId]) {
            this.availabilityData[cellId] = 'available';
            cell.className = 'grid-cell available';
            console.log('Set to available');
        } else if (this.availabilityData[cellId] === 'available') {
            this.availabilityData[cellId] = 'tentative';
            cell.className = 'grid-cell tentative';
            console.log('Set to tentative');
        } else if (this.availabilityData[cellId] === 'tentative') {
            this.availabilityData[cellId] = 'busy';
            cell.className = 'grid-cell busy';
            console.log('Set to busy');
        } else {
            delete this.availabilityData[cellId];
            cell.className = 'grid-cell clickable';
            console.log('Cleared');
        }

        console.log('Current availabilityData:', this.availabilityData);
    }

    loadGroupOverview() {
        if (!this.selectedSession) return;

        const totalPlayers = this.selectedSession.participants.length;
        const respondedCount = Object.keys(this.selectedSession.responses || {}).filter(userId =>
            this.selectedSession.responses[userId].responded
        ).length;

        document.getElementById('total-players').textContent = totalPlayers;
        document.getElementById('responded-count').textContent = respondedCount;

        const progressPercent = totalPlayers > 0 ? (respondedCount / totalPlayers) * 100 : 0;
        document.getElementById('response-progress').style.width = `${progressPercent}%`;

        const playerStatusContainer = document.getElementById('player-status');
        playerStatusContainer.innerHTML = this.selectedSession.participants.map(participantId => {
            const user = window.DDSchedulerApp.getUserById(participantId);
            const hasResponded = this.selectedSession.responses[participantId]?.responded;

            return `
                <div class="flex items-center justify-between text-sm">
                    <div class="flex items-center">
                        <span class="status-indicator ${hasResponded ? 'status-responded' : 'status-pending'}"></span>
                        <span>${user ? user.name : 'Unknown Player'}</span>
                    </div>
                    <span class="text-gray-500">${hasResponded ? 'Responded' : 'Pending'}</span>
                </div>
            `;
        }).join('');
    }

    initializeOverlapChart() {
        const chartContainer = document.getElementById('overlap-chart');
        if (!chartContainer || !this.selectedSession) return;

        const chart = echarts.init(chartContainer);

        let timeSlots = [];
        try {
            const timeSlotsData = typeof this.selectedSession.time_range === 'string'
                ? JSON.parse(this.selectedSession.time_range)
                : this.selectedSession.time_range;
            timeSlots = Array.isArray(timeSlotsData) ? timeSlotsData : ['18:00', '19:00', '20:00', '21:00'];
        } catch (e) {
            timeSlots = ['18:00', '19:00', '20:00', '21:00'];
        }

        const times = timeSlots.map(time => this.formatTime(time));
        const counts = timeSlots.map(() => Math.floor(Math.random() * this.selectedSession.participants.length));

        const option = {
            grid: {
                left: '15%',
                right: '10%',
                top: '10%',
                bottom: '15%'
            },
            xAxis: {
                type: 'category',
                data: times,
                axisLabel: {
                    fontSize: 10,
                    color: '#666'
                }
            },
            yAxis: {
                type: 'value',
                max: this.selectedSession.participants.length,
                axisLabel: {
                    fontSize: 10,
                    color: '#666'
                }
            },
            series: [{
                data: counts,
                type: 'bar',
                itemStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: '#6b5b95' },
                        { offset: 1, color: '#4a7c59' }
                    ])
                }
            }],
            tooltip: {
                trigger: 'axis',
                formatter: function (params) {
                    return `${params[0].name}<br/>Available: ${params[0].value} players`;
                }
            }
        };

        chart.setOption(option);

        window.addEventListener('resize', () => {
            chart.resize();
        });
    }

    // Bulk action methods
    markAllAvailable() {
        document.querySelectorAll('.grid-cell.clickable').forEach(cell => {
            this.availabilityData[cell.id] = 'available';
            cell.className = 'grid-cell available';
        });
    }

    markAllBusy() {
        document.querySelectorAll('.grid-cell.clickable, .grid-cell.available, .grid-cell.tentative').forEach(cell => {
            this.availabilityData[cell.id] = 'busy';
            cell.className = 'grid-cell busy';
        });
    }

    clearAll() {
        document.querySelectorAll('.grid-cell.available, .grid-cell.tentative, .grid-cell.busy').forEach(cell => {
            delete this.availabilityData[cell.id];
            cell.className = 'grid-cell clickable';
        });
    }

    markWeekends() {
        this.clearAll();
        document.querySelectorAll('.grid-cell.clickable').forEach(cell => {
            const [date] = cell.id.split('_');
            const dayOfWeek = new Date(date).getDay();
            if (dayOfWeek === 0 || dayOfWeek === 6) {
                this.availabilityData[cell.id] = 'available';
                cell.className = 'grid-cell available';
            } else {
                this.availabilityData[cell.id] = 'busy';
                cell.className = 'grid-cell busy';
            }
        });
    }

    markWeekdays() {
        this.clearAll();
        document.querySelectorAll('.grid-cell.clickable').forEach(cell => {
            const [date] = cell.id.split('_');
            const dayOfWeek = new Date(date).getDay();
            if (dayOfWeek >= 1 && dayOfWeek <= 5) {
                this.availabilityData[cell.id] = 'available';
                cell.className = 'grid-cell available';
            } else {
                this.availabilityData[cell.id] = 'busy';
                cell.className = 'grid-cell busy';
            }
        });
    }

    resetAvailability() {
        this.availabilityData = {};
        this.generateAvailabilityGrid();
    }

    saveAsDraft() {
        localStorage.setItem(`availability_draft_${this.selectedSession.id}`, JSON.stringify(this.availabilityData));
        this.showNotification('Draft Saved', 'Your availability has been saved as a draft.');
    }

    async submitAvailability() {
        if (Object.keys(this.availabilityData).length === 0) {
            this.showNotification('No availability marked', 'Please mark your availability before submitting.', 'error');
            return;
        }

        try {
            if (!this.selectedSession) throw new Error('No session selected');
            if (!this.currentUser?.id) {
                this.promptUserIdentification();
                throw new Error('User not identified');
            }

            if (!this.currentUser?.accessToken) {
                this.showNotification('Authorization Error', 'Access token missing. Please rejoin the session.', 'error');
                throw new Error('Access token missing');
            }

            const availabilityList = Object.entries(this.availabilityData).map(([key, status]) => {
                const [date, time] = key.split('_');
                return {
                    date: date,
                    timeSlot: time,
                    status: status
                };
            });

            const response = await fetch(`/api/polls/${this.selectedSession.id}/participants/${this.currentUser.id}/availability`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    availability: availabilityList,
                    access_token: this.currentUser.accessToken
                })
            });

            if (!response.ok) {
                const errorData = await response.text();
                throw new Error(errorData || 'Failed to submit availability');
            }

            this.showSuccessMessage();
            localStorage.removeItem(`availability_draft_${this.selectedSession.id}`);

            if (window.DDSchedulerApp) {
                await window.DDSchedulerApp.fetchPolls();
            }

            setTimeout(() => {
                window.location.href = 'index.html';
            }, 2000);

        } catch (error) {
            console.error('Error submitting availability:', error);
            this.showNotification('Submission Failed', error.message || 'Failed to submit availability. Please try again.', 'error');
        }
    }

    showSuccessMessage() {
        const message = document.createElement('div');
        message.className = 'fixed inset-0 bg-black/50 flex items-center justify-center z-50';
        message.innerHTML = `
            <div class="bg-white p-8 rounded-2xl shadow-2xl max-w-md mx-4 text-center">
                <div class="w-16 h-16 bg-emerald rounded-full flex items-center justify-center mx-auto mb-4">
                    <span class="text-white text-2xl">✓</span>
                </div>
                <h3 class="font-cinzel text-2xl font-bold text-forest mb-2">Availability Submitted!</h3>
                <p class="text-gray-600 mb-4">Thank you for marking your availability. The DM will be notified.</p>
                <p class="text-sm text-gray-500">Redirecting to dashboard...</p>
            </div>
        `;

        document.body.appendChild(message);

        anime({
            targets: message,
            opacity: [0, 1],
            duration: 300,
            easing: 'easeOutQuart'
        });
    }

    showNotification(title, message, type = 'success') {
        const notification = document.createElement('div');
        const bgColor = type === 'error' ? 'bg-deep-red' : 'bg-forest';

        notification.className = `fixed top-20 right-4 ${bgColor} text-white p-4 rounded-lg shadow-lg z-50 max-w-sm`;
        notification.innerHTML = `
            <div class="flex items-start space-x-3">
                <div class="w-8 h-8 bg-white/20 rounded-full flex items-center justify-center flex-shrink-0">
                    <span class="text-white text-sm">${type === 'error' ? '!' : '✓'}</span>
                </div>
                <div class="flex-1">
                    <div class="font-semibold text-sm">${title}</div>
                    <div class="text-xs opacity-90 mt-1">${message}</div>
                </div>
                <button onclick="this.parentElement.parentElement.remove()" class="text-white/60 hover:text-white text-sm">×</button>
            </div>
        `;

        document.body.appendChild(notification);

        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    setupEventListeners() {
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch (e.key) {
                    case 's':
                        e.preventDefault();
                        this.saveAsDraft();
                        break;
                    case 'Enter':
                        e.preventDefault();
                        this.submitAvailability();
                        break;
                }
            }
        });
    }

    // Utility methods
    formatDuration(minutes) {
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        return mins > 0 ? `${hours}h ${mins}m` : `${hours} hours`;
    }

    formatDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('en-US', {
            weekday: 'short',
            month: 'short',
            day: 'numeric'
        });
    }

    formatTime(timeString) {
        const [hours, minutes] = timeString.split(':');
        const hour12 = hours > 12 ? hours - 12 : (hours == 0 ? 12 : hours);
        const ampm = hours >= 12 ? 'PM' : 'AM';
        return `${hour12}:00 ${ampm}`;
    }
}

// Initialize availability manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.availabilityManager = new AvailabilityManager();
});