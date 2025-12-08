// Poll Creation Wizard
class PollCreator {
    constructor() {
        this.currentStep = 1;
        this.totalSteps = 5;
        this.formData = {};
        this.selectedTimeSlots = [];
        this.init();
    }

    init() {
        this.generateTimeSlots();
        this.initializeDatePicker();
        this.setupFormValidation();
        this.updateStepIndicators();
    }

    generateTimeSlots() {
        const container = document.getElementById('time-slots');
        if (!container) return;

        const timeSlots = [
            '16:00', '17:00', '18:00', '19:00', '20:00', '21:00',
            '22:00', '23:00', '10:00', '11:00', '12:00', '13:00',
            '14:00', '15:00'
        ];

        container.innerHTML = timeSlots.map(time => {
            const displayTime = this.formatTime(time);
            return `
                <div class="time-slot" data-time="${time}" onclick="pollCreator.toggleTimeSlot('${time}')">
                    ${displayTime}
                </div>
            `;
        }).join('');
    }

    formatTime(time) {
        const [hours, minutes] = time.split(':');
        const hour12 = hours > 12 ? hours - 12 : hours;
        const ampm = hours >= 12 ? 'PM' : 'AM';
        return `${hour12}:00 ${ampm}`;
    }

    initializeDatePicker() {
        const dateRangeInput = document.getElementById('date-range');
        if (!dateRangeInput) return;

        // Destroy existing instance if it exists
        if (this.fp) {
            this.fp.destroy();
        }

        // Default mode if not set
        if (!this.dateMode) this.dateMode = 'day';

        const config = {
            mode: this.dateMode === 'day' ? 'multiple' : 'range',
            dateFormat: 'Y-m-d',
            minDate: 'today',
            maxDate: new Date().fp_incr(90), // 90 days from now
            onChange: (selectedDates, dateStr, instance) => {
                if (selectedDates.length === 0) return;

                if (this.dateMode === 'day') {
                    // Multiple individual dates (LettuceMeet style)
                    // Store as array of date strings
                    this.formData.dateRange = {
                        dates: selectedDates.map(date => this.formatDateStr(date)),
                        mode: 'multiple'
                    };
                } else {
                    // Week or Month mode - calculate range
                    const date = selectedDates[0];
                    let start, end;

                    if (this.dateMode === 'week') {
                        // Calculate Monday and Sunday of the selected week
                        const day = date.getDay(); // 0 is Sun, 1 is Mon
                        const diffToMon = date.getDate() - day + (day === 0 ? -6 : 1);

                        start = new Date(date);
                        start.setDate(diffToMon);

                        end = new Date(start);
                        end.setDate(start.getDate() + 6);
                    } else if (this.dateMode === 'month') {
                        // First and last day of month
                        start = new Date(date.getFullYear(), date.getMonth(), 1);
                        end = new Date(date.getFullYear(), date.getMonth() + 1, 0);
                    }

                    // Update form data with range
                    this.formData.dateRange = {
                        start: this.formatDateStr(start),
                        end: this.formatDateStr(end),
                        mode: 'range'
                    };

                    // Update calendar selection visually
                    const currentStartStr = selectedDates[0].toDateString();
                    const currentEndStr = selectedDates[selectedDates.length - 1].toDateString();
                    const newStartStr = start.toDateString();
                    const newEndStr = end.toDateString();

                    if (currentStartStr !== newStartStr || currentEndStr !== newEndStr || selectedDates.length === 1) {
                        instance.setDate([start, end], false); // false = do not trigger onChange
                        // Manually update input value since we suppressed the event
                        instance.input.value = `${this.formatDateStr(start)} to ${this.formatDateStr(end)}`;
                    }
                }
            }
        };

        this.fp = flatpickr(dateRangeInput, config);
    }

    formatDateStr(date) {
        return date.toISOString().split('T')[0];
    }

    updateDateMode(mode) {
        this.dateMode = mode;
        this.initializeDatePicker();
    }

    setupFormValidation() {
        const form = document.getElementById('poll-creation-form');
        if (form) {
            form.addEventListener('submit', (e) => {
                e.preventDefault();
                this.handleFormSubmission();
            });
        }
    }

