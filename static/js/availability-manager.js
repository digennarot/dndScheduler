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
            this.checkUrlForSession();
        });
    }

    checkUrlForSession() {
        // Support both query param ?id=123 and path /p/123
        const urlParams = new URLSearchParams(window.location.search);
        let pollId = urlParams.get('id');

        if (!pollId) {
            const pathMatch = window.location.pathname.match(/\/p\/([^\/]+)/);
            if (pathMatch) {
                pollId = pathMatch[1];
            }
        }

        if (pollId) {
            this.selectSession(pollId);
        }
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
                    <h3 class="text-xl font-semibold text-gray-600 mb-2">Nessuna Sessione Attiva</h3>
                    <p class="text-gray-500">Torna più tardi o chiedi al tuo DM di creare un nuovo sondaggio.</p>
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
                    
                    <p class="text-gray-600 text-sm mb-3 line-clamp-2">${poll.description || 'Nessuna descrizione'}</p>
                    
                    <div class="space-y-2 text-sm">
                        <div class="flex items-center justify-between">
                            <span class="text-gray-500">DM:</span>
                            <span class="font-medium">${organizer ? organizer.name : 'Sconosciuto'}</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-500">Durata:</span>
                            <span class="font-medium">${this.formatDuration(poll.duration || 180)}</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-500">Risposte:</span>
                            <span class="font-medium">${respondedCount}/${poll.participants.length}</span>
                        </div>
                    </div>
                    
                    <div class="mt-3 pt-3 border-t border-gray-100">
                        <div class="flex items-center justify-between">
                            <span class="text-xs text-gray-500">${poll.status === 'active' ? 'Attiva' : 'Finalizzata'}</span>
                            <div class="w-16 h-1 bg-gray-200 rounded-full">
                                <div class="h-1 bg-emerald rounded-full" style="width: ${responseRate}%"></div>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
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
                throw new Error(errorData.message || 'Impossibile unirsi alla sessione');
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
            this.showNotification('Sessione Partecipata', 'Ti sei unito con successo alla sessione!');

            if (window.DDSchedulerApp) {
                await window.DDSchedulerApp.fetchPolls();
            }

        } catch (error) {
            console.error('Error joining session:', error);

            // Clear user data on failed join
            this.currentUser = null;
            localStorage.removeItem('currentUser');
            this.updateUserDisplay(); // Hide user info

            this.showNotification('Errore', error.message || 'Impossibile unirsi alla sessione', 'error');
        }
    }

    async identifyUser() {
        if (this.currentUser) return this.currentUser;

        // Check local storage for token
        const token = localStorage.getItem('authToken');
        if (!token) return null;

        try {
            const response = await fetch('/api/auth/me', {
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });

            if (response.ok) {
                const user = await response.json();
                this.currentUser = user;
                // Preserve stored token
                this.currentUser.accessToken = token;
                localStorage.setItem('currentUser', JSON.stringify(user));
                this.updateUserDisplay();
                return user;
            } else {
                // Token invalid
                localStorage.removeItem('authToken');
                localStorage.removeItem('currentUser');
            }
        } catch (e) {
            console.error("Auth check failed", e);
        }
        return null;
    }

    async selectSession(pollId) {
        if (!window.DDSchedulerApp) return;

        this.selectedSession = window.DDSchedulerApp.getPollById(pollId);

        // If not found (e.g. private poll via direct link), try fetching it
        if (!this.selectedSession) {
            console.log('Poll not found locally, fetching...', pollId);
            this.selectedSession = await window.DDSchedulerApp.fetchSinglePoll(pollId);
        }

        if (!this.selectedSession) {
            console.error('Session not found', pollId);
            this.showNotification('Errore', 'Sessione non trovata o non accessibile.', 'error');
            return;
        }

        // Identify user first
        await this.identifyUser();

        // Check if user is identified
        if (!this.currentUser) {
            this.promptUserIdentification();
            return;
        }

        // Check if user is a participant (BY USER ID or ID)
        // 1. Try matching User ID (AuthUser)
        const participantRecord = this.selectedSession.participants.find(p => p.user_id && p.user_id === this.currentUser.id);

        if (participantRecord) {
            // Found linked participant!
            this.currentUser.participantId = participantRecord.id;
            // No access token needed if owned by user
            this.showAvailabilityInterface();
            return;
        }

        // 2. Try matching Participant ID (Guest/Legacy)
        if (this.selectedSession.participants.some(p => p.id === this.currentUser.id)) {
            // Current User ID IS the participant ID (Guest flow)
            this.currentUser.participantId = this.currentUser.id;
            // Logic in joinSession sets currentUser.id = participant.id
            // So this branch usually handles guests or legacy manual joins.
        } else {
            // Use joinSession to become a participant
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
                        <label class="block text-sm font-semibold text-gray-700 mb-1">Email (Opzionale)</label>
                        <input type="email" id="join-email" placeholder="Lascia vuoto per restare anonimo" class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent">
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
            const emailInput = document.getElementById('join-email').value;
            const email = emailInput.trim() === '' ? null : emailInput.trim();

            // Store temporarily but don't update UI yet
            // Use random ID for anonymous if no email
            const tempUser = {
                id: email ? email.replace(/[^a-zA-Z0-9]/g, '-').toLowerCase() : `anon-${Date.now()}`,
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

    showAvailabilityInterface() {
        document.getElementById('availability-section').style.display = 'block';
        this.updateSessionInfo();
        this.generateAvailabilityGrid();
        this.loadGroupOverview();
        this.renderBestTimes();
        this.initializeOverlapChart();

        document.getElementById('availability-section').scrollIntoView({
            behavior: 'smooth'
        });
    }

    renderBestTimes() {
        const container = document.getElementById('best-times');
        if (!container || !this.selectedSession) return;

        const heatmapData = this.calculateHeatmapData();
        const totalPlayers = this.selectedSession.participants.length;

        // Convert to array and sort
        const sortedSlots = Object.entries(heatmapData)
            .map(([key, data]) => {
                const [date, time] = key.split('_');
                return {
                    key,
                    date,
                    time,
                    count: data.count,
                    voters: data.voters
                };
            })
            .sort((a, b) => b.count - a.count);

        const topSlots = sortedSlots.slice(0, 3);

        if (topSlots.length === 0 || topSlots[0].count === 0) {
            container.innerHTML = '<p class="text-sm text-gray-500 italic">Nessuna disponibilità registrata.</p>';
            return;
        }

        container.innerHTML = topSlots.map((slot, index) => {
            const dateObj = new Date(slot.date);
            const dateStr = dateObj.toLocaleDateString('it-IT', { weekday: 'short', day: 'numeric', month: 'short' });
            const percentage = Math.round((slot.count / totalPlayers) * 100);

            // Medals for top 3
            const medals = ['🥇', '🥈', '🥉'];
            const medal = medals[index] || '';

            return `
                <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg border border-gray-100">
                    <div class="flex items-center gap-3">
                        <span class="text-xl">${medal}</span>
                        <div>
                            <div class="font-medium text-forest">${dateStr}</div>
                            <div class="text-sm text-gray-600">${this.formatTime(slot.time)}</div>
                        </div>
                    </div>
                    <div class="text-right flex items-center gap-2">
                         <div class="text-right mr-2">
                            <div class="font-bold text-emerald">${slot.count}/${totalPlayers}</div>
                            <div class="text-xs text-gray-500">${percentage}%</div>
                        </div>
                        <button onclick="availabilityManager.exportToICS('${slot.date}', '${slot.time}')" 
                                class="p-1.5 hover:bg-amber/10 text-amber rounded-full transition-colors"
                                title="Aggiungi al Calendario">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                                <line x1="16" y1="2" x2="16" y2="6"></line>
                                <line x1="8" y1="2" x2="8" y2="6"></line>
                                <line x1="3" y1="10" x2="21" y2="10"></line>
                                <line x1="11" y1="15" x2="13" y2="17"></line>
                                <line x1="11" y1="17" x2="13" y2="15"></line>
                            </svg>
                        </button>
                    </div>
                </div>
            `;
        }).join('');
    }

    exportToICS(dateStr, timeStr) {
        if (!this.selectedSession) return;

        const title = this.selectedSession.name || 'Sessione D&D';
        const description = `Sessione organizzata con D&D Scheduler.\nLink: ${window.location.href}`;

        // Parse date and time
        // dateStr is YYYY-MM-DD
        const [year, month, day] = dateStr.split('-').map(Number);
        const [hour, minute] = timeStr.split(':').map(Number);

        // Create start date object
        const startDate = new Date(year, month - 1, day, hour, minute);

        // Calculate end date (assume default duration based on session or fallback to 4 hours)
        let durationHours = 4;
        if (this.selectedSession.duration) {
            durationHours = parseInt(this.selectedSession.duration) || 4;
        }

        const endDate = new Date(startDate.getTime() + (durationHours * 60 * 60 * 1000));

        // Format dates for ICS (YYYYMMDDTHHMMSS)
        const formatICSDate = (date) => {
            return date.toISOString().replace(/[-:]/g, '').split('.')[0] + 'Z';
        };

        const start = formatICSDate(startDate);
        const end = formatICSDate(endDate);
        const now = formatICSDate(new Date());

        const icsContent = [
            'BEGIN:VCALENDAR',
            'VERSION:2.0',
            'PRODID:-//DND Scheduler//IT',
            'CALSCALE:GREGORIAN',
            'BEGIN:VEVENT',
            `DTSTAMP:${now}`,
            `DTSTART:${start}`,
            `DTEND:${end}`,
            `SUMMARY:${title}`,
            `DESCRIPTION:${description.replace(/\n/g, '\\n')}`,
            `URL:${window.location.href}`,
            'END:VEVENT',
            'END:VCALENDAR'
        ].join('\r\n'); // ICS requires CRLF

        // Create blob and download link
        const blob = new Blob([icsContent], { type: 'text/calendar;charset=utf-8' });
        const link = document.createElement('a');
        link.href = window.URL.createObjectURL(blob);
        link.setAttribute('download', `dnd_session_${dateStr}.ics`);
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    }

    updateSessionInfo() {
        if (!this.selectedSession) return;

        const organizer = window.DDSchedulerApp.getUserById(this.selectedSession.organizer);

        document.getElementById('campaign-title').textContent = this.selectedSession.title;
        document.getElementById('campaign-description').textContent = this.selectedSession.description || 'Nessuna descrizione';
        document.getElementById('dm-name').textContent = organizer ? organizer.name : 'Sconosciuto';
        document.getElementById('session-duration').textContent = this.formatDuration(this.selectedSession.duration || 180);
        document.getElementById('session-timezone').textContent = this.selectedSession.timezone || 'Non specificato';
        document.getElementById('response-deadline').textContent = this.selectedSession.settings?.deadline || 'Nessuna scadenza';
    }

    generateAvailabilityGrid() {
        const container = document.getElementById('availability-grid');
        if (!container || !this.selectedSession) return;

        console.log('Generating MagicalGrid for session:', this.selectedSession);

        // Parse dates
        let dates = [];
        try {
            const datesData = typeof this.selectedSession.dates === 'string'
                ? JSON.parse(this.selectedSession.dates)
                : this.selectedSession.dates;
            dates = Array.isArray(datesData) ? datesData : [];
        } catch (e) {
            dates = [];
        }

        // Parse time slots
        let timeSlots = [];
        try {
            const timeSlotsData = typeof this.selectedSession.time_range === 'string'
                ? JSON.parse(this.selectedSession.time_range)
                : this.selectedSession.time_range;
            timeSlots = Array.isArray(timeSlotsData) ? timeSlotsData : ['18:00', '19:00', '20:00', '21:00'];
        } catch (e) {
            timeSlots = ['18:00', '19:00', '20:00', '21:00'];
        }

        if (dates.length === 0) {
            container.innerHTML = '<div class="col-span-full text-center text-gray-500 py-8">Nessuna data disponibile per questa sessione</div>';
            return;
        }

        // Initialize MagicalGrid
        // Note: Drag interaction implemented via MagicalGrid component
        if (window.MagicalGrid) {
            this.param_grid = new window.MagicalGrid('availability-grid', {
                days: dates,
                timeSlots: timeSlots,
                onSlotChange: (day, time, state) => {
                    const cellId = `${day}_${time}`;
                    if (state === 'empty') {
                        delete this.availabilityData[cellId];
                    } else {
                        this.availabilityData[cellId] = state;
                    }
                    // Optional: Auto-save draft on change?
                    // this.saveAsDraft(); 
                }
            });

            // Re-apply existing availability
            Object.entries(this.availabilityData).forEach(([cellId, status]) => {
                const cell = document.getElementById(cellId);
                if (cell) {
                    if (status === 'available' || status === 'tentative') {
                        cell.classList.add(status);
                        cell.dataset.state = status;
                    }
                }
            });
        } else {
            console.error("MagicalGrid class not found on window");
        }
    }

    createGridCell(content, type, id = null) {
        const cell = document.createElement('div');
        // Map old types to new CSS classes
        let cssClass = 'grid-cell';
        if (type === 'header-cell') cssClass += ' grid-header-cell';
        if (type === 'time-label') cssClass += ' grid-time-label';
        if (type === 'clickable') cssClass += ' clickable';

        cell.className = cssClass;
        cell.textContent = content; // Default text content
        if (id) cell.id = id;
        return cell;
    }

    applyPaint(cell, cellId) {
        if (this.paintState === 'available') {
            this.availabilityData[cellId] = 'available';
            cell.classList.add('available');
            cell.classList.remove('busy');
        } else if (this.paintState === 'remove') {
            delete this.availabilityData[cellId];
            cell.classList.remove('available');
            cell.classList.remove('busy');
        }
    }

    toggleAvailability(cellId) {
        // Deprecated
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
                        <span>${user ? user.name : 'Giocatore Sconosciuto'}</span>
                    </div>
                    <span class="text-gray-500">${hasResponded ? 'Risposto' : 'In attesa'}</span>
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

        let dates = [];
        try {
            const datesData = typeof this.selectedSession.dates === 'string'
                ? JSON.parse(this.selectedSession.dates)
                : this.selectedSession.dates;
            if (Array.isArray(datesData)) {
                dates = datesData;
            }
        } catch (e) {
            dates = [];
        }

        const times = timeSlots.map(time => this.formatTime(time));

        const counts = timeSlots.map(time => {
            if (dates.length === 0) return 0;
            let totalAvailabilityForSlot = 0;
            dates.forEach(date => {
                const participantsAvailable = this.calculateOverlap(date, time);
                totalAvailabilityForSlot += participantsAvailable;
            });
            return Math.round(totalAvailabilityForSlot / dates.length);
        });

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
                max: this.selectedSession.participants.length || 1,
                interval: 1,
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
                    return `${params[0].name}<br/>Media Disponibili: ${params[0].value} giocatori`;
                }
            }
        };

        chart.setOption(option);

        window.addEventListener('resize', () => {
            chart.resize();
        });
    }

    calculateOverlap(date, time) {
        if (!this.selectedSession || !this.selectedSession.responses) return 0;

        let overlapCount = 0;
        Object.values(this.selectedSession.responses).forEach(response => {
            if (response.responded && response.availability) {
                const cellId = `${date}_${time}`;
                const status = response.availability[cellId];
                if (status === 'available' || status === 'tentative') {
                    overlapCount++;
                }
            }
        });

        return overlapCount;
    }

    calculateHeatmapData() {
        if (!this.selectedSession || !this.selectedSession.responses) return {};

        const heatmap = {};
        const responses = this.selectedSession.responses;

        // Initialize heatmap keys based on grid
        const dates = JSON.parse(this.selectedSession.dates || "[]");
        const times = JSON.parse(this.selectedSession.time_range || "[]");

        dates.forEach(date => {
            times.forEach(time => {
                heatmap[`${date}_${time}`] = { count: 0, voters: [] };
            });
        });

        // Iterate through all user responses
        Object.entries(responses).forEach(([userId, response]) => {
            if (!response.responded || !response.availability) return;
            const user = window.DDSchedulerApp.getUserById(userId);
            const userName = user ? user.name : 'Unknown';

            Object.entries(response.availability).forEach(([cellId, status]) => {
                if (status === 'available' || status === 'tentative') { // Count tentative? Maybe weight it? For now count as 1.
                    if (!heatmap[cellId]) {
                        // Handle potential schema mismatches or old data
                        heatmap[cellId] = { count: 0, voters: [] };
                    }
                    heatmap[cellId].count++;
                    heatmap[cellId].voters.push(userName);
                }
            });
        });

        return heatmap;
    }

    toggleView() {
        if (!this.param_grid) return;

        // Toggle state
        const currentMode = this.param_grid.config.mode;
        const newMode = currentMode === 'edit' ? 'heatmap' : 'edit';

        const btn = document.getElementById('toggle-view-btn');

        if (newMode === 'heatmap') {
            // Switch to Heatmap
            const heatmapData = this.calculateHeatmapData();
            this.param_grid.updateConfig({
                mode: 'heatmap',
                heatmapData: heatmapData
            });
            if (btn) {
                btn.textContent = 'Modifica Disponibilità';
                btn.classList.remove('border-amber', 'text-amber');
                btn.classList.add('bg-amber', 'text-dnd-black');
            }
            // Hide bulk actions in heatmap mode
            document.querySelector('.bulk-actions').style.display = 'none';
        } else {
            // Switch to Edit
            this.param_grid.updateConfig({
                mode: 'edit'
            });
            // Re-apply personal availability visual state
            Object.entries(this.availabilityData).forEach(([cellId, status]) => {
                const cell = document.getElementById(cellId);
                if (cell && (status === 'available' || status === 'tentative')) {
                    cell.classList.add(status);
                    cell.dataset.state = status;
                }
            });

            if (btn) {
                btn.textContent = 'Vista Gruppo';
                btn.classList.add('border-amber', 'text-amber');
                btn.classList.remove('bg-amber', 'text-dnd-black');
            }
            document.querySelector('.bulk-actions').style.display = 'flex';
        }
    }

    markAllAvailable() {
        document.querySelectorAll('.grid-cell.clickable').forEach(cell => {
            this.availabilityData[cell.id] = 'available';
            cell.className = 'grid-cell clickable available'; // Keep 'clickable' for selector
        });
    }

    markAllBusy() {
        document.querySelectorAll('.grid-cell.clickable, .grid-cell.available, .grid-cell.tentative').forEach(cell => {
            this.availabilityData[cell.id] = 'busy';
            cell.className = 'grid-cell busy';
            // Logic for 'busy' might be just removing 'available' if we simplify to binary state, 
            // but keeping 'busy' class for visual confirmation is fine.
            // However, my applyPaint uses binary state (available or nothing).
            // Let's align: LettuceMeet is usually binary (Available vs Busy).
            // If we want busy, we can keep the state. 
            // But my paint logic deletes the key for 'remove'.

            // Updating this to match paint logic:
            delete this.availabilityData[cell.id];
            cell.classList.remove('available');
            cell.classList.add('busy'); // Optional, if we want explicit red.
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
                cell.className = 'grid-cell clickable available';
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
                cell.className = 'grid-cell clickable available';
            }
        });
    }

    resetAvailability() {
        this.availabilityData = {};
        this.generateAvailabilityGrid();
    }

    saveAsDraft() {
        localStorage.setItem(`availability_draft_${this.selectedSession.id}`, JSON.stringify(this.availabilityData));
        this.showNotification('Bozza Salvata', 'La tua disponibilità è stata salvata come bozza.');
    }

    async submitAvailability() {
        if (Object.keys(this.availabilityData).length === 0) {
            this.showNotification('Nessuna disponibilità segnata', 'Per favore segna la tua disponibilità prima di inviare.', 'error');
            return;
        }

        try {
            if (!this.selectedSession) throw new Error('Nessuna sessione selezionata');
            if (!this.currentUser?.id) {
                this.promptUserIdentification();
                throw new Error('Utente non identificato');
            }

            if (!this.currentUser?.accessToken) {
                this.showNotification('Errore Autorizzazione', 'Token mancante. Per favore unisciti di nuovo alla sessione.', 'error');
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

            const participantId = this.currentUser.participantId || this.currentUser.id;

            const response = await fetch(`/api/polls/${this.selectedSession.id}/participants/${participantId}/availability`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.currentUser.accessToken}`
                },
                body: JSON.stringify({
                    availability: availabilityList,
                    access_token: this.currentUser.accessToken
                })
            });

            if (!response.ok) {
                const errorData = await response.text();
                throw new Error(errorData || 'Impossibile inviare la disponibilità');
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
            this.showNotification('Invio Fallito', error.message || 'Impossibile inviare la disponibilità. Riprova.', 'error');
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
                <h3 class="font-cinzel text-2xl font-bold text-forest mb-2">Disponibilità Inviata!</h3>
                <p class="text-gray-600 mb-4">Grazie per aver segnato la tua disponibilità. Il DM verrà notificato.</p>
                <p class="text-sm text-gray-500">Reindirizzamento alla bacheca...</p>
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

    formatDuration(minutes) {
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        return mins > 0 ? `${hours}h ${mins}m` : `${hours} ore`;
    }

    formatDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('it-IT', {
            weekday: 'short',
            month: 'short',
            day: 'numeric'
        });
    }

    formatTime(timeString) {
        const [hours, minutes] = timeString.split(':');
        // Simple 24h format for Italy
        return `${hours}:${minutes}`;
    }
}

// Initialize availability manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.availabilityManager = new AvailabilityManager();
});