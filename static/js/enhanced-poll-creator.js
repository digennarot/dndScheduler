/**
 * Enhanced Poll Creator with Per-Day Time Preferences
 * Allows users to select different time slots for each selected date
 */

class EnhancedPollCreator {
    constructor() {
        this.currentStep = 1;
        this.selectedDates = [];
        this.dateMode = 'day';
        this.formData = {};
        // New: per-day time preferences
        this.timePreferences = {}; // { "2025-12-10": ["18:00", "19:00"], ... }

        this.init();
    }

    init() {
        this.setupFlatpickr();
        this.generateTimeSlots();
        this.setupFormSubmission();
        this.setupParticles();
    }

    setupFlatpickr() {
        const dateInput = document.getElementById('date-range');
        if (!dateInput) return;

        this.flatpickrInstance = flatpickr(dateInput, {
            mode: 'multiple',
            dateFormat: 'Y-m-d',
            minDate: 'today',
            onChange: (selectedDates, dateStr, instance) => {
                this.selectedDates = selectedDates.map(d => {
                    const year = d.getFullYear();
                    const month = String(d.getMonth() + 1).padStart(2, '0');
                    const day = String(d.getDate()).padStart(2, '0');
                    return `${year}-${month}-${day}`;
                });
                console.log('Selected dates:', this.selectedDates);
            }
        });
    }

    updateDateMode(mode) {
        this.dateMode = mode;
        const dateInput = document.getElementById('date-range');

        if (mode === 'week') {
            this.flatpickrInstance.set('mode', 'range');
            dateInput.placeholder = 'Select a week...';
        } else if (mode === 'month') {
            this.flatpickrInstance.set('mode', 'range');
            dateInput.placeholder = 'Select a month...';
        } else {
            this.flatpickrInstance.set('mode', 'multiple');
            dateInput.placeholder = 'Select specific dates...';
        }
    }

    /**
     * Generate time slots for selection
     * This creates the initial time slot buttons
     */
    generateTimeSlots() {
        const container = document.getElementById('time-slots');
        if (!container) return;

        const timeSlots = [
            '09:00', '10:00', '11:00', '12:00', '13:00', '14:00',
            '15:00', '16:00', '17:00', '18:00', '19:00', '20:00',
            '21:00', '22:00', '23:00'
        ];

        container.innerHTML = timeSlots.map(time => {
            const displayTime = this.formatTime(time);
            return `
                <div class="time-slot" data-time="${time}" onclick="enhancedPollCreator.toggleTimeSlot('${time}')">
                    ${displayTime}
                </div>
            `;
        }).join('');
    }