    toggleTimeSlot(time) {
        const element = document.querySelector(`[data-time="${time}"]`);
        if (!element) return;

        if (this.selectedTimeSlots.includes(time)) {
            this.selectedTimeSlots = this.selectedTimeSlots.filter(t => t !== time);
            element.classList.remove('selected');
        } else {
            this.selectedTimeSlots.push(time);
            element.classList.add('selected');
        }

        this.updateTimeSlotValidation();
    }

    updateTimeSlotValidation() {
        const nextBtn = document.getElementById('next-btn');
        if (this.selectedTimeSlots.length === 0) {
            nextBtn.disabled = true;
            nextBtn.classList.add('opacity-50', 'cursor-not-allowed');
        } else {
            nextBtn.disabled = false;
            nextBtn.classList.remove('opacity-50', 'cursor-not-allowed');
        }
    }

    nextStep() {
        if (this.validateCurrentStep()) {
            this.saveCurrentStepData();

            if (this.currentStep < this.totalSteps) {
                this.currentStep++;
                this.updateStepDisplay();
                this.updateStepIndicators();
                this.updateNavigationButtons();
            }
        }
    }

    previousStep() {
        if (this.currentStep > 1) {
            this.currentStep--;
            this.updateStepDisplay();
            this.updateStepIndicators();
            this.updateNavigationButtons();
        }
    }

    goToStep(step) {
        if (step >= 1 && step <= this.totalSteps) {
            this.currentStep = step;
            this.updateStepDisplay();
            this.updateStepIndicators();
            this.updateNavigationButtons();
        }
    }

    validateCurrentStep() {
        switch (this.currentStep) {
            case 1:
                return this.validateStep1();
            case 2:
                return this.validateStep2();
            case 3:
                return this.validateStep3();
            case 4:
                return this.validateStep4();
            default:
                return true;
        }
    }

    validateStep1() {
        const title = document.getElementById('campaign-title').value.trim();
        const duration = document.getElementById('session-duration').value;

        if (!title) {
            this.showValidationError('Campaign title is required');
            return false;
        }

        if (!duration) {
            this.showValidationError('Session duration is required');
            return false;
        }

        return true;
    }

    validateStep2() {
        if (!this.formData.dateRange) {
            this.showValidationError('Please select dates');
            return false;
        }

        // Check if we have either multiple dates or a range
        const hasMultipleDates = this.formData.dateRange.dates && this.formData.dateRange.dates.length > 0;
        const hasRange = this.formData.dateRange.start && this.formData.dateRange.end;

        if (!hasMultipleDates && !hasRange) {
            this.showValidationError('Please select at least one date');
            return false;
        }

        return true;
    }

    validateStep3() {
        if (this.selectedTimeSlots.length === 0) {
            this.showValidationError('Please select at least one preferred time slot');
            return false;
        }

        const timezone = document.getElementById('timezone').value;
        if (!timezone) {
            this.showValidationError('Please select your timezone');
            return false;
        }

        return true;
    }

    validateStep4() {
        const emails = document.getElementById('player-emails').value.trim();
        if (!emails) {
            this.showValidationError('Please enter at least one player email address');
            return false;
        }

        // Basic email validation
        const emailList = emails.split(/[,\n]/).map(email => email.trim()).filter(email => email);
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

        for (const email of emailList) {
            if (!emailRegex.test(email)) {
                this.showValidationError(`Invalid email address: ${email}`);
                return false;
            }
        }

        if (emailList.length > 10) {
            this.showValidationError('Maximum 10 players allowed per poll');
            return false;
        }

        return true;
    }

    saveCurrentStepData() {
        switch (this.currentStep) {
            case 1:
                this.formData.title = document.getElementById('campaign-title').value.trim();
                this.formData.description = document.getElementById('campaign-description').value.trim();
                this.formData.duration = parseInt(document.getElementById('session-duration').value);
                this.formData.type = document.getElementById('campaign-type').value;
                break;
            case 2:
                // Date range is already saved in formData.dateRange
                break;
            case 3:
                this.formData.timeSlots = [...this.selectedTimeSlots];
                this.formData.timezone = document.getElementById('timezone').value;
                this.formData.recurringPattern = document.getElementById('recurring-pattern').value;
                break;
            case 4:
                const emails = document.getElementById('player-emails').value.trim();
                this.formData.emails = emails.split(/[,\n]/).map(email => email.trim()).filter(email => email);
                this.formData.deadline = document.getElementById('response-deadline').value;
                this.formData.privacy = document.getElementById('privacy-setting').value;
                break;
        }
    }

