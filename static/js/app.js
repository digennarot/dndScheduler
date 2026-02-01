// Main Application Controller
class DDSchedulerApp {
    constructor() {
        this.polls = [];
        this.users = []; // Will be populated from API/mock data where needed, but Auth is separate

        // Integrate with AuthManager
        if (window.authManager) {
            this.currentUser = window.authManager.getCurrentUser();
            // Listen for auth changes if needed, but simple reference is enough for now
        } else {
            console.warn('AuthManager not found, falling back to guest mode');
            this.currentUser = null;
        }

        this.init();
    }

    async init() {
        await this.fetchPolls();
        this.setupEventListeners();
        this.initializeAnimations();
    }

    async fetchPolls() {
        try {
            const response = await fetch('/api/polls');
            if (!response.ok) throw new Error('Failed to fetch polls');
            const basicPolls = await response.json();

            this.polls = await Promise.all(basicPolls.map(async (basicPoll) => {
                const detailResponse = await fetch(`/api/polls/${basicPoll.id}`);
                const data = await detailResponse.json();

                // Transform to frontend model
                // Transform to frontend model
                return this.transformPollData(data);
            }));

            console.log('Polls loaded:', this.polls);
            document.dispatchEvent(new CustomEvent('pollsLoaded'));
        } catch (error) {
            console.error('Failed to fetch polls:', error);
        }
    }

    setupEventListeners() {
        // Navigation mobile toggle
        const mobileMenuBtn = document.getElementById('mobile-menu-btn');
        if (mobileMenuBtn) {
            mobileMenuBtn.addEventListener('click', this.toggleMobileMenu);
        }

        // Smooth scrolling for anchor links
        document.querySelectorAll('a[href^="#"]').forEach(anchor => {
            anchor.addEventListener('click', function (e) {
                e.preventDefault();
                const target = document.querySelector(this.getAttribute('href'));
                if (target) {
                    target.scrollIntoView({ behavior: 'smooth' });
                }
            });
        });
    }

    initializeAnimations() {
        // Animate hero text on load
        anime({
            targets: '.hero-text',
            opacity: [0, 1],
            translateY: [30, 0],
            duration: 1000,
            easing: 'easeOutQuart',
            delay: 300
        });

        // Animate cards on scroll
        const observerOptions = {
            threshold: 0.1,
            rootMargin: '0px 0px -50px 0px'
        };

        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    anime({
                        targets: entry.target,
                        opacity: [0, 1],
                        translateY: [20, 0],
                        duration: 600,
                        easing: 'easeOutQuart'
                    });
                }
            });
        }, observerOptions);

        // Observe all cards
        document.querySelectorAll('.card-hover').forEach(card => {
            observer.observe(card);
        });
    }

    toggleMobileMenu() {
        const mobileMenu = document.getElementById('mobile-menu');
        if (mobileMenu) {
            mobileMenu.classList.toggle('hidden');
        }
    }

    getUserById(userId) {
        return this.users.find(user => user.id === userId);
    }

    getPollById(pollId) {
        return this.polls.find(poll => poll.id === pollId);
    }

    calculateResponseRate(poll) {
        const totalParticipants = poll.participants.length;
        const respondedParticipants = Object.keys(poll.responses).filter(userId =>
            poll.responses[userId].responded
        ).length;

        return Math.round((respondedParticipants / totalParticipants) * 100);
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
        const [date, time] = timeString.split(' ');
        const [hours, minutes] = time.split(':');
        // Use 24h format for Italian
        return `${hours}:${minutes}`;
    }

    escapeHtml(unsafe) {
        if (!unsafe) return '';
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }

    async fetchSinglePoll(pollId) {
        try {
            const tokenKey = `dnd_poll_admin_${pollId}`;
            const token = localStorage.getItem(tokenKey);

            const headers = {};
            if (token) {
                headers['Authorization'] = `Bearer ${token}`;
            }

            const response = await fetch(`/api/polls/${pollId}`, { headers });
            if (!response.ok) return null;

            const data = await response.json();
            const poll = this.transformPollData(data);

            // Add/Update in this.polls
            const existingIndex = this.polls.findIndex(p => p.id === poll.id);
            if (existingIndex >= 0) {
                this.polls[existingIndex] = poll;
            } else {
                this.polls.push(poll);
            }
            return poll;
        } catch (e) {
            console.error('Error fetching single poll', e);
            return null;
        }
    }

    transformPollData(data) {
        const poll = data.poll;
        let parsedDates = [];
        try {
            parsedDates = JSON.parse(poll.dates);
        } catch (e) {
            parsedDates = [];
        }

        // Handle both array format and object format
        if (Array.isArray(parsedDates)) {
            poll.datesList = parsedDates;
            poll.dateRange = {
                start: parsedDates[0],
                end: parsedDates[parsedDates.length - 1]
            };
        } else {
            poll.dateRange = parsedDates;
            poll.datesList = [];
        }

        poll.timeSlots = poll.time_range ? JSON.parse(poll.time_range) : [];

        poll.participants = data.participants.map(p => ({
            id: p.id,
            name: p.name,
            email: p.email
        }));

        data.participants.forEach(p => {
            if (!this.users.find(u => u.id === p.id)) {
                this.users.push({
                    id: p.id,
                    name: p.name,
                    role: 'participant',
                    avatar: p.name.substring(0, 2).toUpperCase()
                });
            }
        });

        poll.responses = {};
        data.participants.forEach(p => {
            const userAvailability = data.availability.filter(a => a.participant_id === p.id);
            if (userAvailability.length > 0) {
                const availabilityMap = {};
                let availableCount = 0;
                userAvailability.forEach(a => {
                    availabilityMap[`${a.date}_${a.time_slot}`] = a.status;
                    if (a.status === 'available') availableCount++;
                });

                const totalSlots = (poll.datesList.length || 1) * (poll.timeSlots.length || 1);

                poll.responses[p.id] = {
                    responded: true,
                    availability: availabilityMap,
                    availabilityScore: totalSlots > 0 ? Math.round((availableCount / totalSlots) * 100) : 0
                };
            } else {
                poll.responses[p.id] = { responded: false };
            }
        });

        return poll;
    }
}

// Initialize the application
const app = new DDSchedulerApp();

// Export for use in other modules
window.DDSchedulerApp = app;