    /**
     * NEW: Render per-day time selection interface
     * This is called when entering Step 3
     */
    renderPerDayTimeSelection() {
        const container = document.getElementById('time-slots');
        if (!container) return;

        if (this.selectedDates.length === 0) {
            container.innerHTML = `
                <div class="col-span-full text-center py-8 text-gray-500">
                    <p class="text-lg mb-2">📅 No dates selected yet</p>
                    <p class="text-sm">Please go back to Step 2 and select your available dates first.</p>
                </div>
            `;
            return;
        }

        // Initialize time preferences for new dates
        this.selectedDates.forEach(date => {
            if (!this.timePreferences[date]) {
                this.timePreferences[date] = [];
            }
        });

        // Remove preferences for dates that are no longer selected
        Object.keys(this.timePreferences).forEach(date => {
            if (!this.selectedDates.includes(date)) {
                delete this.timePreferences[date];
            }
        });

        const timeSlots = [
            '09:00', '10:00', '11:00', '12:00', '13:00', '14:00',
            '15:00', '16:00', '17:00', '18:00', '19:00', '20:00',
            '21:00', '22:00', '23:00'
        ];

        container.innerHTML = `
            <div class="col-span-full mb-6">
                <div class="bg-amber/10 p-4 rounded-lg border border-amber/30">
                    <div class="flex items-center justify-between">
                        <div>
                            <h4 class="font-semibold text-forest mb-1">⚡ Quick Actions</h4>
                            <p class="text-sm text-gray-600">Select times for one date, then apply to all</p>
                        </div>
                        <button type="button" onclick="enhancedPollCreator.applyToAllDates()" 
                            class="px-4 py-2 bg-amber text-white rounded-lg hover:bg-amber/90 transition-colors text-sm font-medium">
                            Copy First Date to All
                        </button>
                    </div>
                </div>
            </div>

            ${this.selectedDates.map((date, index) => {
            const dateObj = new Date(date + 'T00:00:00');
            const dayName = dateObj.toLocaleDateString('en-US', { weekday: 'long' });
            const formattedDate = dateObj.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
            const selectedTimes = this.timePreferences[date] || [];

            return `
                    <div class="col-span-full mb-8 border-b border-gray-200 pb-6 last:border-0">
                        <div class="flex items-center justify-between mb-4">
                            <div>
                                <h4 class="font-cinzel text-lg font-semibold text-forest">${dayName}</h4>
                                <p class="text-sm text-gray-600">${formattedDate}</p>
                            </div>
                            <div class="text-sm text-gray-500">
                                ${selectedTimes.length} time${selectedTimes.length !== 1 ? 's' : ''} selected
                            </div>
                        </div>
                        <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-2">
                            ${timeSlots.map(time => {
                const isSelected = selectedTimes.includes(time);
                const displayTime = this.formatTime(time);
                return `
                                    <div class="time-slot ${isSelected ? 'selected' : ''}" 
                                        data-date="${date}" 
                                        data-time="${time}" 
                                        onclick="enhancedPollCreator.togglePerDayTimeSlot('${date}', '${time}')">
                                        ${displayTime}
                                    </div>
                                `;
            }).join('')}
                        </div>
                    </div>
                `;
        }).join('')}
        `;
    }

    /**
     * Toggle time slot for a specific date
     */
    togglePerDayTimeSlot(date, time) {
        if (!this.timePreferences[date]) {
            this.timePreferences[date] = [];
        }

        const index = this.timePreferences[date].indexOf(time);
        if (index > -1) {
            // Remove time
            this.timePreferences[date].splice(index, 1);
        } else {
            // Add time
            this.timePreferences[date].push(time);
        }

        // Re-render to update UI
        this.renderPerDayTimeSelection();
    }

    /**
     * Apply the first date's time selections to all dates
     */
    applyToAllDates() {
        if (this.selectedDates.length === 0) return;

        const firstDate = this.selectedDates[0];
        const firstDateTimes = this.timePreferences[firstDate] || [];

        if (firstDateTimes.length === 0) {
            alert('Please select at least one time slot for the first date before copying to all dates.');
            return;
        }

        // Copy to all dates
        this.selectedDates.forEach(date => {
            this.timePreferences[date] = [...firstDateTimes];
        });

        // Re-render to show changes
        this.renderPerDayTimeSelection();

        // Show confirmation
        this.showNotification(`Applied ${firstDateTimes.length} time slot(s) to all ${this.selectedDates.length} dates!`);
    }

    /**
     * Legacy method for backward compatibility
     * (kept for old time slot interface if needed)
     */
    toggleTimeSlot(time) {
        const element = document.querySelector(`.time-slot[data-time="${time}"]`);
        if (!element) return;

        element.classList.toggle('selected');
    }

    formatTime(time) {
        const [hours, minutes] = time.split(':');
        const hour = parseInt(hours);
        const ampm = hour >= 12 ? 'PM' : 'AM';
        const displayHour = hour > 12 ? hour - 12 : (hour === 0 ? 12 : hour);
        return `${displayHour}:${minutes} ${ampm}`;
    }

    nextStep() {
        if (!this.validateStep(this.currentStep)) {
            return;
        }

        // Special handling for Step 2 -> Step 3 transition
        if (this.currentStep === 2) {
            // Render the per-day time selection interface
            setTimeout(() => {
                this.renderPerDayTimeSelection();
            }, 100);
        }

        if (this.currentStep < 5) {
            this.goToStep(this.currentStep + 1);
        }
    }