    updateStepDisplay() {
        // Hide all steps
        document.querySelectorAll('.form-step').forEach(step => {
            step.classList.remove('active');
        });

        // Show current step
        const currentStepElement = document.getElementById(`step-${this.currentStep}`);
        if (currentStepElement) {
            currentStepElement.classList.add('active');
        }

        // Special handling for review step
        if (this.currentStep === 5) {
            this.populateReviewStep();
        }

        // Scroll to top of form
        window.scrollTo({ top: 0, behavior: 'smooth' });
    }

    updateStepIndicators() {
        for (let i = 1; i <= this.totalSteps; i++) {
            const indicator = document.getElementById(`step-${i}-indicator`);
            if (indicator) {
                indicator.classList.remove('step-active', 'step-completed', 'step-inactive');

                if (i < this.currentStep) {
                    indicator.classList.add('step-completed');
                } else if (i === this.currentStep) {
                    indicator.classList.add('step-active');
                } else {
                    indicator.classList.add('step-inactive');
                }
            }
        }
    }

    updateNavigationButtons() {
        const prevBtn = document.getElementById('prev-btn');
        const nextBtn = document.getElementById('next-btn');

        // Show/hide previous button
        if (this.currentStep > 1) {
            prevBtn.style.display = 'block';
        } else {
            prevBtn.style.display = 'none';
        }

        // Update next button text and behavior
        if (this.currentStep === this.totalSteps) {
            nextBtn.style.display = 'none';
        } else {
            nextBtn.style.display = 'block';
            nextBtn.textContent = this.currentStep === this.totalSteps - 1 ? 'Review →' : 'Next →';
        }
    }

