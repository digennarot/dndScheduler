/**
 * MagicalGrid
 *
 * The visual layer of the D&D Scheduler (The Ritual).
 * Renders a CSS Grid representing time slots and days.
 * Handles user interaction (drag-to-paint) relative to the DOM.
 */
export class MagicalGrid {
    constructor(containerId, config = {}) {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            console.error(`MagicalGrid: Container #${containerId} not found.`);
            return;
        }

        this.config = Object.assign({
            days: [], // Array of date strings
            timeSlots: [], // Array of time strings "HH:MM"
            onSlotChange: (day, time, state) => { }, // Callback
            mode: 'edit', // 'edit' | 'heatmap'
            heatmapData: {} // { "yyyy-mm-dd_hh:mm": { count: N, voters: ["Name"] } }
        }, config);

        this.isDragging = false;
        this.paintState = null; // 'available', 'tentative', 'empty'
        this.tooltip = null;

        this.init();
    }

    init() {
        console.log("MagicalGrid: Initializing the Ritual...");
        this.createTooltip();
        this.render();
        this.setupInteractions();
    }

    createTooltip() {
        this.tooltip = document.createElement('div');
        this.tooltip.className = 'grid-tooltip';
        document.body.appendChild(this.tooltip);
    }

    render() {
        this.container.innerHTML = '';
        this.container.classList.add('magical-grid');

        // CSS Grid Layout: TimeLabels + Days...
        const numCols = this.config.days.length + 1; // +1 for labels
        this.container.style.gridTemplateColumns = `60px repeat(${this.config.days.length}, 1fr)`;

        this.renderHeaders();
        this.renderSlots();
    }

    renderHeaders() {
        // Corner
        const corner = document.createElement('div');
        corner.className = 'grid-cell header';
        this.container.appendChild(corner);

        // Day Headers
        this.config.days.forEach(day => {
            const el = document.createElement('div');
            el.className = 'grid-cell header';
            el.textContent = this.formatDate(day);
            this.container.appendChild(el);
        });
    }

    renderSlots() {
        this.config.timeSlots.forEach(timeLabel => {
            // Time Label
            const label = document.createElement('div');
            label.className = 'grid-cell time-label';
            label.textContent = timeLabel;
            this.container.appendChild(label);

            // Cells
            this.config.days.forEach((day, dayIndex) => {
                const cell = document.createElement('div');
                cell.id = `${day}_${timeLabel}`; // ID format: YYYY-MM-DD_HH:MM
                cell.dataset.day = day;
                cell.dataset.time = timeLabel;

                if (this.config.mode === 'heatmap') {
                    // HEATMAP MODE
                    cell.className = 'grid-cell';
                    const key = `${day}_${timeLabel}`;
                    const data = this.config.heatmapData[key];

                    if (data && data.count > 0) {
                        // Ensure max intensity is 5
                        const intensity = Math.min(data.count, 5);
                        cell.classList.add(`heatmap-${intensity}`);
                        cell.dataset.voters = data.voters.join(', ');
                        cell.dataset.count = data.count;

                        // Tooltip events
                        cell.addEventListener('mouseenter', (e) => this.showTooltip(e, data));
                        cell.addEventListener('mouseleave', () => this.hideTooltip());
                        cell.addEventListener('mousemove', (e) => this.moveTooltip(e));
                    } else {
                        cell.classList.add('heatmap-0');
                    }

                } else {
                    // EDIT MODE
                    cell.className = 'grid-cell clickable';
                    cell.dataset.state = 'empty';
                    // Prevent text selection during drag
                    cell.addEventListener('mousedown', (e) => e.preventDefault());
                }

                this.container.appendChild(cell);
            });
        });
    }

    setupInteractions() {
        if (this.config.mode !== 'edit') return;

        // MOUSE EVENTS
        this.container.addEventListener('mousedown', (e) => this.handleStart(e));
        this.container.addEventListener('mouseover', (e) => this.handleMove(e));
        document.addEventListener('mouseup', () => this.handleEnd());

        // TOUCH EVENTS (Basic support, requires resolving elementFromPoint)
        this.container.addEventListener('touchstart', (e) => this.handleStart(e));
        this.container.addEventListener('touchmove', (e) => this.handleTouchMove(e));
        this.container.addEventListener('touchend', () => this.handleEnd());
    }

    // ... Interaction Handlers (handleStart, handleMove, etc.) ...

    handleStart(e) {
        if (this.config.mode !== 'edit') return;
        const cell = e.target.closest('.grid-cell.clickable');
        if (!cell) return;

        e.preventDefault(); // Prevent scroll/selection
        this.isDragging = true;

        // Determine next state based on current state
        // Cycle: Empty -> Available (Green) -> Tentative (Yellow) -> Empty
        const currentState = cell.dataset.state;
        if (currentState === 'empty') this.paintState = 'available';
        else if (currentState === 'available') this.paintState = 'tentative';
        else this.paintState = 'empty';

        this.applyState(cell);
    }

    handleMove(e) {
        if (!this.isDragging) return;
        const cell = e.target.closest('.grid-cell.clickable');
        if (cell) {
            this.applyState(cell);
        }
    }

    handleTouchMove(e) {
        if (!this.isDragging) return;
        e.preventDefault(); // Prevent scrolling

        const touch = e.touches[0];
        const target = document.elementFromPoint(touch.clientX, touch.clientY);
        if (target) {
            const cell = target.closest('.grid-cell.clickable');
            if (cell) {
                this.applyState(cell);
            }
        }
    }

    handleEnd() {
        this.isDragging = false;
        this.paintState = null;
    }

    applyState(cell) {
        if (cell.dataset.state === this.paintState) return;

        cell.dataset.state = this.paintState;

        // Update Classes
        cell.classList.remove('available', 'tentative', 'busy');
        if (this.paintState !== 'empty') {
            cell.classList.add(this.paintState);
        }

        // Notify
        if (this.config.onSlotChange) {
            this.config.onSlotChange(cell.dataset.day, cell.dataset.time, this.paintState);
        }
    }

    // ... Tooltip Methods ...

    showTooltip(e, data) {
        const count = data.count;
        const voterList = data.voters.join('<br>');
        this.tooltip.innerHTML = `<strong>${count} voted:</strong><br>${voterList}`;
        this.tooltip.classList.add('visible');
        this.moveTooltip(e);
    }

    hideTooltip() {
        this.tooltip.classList.remove('visible');
    }

    moveTooltip(e) {
        const x = e.pageX + 10;
        const y = e.pageY + 10;
        this.tooltip.style.left = `${x}px`;
        this.tooltip.style.top = `${y}px`;
    }

    formatDate(dateStr) {
        const date = new Date(dateStr);
        return date.toLocaleDateString('it-IT', { weekday: 'short', day: 'numeric' });
    }

    updateConfig(newConfig) {
        this.config = Object.assign(this.config, newConfig);
        this.render();
        // Re-setup interactions if switching back to edit mode
        if (this.config.mode === 'edit') {
            // Remove old listeners to be safe? 
            // Actually setupInteractions adds to container, which is preserved.
            // But we clear innerHTML.
            // We need to re-bind listeners if we were in heatmap mode?
            // Actually the container listeners are persistent.
        }
    }
}
