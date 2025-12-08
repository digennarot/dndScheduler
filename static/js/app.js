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
                const poll = data.poll;
                const parsedDates = JSON.parse(poll.dates);
                // Handle both array format [start, end] and object format {start, end} if legacy
                if (Array.isArray(parsedDates)) {
                    poll.dateRange = { start: parsedDates[0], end: parsedDates[1] };
                } else {
                    poll.dateRange = parsedDates;
                }
                poll.timeSlots = poll.time_range ? JSON.parse(poll.time_range) : []; // stored as JSON

                // Map participants to full objects to allow filtering by email/id
                poll.participants = data.participants.map(p => ({
                    id: p.id,
                    name: p.name,
                    email: p.email // Ensure this is available from backend if filtered
                }));

                // Add participants to users list if not present (to support getUserById)
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

                // Map responses
                poll.responses = {};
                data.participants.forEach(p => {
                    // Find availability for this participant
                    const userAvailability = data.availability.filter(a => a.participant_id === p.id);
                    if (userAvailability.length > 0) {
                        const availabilityMap = {};
                        let availableCount = 0;
                        userAvailability.forEach(a => {
                            availabilityMap[`${a.date}_${a.time_slot}`] = a.status;
                            if (a.status === 'available') availableCount++;
                        });

                        poll.responses[p.id] = {
                            responded: true,
                            availability: availabilityMap,
                            // Calculate simple percentage for the UI
                            availabilityScore: Math.round((availableCount / (poll.timeSlots.length * poll.dateRange.length || 1)) * 100)
                        };
                    } else {
                        poll.responses[p.id] = { responded: false };
                    }
                });

                return poll;
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
        return date.toLocaleDateString('en-US', {
            weekday: 'short',
            month: 'short',
            day: 'numeric'
        });
    }

    formatTime(timeString) {
        const [date, time] = timeString.split(' ');
        const [hours, minutes] = time.split(':');
        const hour12 = hours > 12 ? hours - 12 : hours;
        const ampm = hours >= 12 ? 'PM' : 'AM';
        return `${hour12}:${minutes} ${ampm}`;
    }
}

// Initialize the application
const app = new DDSchedulerApp();

// Export for use in other modules
window.DDSchedulerApp = app;