    populateReviewStep() {
        const container = document.getElementById('review-content');
        if (!container) return;

        const formatTimeSlots = (slots) => {
            return slots.map(slot => this.formatTime(slot)).join(', ');
        };

        const formatDuration = (minutes) => {
            const hours = Math.floor(minutes / 60);
            const mins = minutes % 60;
            return mins > 0 ? `${hours}h ${mins}m` : `${hours} hours`;
        };

        container.innerHTML = `
            <div class="bg-gray-50 p-6 rounded-lg space-y-4">
                <div class="grid md:grid-cols-2 gap-6">
                    <div>
                        <h4 class="font-semibold text-forest mb-2">Campaign Details</h4>
                        <div class="space-y-2 text-sm">
                            <div><span class="font-medium">Title:</span> ${this.formData.title}</div>
                            <div><span class="font-medium">Type:</span> ${this.formData.type || 'Not specified'}</div>
                            <div><span class="font-medium">Duration:</span> ${formatDuration(this.formData.duration)}</div>
                            ${this.formData.description ? `<div><span class="font-medium">Description:</span> ${this.formData.description}</div>` : ''}
                        </div>
                    </div>
                    
                    <div>
                        <h4 class="font-semibold text-forest mb-2">Schedule Preferences</h4>
                        <div class="space-y-2 text-sm">
                            <div><span class="font-medium">Dates:</span> ${this.formData.dateRange.mode === 'multiple'
                ? this.formData.dateRange.dates.join(', ')
                : `${this.formData.dateRange.start} to ${this.formData.dateRange.end}`
            }</div>
                            <div><span class="font-medium">Timezone:</span> ${this.formData.timezone}</div>
                            <div><span class="font-medium">Pattern:</span> ${this.formData.recurringPattern || 'Flexible'}</div>
                            <div><span class="font-medium">Preferred Times:</span> ${formatTimeSlots(this.formData.timeSlots)}</div>
                        </div>
                    </div>
                </div>
                
                <div>
                    <h4 class="font-semibold text-forest mb-2">Invited Players (${this.formData.emails.length})</h4>
                    <div class="flex flex-wrap gap-2">
                        ${this.formData.emails.map(email => `
                            <span class="participant-chip">${email}</span>
                        `).join('')}
                    </div>
                </div>
                
                <div class="grid md:grid-cols-2 gap-6">
                    <div>
                        <h4 class="font-semibold text-forest mb-2">Settings</h4>
                        <div class="space-y-2 text-sm">
                            <div><span class="font-medium">Privacy:</span> ${this.formData.privacy || 'Private'}</div>
                            ${this.formData.deadline ? `<div><span class="font-medium">Response Deadline:</span> ${this.formData.deadline}</div>` : ''}
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    async handleFormSubmission() {
        const submitBtn = document.querySelector('button[type="submit"]') || document.getElementById('next-btn');
        if (submitBtn) {
            submitBtn.disabled = true;
            submitBtn.textContent = 'Creating...';
        }

        try {
            // Append extra details to description since we don't have dedicated fields yet
            let fullDescription = this.formData.description || "";
            if (this.formData.timezone) {
                fullDescription += `\n\nTimezone: ${this.formData.timezone}`;
            }
            if (this.formData.recurringPattern) {
                fullDescription += `\nRecurring Pattern: ${this.formData.recurringPattern}`;
            }

            // Prepare dates array based on selection mode
            let datesArray;
            if (this.formData.dateRange.mode === 'multiple') {
                // Use the specific dates selected
                datesArray = this.formData.dateRange.dates;
            } else {
                // Generate all dates in the range for week/month mode
                const start = new Date(this.formData.dateRange.start);
                const end = new Date(this.formData.dateRange.end);
                datesArray = [];

                for (let d = new Date(start); d <= end; d.setDate(d.getDate() + 1)) {
                    datesArray.push(this.formatDateStr(new Date(d)));
                }
            }

            const payload = {
                title: this.formData.title,
                description: fullDescription.trim(),
                location: "Online", // Default to Online
                dates: datesArray,
                timeRange: JSON.stringify(this.formData.timeSlots),
                participants: this.formData.emails
            };

            const response = await fetch('/api/polls', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload)
            });

            if (!response.ok) {
                throw new Error('Failed to create poll');
            }

            const data = await response.json();

            // Show success message
            this.showSuccessMessage();

            // Redirect to manage page after delay
            setTimeout(() => {
                window.location.href = `manage.html?poll=${data.id}`;
            }, 2000);
        } catch (error) {
            console.error('Error creating poll:', error);
            this.showValidationError('Failed to create poll. Please try again.');
            if (submitBtn) {
                submitBtn.disabled = false;
                submitBtn.textContent = 'Finish';
            }
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
                <h3 class="font-cinzel text-2xl font-bold text-forest mb-2">Campaign Created!</h3>
                <p class="text-gray-600 mb-4">Your scheduling poll has been created and invitations sent to players.</p>
                <p class="text-sm text-gray-500">Redirecting to management dashboard...</p>
            </div>
        `;

        document.body.appendChild(message);

        // Animate in
        anime({
            targets: message,
            opacity: [0, 1],
            duration: 300,
            easing: 'easeOutQuart'
        });
    }

    showValidationError(message) {
        // Create error notification
        const error = document.createElement('div');
        error.className = 'fixed top-20 right-4 bg-deep-red text-white p-4 rounded-lg shadow-lg z-50 max-w-sm';
        error.innerHTML = `
            <div class="flex items-start space-x-3">
                <div class="w-6 h-6 bg-white/20 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span class="text-white text-sm">!</span>
                </div>
                <div class="flex-1">
                    <div class="font-semibold text-sm">Validation Error</div>
                    <div class="text-xs opacity-90 mt-1">${message}</div>
                </div>
                <button onclick="this.parentElement.parentElement.remove()" class="text-white/60 hover:text-white text-sm">×</button>
            </div>
        `;

        document.body.appendChild(error);

        // Auto remove after 5 seconds
        setTimeout(() => {
            if (error.parentElement) {
                error.remove();
            }
        }, 5000);
    }
}

// Initialize poll creator when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.pollCreator = new PollCreator();
});