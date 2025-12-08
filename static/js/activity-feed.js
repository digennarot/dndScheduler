/**
 * Activity Feed Module
 * Mostra le attività recenti sulla homepage/dashboard
 */

(function () {
    'use strict';

    // Configuration for activity types
    const ACTIVITY_TYPES = {
        'poll_created': {
            icon: '🎲',
            color: 'emerald',
            // API messages are pre-formatted, but we keep template for mocks
            template: (data) => `<strong>${data.user}</strong> ha creato la campagna <strong>${data.poll}</strong>`
        },
        'response_submitted': {
            icon: '✅',
            color: 'blue',
            template: (data) => `<strong>${data.user}</strong> ha indicato la disponibilità per <strong>${data.poll}</strong>`
        },
        'poll_finalized': {
            icon: '🎯',
            color: 'purple',
            template: (data) => `La sessione <strong>${data.poll}</strong> è stata finalizzata per il <strong>${data.date}</strong>`
        },
        'user_joined': {
            icon: '👋',
            color: 'amber',
            template: (data) => `<strong>${data.user}</strong> si è unito alla piattaforma`
        },
        'reminder_sent': {
            icon: '📧',
            color: 'cyan',
            template: (data) => `Promemoria inviato per <strong>${data.poll}</strong> a ${data.count} giocatori`
        },
        'default': {
            icon: '📝',
            color: 'gray',
            template: (data) => data.message || 'Attività generica'
        }
    };

    /**
     * Carica e mostra attività recenti
     */
    async function loadRecentActivity() {
        const container = document.getElementById('activity-feed');
        if (!container) return;

        try {
            // Mostra loading
            container.innerHTML = '<div class="text-center text-gray-500 py-8">Caricamento attività...</div>';

            // Carica attività
            const activities = await fetchActivities();

            if (!activities || activities.length === 0) {
                showNoActivity(container);
                return;
            }

            // Mostra attività
            displayActivities(container, activities);

        } catch (error) {
            console.error('Errore caricamento attività:', error);
            showError(container);
        }
    }

    /**
     * Recupera attività dall'API
     */
    async function fetchActivities() {
        const response = await fetch('/api/activity/recent?limit=10');
        if (response.ok) {
            const rawActivities = await response.json();
            // Process API data to add UI properties
            return rawActivities.map(processActivity);
        }
        throw new Error('API request failed');
    }

    /**
     * Arricchisce i dati grezzi dell'attività con proprietà UI (icona, colore, tempo)
     */
    function processActivity(activity) {
        const typeConfig = ACTIVITY_TYPES[activity.activity_type] || ACTIVITY_TYPES['default'];

        // Calculate time ago
        const timestamp = new Date(activity.timestamp * 1000); // Backend uses Unix timestamp (seconds)

        return {
            ...activity,
            icon: typeConfig.icon,
            color: typeConfig.color,
            timeAgo: getTimeAgo(timestamp),
            // Backend sends Unix timestamp, convert if needed for sorting logic elsewhere, 
            // but we already have timeAgo for display
            timestampDate: timestamp
        };
    }



    /**
     * Mostra attività nel container
     */
    function displayActivities(container, activities) {
        const html = activities.map(activity => `
            <div class="bg-white rounded-lg p-4 shadow-sm hover:shadow-md transition-shadow border-l-4 border-${activity.color}-500">
                <div class="flex items-start space-x-3">
                    <div class="flex-shrink-0 text-2xl">
                        ${activity.icon}
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-sm text-gray-800">
                            ${activity.message}
                        </p>
                        <p class="text-xs text-gray-500 mt-1">
                            ${activity.timeAgo}
                        </p>
                    </div>
                </div>
            </div>
        `).join('');

        container.innerHTML = html;
    }

    /**
     * Mostra messaggio nessuna attività
     */
    function showNoActivity(container) {
        container.innerHTML = `
            <div class="text-center py-12">
                <div class="text-6xl mb-4">🎲</div>
                <h4 class="font-cinzel text-xl font-semibold text-gray-700 mb-2">
                    Nessuna Attività Recente
                </h4>
                <p class="text-gray-500">
                    Inizia creando la tua prima campagna!
                </p>
                <button onclick="window.location.href='create-poll.html'" 
                    class="mt-4 bg-forest text-white px-6 py-2 rounded-lg font-semibold hover:bg-forest/90 transition-colors">
                    Crea Campagna
                </button>
            </div>
        `;
    }

    /**
     * Mostra messaggio di errore
     */
    function showError(container) {
        container.innerHTML = `
            <div class="text-center py-12">
                <div class="text-6xl mb-4">⚠️</div>
                <h4 class="font-cinzel text-xl font-semibold text-gray-700 mb-2">
                    Errore Caricamento
                </h4>
                <p class="text-gray-500 mb-4">
                    Impossibile caricare le attività recenti
                </p>
                <button onclick="window.location.reload()" 
                    class="bg-forest text-white px-6 py-2 rounded-lg font-semibold hover:bg-forest/90 transition-colors">
                    Riprova
                </button>
            </div>
        `;
    }

    /**
     * Formatta data
     */
    function formatDate(date) {
        const options = {
            day: 'numeric',
            month: 'long',
            year: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        };
        return date.toLocaleDateString('it-IT', options);
    }

    /**
     * Calcola tempo trascorso
     */
    function getTimeAgo(date) {
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);

        if (diffMins < 1) return 'Proprio ora';
        if (diffMins < 60) return `${diffMins} ${diffMins === 1 ? 'minuto' : 'minuti'} fa`;
        if (diffHours < 24) return `${diffHours} ${diffHours === 1 ? 'ora' : 'ore'} fa`;
        if (diffDays < 7) return `${diffDays} ${diffDays === 1 ? 'giorno' : 'giorni'} fa`;

        return formatDate(date);
    }

    /**
     * Inizializza quando il DOM è pronto
     */
    function init() {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', loadRecentActivity);
        } else {
            loadRecentActivity();
        }
    }

    // Avvia
    init();

    // Espone funzione per ricaricare
    window.reloadActivity = loadRecentActivity;

})();