    previousStep() {
        if (this.currentStep > 1) {
            this.goToStep(this.currentStep - 1);
        }
    }

    goToStep(step) {
        // Hide current step
        document.getElementById(`step-${this.currentStep}`)?.classList.remove('active');
        document.getElementById(`step-${this.currentStep}-indicator`)?.classList.remove('step-active');
        document.getElementById(`step-${this.currentStep}-indicator`)?.classList.add('step-completed');

        // Show new step
        this.currentStep = step;
        document.getElementById(`step-${step}`)?.classList.add('active');
        document.getElementById(`step-${step}-indicator`)?.classList.remove('step-inactive', 'step-completed');
        document.getElementById(`step-${step}-indicator`)?.classList.add('step-active');

        // Update navigation buttons
        const prevBtn = document.getElementById('prev-btn');
        const nextBtn = document.getElementById('next-btn');

        if (prevBtn) {
            prevBtn.style.display = step === 1 ? 'none' : 'block';
        }

        if (nextBtn) {
            nextBtn.style.display = step === 5 ? 'none' : 'block';
        }

        // Special handling for Step 3
        if (step === 3) {
            this.renderPerDayTimeSelection();
        }

        // Update review content if on step 5
        if (step === 5) {
            this.updateReviewContent();
        }

        // Scroll to top
        window.scrollTo({ top: 0, behavior: 'smooth' });
    }

    validateStep(step) {
        switch (step) {
            case 1:
                const title = document.getElementById('campaign-title')?.value;
                const duration = document.getElementById('session-duration')?.value;
                if (!title || !duration) {
                    alert('Please fill in all required fields (Campaign Name and Session Duration)');
                    return false;
                }
                return true;

            case 2:
                if (this.selectedDates.length === 0) {
                    alert('Please select at least one date for your campaign');
                    return false;
                }
                return true;

            case 3:
                // Check if at least one date has time slots selected
                const hasAnyTimes = Object.values(this.timePreferences).some(times => times.length > 0);
                if (!hasAnyTimes) {
                    alert('Please select at least one time slot for at least one date');
                    return false;
                }

                const timezone = document.getElementById('timezone')?.value;
                if (!timezone) {
                    alert('Please select your timezone');
                    return false;
                }
                return true;

            case 4:
                const emails = document.getElementById('player-emails')?.value;
                if (!emails || emails.trim().length === 0) {
                    alert('Please enter at least one player email address');
                    return false;
                }
                return true;

            default:
                return true;
        }
    }

    updateReviewContent() {
        const container = document.getElementById('review-content');
        if (!container) return;

        const title = document.getElementById('campaign-title')?.value || '';
        const description = document.getElementById('campaign-description')?.value || '';
        const duration = document.getElementById('session-duration')?.value || '';
        const type = document.getElementById('campaign-type')?.value || '';
        const timezone = document.getElementById('timezone')?.value || '';
        const emails = document.getElementById('player-emails')?.value || '';
        const emailList = emails.split(/[,\n]/).map(e => e.trim()).filter(e => e);

        // Format dates with their time preferences
        const datesWithTimes = this.selectedDates.map(date => {
            const dateObj = new Date(date + 'T00:00:00');
            const dayName = dateObj.toLocaleDateString('en-US', { weekday: 'short' });
            const formattedDate = dateObj.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
            const times = this.timePreferences[date] || [];
            const timesFormatted = times.map(t => this.formatTime(t)).join(', ');

            return `
                <div class="flex justify-between items-start py-2 border-b border-gray-100 last:border-0">
                    <span class="font-medium text-forest">${dayName}, ${formattedDate}</span>
                    <span class="text-sm text-gray-600">${timesFormatted || 'No times selected'}</span>
                </div>
            `;
        }).join('');

        container.innerHTML = `
            <div class="space-y-6">
                <div>
                    <h4 class="font-semibold text-gray-700 mb-2">Campaign Details</h4>
                    <div class="bg-gray-50 p-4 rounded-lg space-y-2">
                        <p><span class="font-medium">Title:</span> ${title}</p>
                        <p><span class="font-medium">Description:</span> ${description || 'None provided'}</p>
                        <p><span class="font-medium">Duration:</span> ${duration} minutes</p>
                        <p><span class="font-medium">Type:</span> ${type}</p>
                    </div>
                </div>

                <div>
                    <h4 class="font-semibold text-gray-700 mb-2">Dates & Times (${this.selectedDates.length} dates)</h4>
                    <div class="bg-gray-50 p-4 rounded-lg">
                        ${datesWithTimes}
                    </div>
                </div>

                <div>
                    <h4 class="font-semibold text-gray-700 mb-2">Time Settings</h4>
                    <div class="bg-gray-50 p-4 rounded-lg">
                        <p><span class="font-medium">Timezone:</span> ${timezone}</p>
                    </div>
                </div>

                <div>
                    <h4 class="font-semibold text-gray-700 mb-2">Invited Players (${emailList.length})</h4>
                    <div class="bg-gray-50 p-4 rounded-lg">
                        <div class="flex flex-wrap gap-2">
                            ${emailList.map(email => `
                                <span class="participant-chip">${email}</span>
                            `).join('')}
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    setupFormSubmission() {
        const form = document.getElementById('poll-creation-form');
        if (!form) return;

        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.submitPoll();
        });
    }

    async submitPoll() {
        try {
            const title = document.getElementById('campaign-title')?.value || '';
            const description = document.getElementById('campaign-description')?.value || '';
            const emails = document.getElementById('player-emails')?.value || '';
            const emailList = emails.split(/[,\n]/).map(e => e.trim()).filter(e => e);

            const payload = {
                title: title,
                description: description,
                location: "Online", // Default for now
                dates: this.selectedDates,
                timePreferences: this.timePreferences, // NEW: per-day time preferences
                participants: emailList
            };

            console.log('Submitting poll:', payload);

            const response = await fetch('/api/polls', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(payload)
            });

            if (!response.ok) {
                // Try to parse JSON error first
                let errorMessage = 'Failed to create poll';
                try {
                    const errorJson = await response.json();
                    errorMessage = errorJson.error || errorMessage;
                } catch (e) {
                    errorMessage = await response.text() || errorMessage;
                }
                throw new Error(errorMessage);
            }

            const result = await response.json();
            console.log('Poll created:', result);

            // Story 1.5: Handle adminToken for anonymous management
            if (result.adminToken) {
                try {
                    const tokenKey = `dnd_poll_admin_${result.id}`;
                    localStorage.setItem(tokenKey, result.adminToken);
                    console.log('Admin token saved');
                } catch (e) {
                    console.warn('Failed to save admin token', e);
                }
            }

            this.showNotification('Campaign created! Redirecting to setup...', 'success');

            // Redirect to management page
            setTimeout(() => {
                window.location.href = `manage.html?poll=${result.id}&new=true`;
            }, 1000);

        } catch (error) {
            console.error('Error creating poll:', error);
            alert('Failed to create poll: ' + error.message);
        }
    }

    showNotification(message, type = 'info') {
        // Create a simple notification
        const notification = document.createElement('div');
        notification.className = `fixed top-4 right-4 z-50 px-6 py-3 rounded-lg shadow-lg text-white ${type === 'success' ? 'bg-emerald' : 'bg-amber'
            }`;
        notification.textContent = message;
        document.body.appendChild(notification);

        setTimeout(() => {
            notification.remove();
        }, 3000);
    }

    setupParticles() {
        // Particle animation setup (if needed)
        // This would be the same as the original implementation
    }
}

// Initialize the enhanced poll creator
let enhancedPollCreator;
document.addEventListener('DOMContentLoaded', () => {
    enhancedPollCreator = new EnhancedPollCreator();
});
