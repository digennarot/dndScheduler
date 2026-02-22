/**
 * Reminder Manager
 * Gestisce l'invio di promemoria via WhatsApp e Telegram
 */

class ReminderManager {
    constructor() {
        this.whatsappEnabled = false;
        this.telegramEnabled = false;
        this.init();
    }

    /**
     * Inizializza il manager
     */
    init() {
        // Verifica se WhatsApp e Telegram sono configurati
        this.checkConfiguration();
    }

    /**
     * Verifica configurazione servizi
     */
    async checkConfiguration() {
        try {
            const response = await fetch('/api/reminder/config');
            if (response.ok) {
                const config = await response.json();
                this.whatsappEnabled = config.whatsapp_enabled || false;
                this.telegramEnabled = config.telegram_enabled || false;
            }
        } catch (error) {
            console.log('Configurazione reminder non disponibile, uso valori di default');
        }
    }

    /**
     * Invia promemoria WhatsApp
     */
    async sendWhatsAppReminder(userId, sessionId, message) {
        if (!this.whatsappEnabled) {
            return {
                success: false,
                message: 'WhatsApp non configurato. Configura il servizio nelle impostazioni.'
            };
        }

        try {
            // Ottieni numero telefono utente
            const user = await this.getUserById(userId);
            if (!user || !user.phone) {
                return {
                    success: false,
                    message: 'Utente non ha un numero di telefono configurato'
                };
            }

            // Invia tramite API
            const response = await fetch('/api/reminder/whatsapp', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    phone: user.phone,
                    message: message,
                    session_id: sessionId
                })
            });

            if (response.ok) {
                return {
                    success: true,
                    message: `Promemoria WhatsApp inviato a ${user.name}`
                };
            } else {
                const error = await response.json();
                return {
                    success: false,
                    message: error.message || 'Errore invio WhatsApp'
                };
            }
        } catch (error) {
            console.error('Errore invio WhatsApp:', error);
            return {
                success: false,
                message: 'Errore di connessione'
            };
        }
    }

    /**
     * Invia promemoria Telegram
     */
    async sendTelegramReminder(userId, sessionId, message) {
        if (!this.telegramEnabled) {
            return {
                success: false,
                message: 'Telegram non configurato. Configura il bot nelle impostazioni.'
            };
        }

        try {
            // Ottieni chat_id Telegram utente
            const user = await this.getUserById(userId);
            if (!user || !user.telegram_chat_id) {
                return {
                    success: false,
                    message: 'Utente non ha Telegram configurato'
                };
            }

            // Invia tramite API
            const response = await fetch('/api/reminder/telegram', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    chat_id: user.telegram_chat_id,
                    message: message,
                    session_id: sessionId
                })
            });

            if (response.ok) {
                return {
                    success: true,
                    message: `Promemoria Telegram inviato a ${user.name}`
                };
            } else {
                const error = await response.json();
                return {
                    success: false,
                    message: error.message || 'Errore invio Telegram'
                };
            }
        } catch (error) {
            console.error('Errore invio Telegram:', error);
            return {
                success: false,
                message: 'Errore di connessione'
            };
        }
    }

    /**
     * Invia promemoria multipli
     */
    async sendReminders(sessionId, userIds, channels = ['email']) {
        const results = {
            success: [],
            failed: []
        };

        const session = await this.getSessionById(sessionId);
        if (!session) {
            return {
                success: false,
                message: 'Sessione non trovata'
            };
        }

        // Messaggio di default
        const message = this.createReminderMessage(session);

        for (const userId of userIds) {
            for (const channel of channels) {
                let result;

                switch (channel) {
                    case 'whatsapp':
                        result = await this.sendWhatsAppReminder(userId, sessionId, message);
                        break;
                    case 'telegram':
                        result = await this.sendTelegramReminder(userId, sessionId, message);
                        break;
                    case 'email':
                        result = await this.sendEmailReminder(userId, sessionId, message);
                        break;
                    default:
                        continue;
                }

                if (result.success) {
                    results.success.push({
                        userId,
                        channel,
                        message: result.message
                    });
                } else {
                    results.failed.push({
                        userId,
                        channel,
                        error: result.message
                    });
                }
            }
        }

        return results;
    }

    /**
     * Crea messaggio promemoria
     */
    createReminderMessage(session) {
        return `🎲 Promemoria D&D Session Scheduler

📋 Sessione: ${session.name}
📅 Data: ${session.date || 'Da definire'}
⏰ Ora: ${session.time || 'Da definire'}
📍 Luogo: ${session.location || 'Da definire'}

Non dimenticare di indicare la tua disponibilità!

👉 ${window.location.origin}/participate.html?session=${session.id}`;
    }

    /**
     * Invia promemoria email (fallback)
     */
    async sendEmailReminder(userId, sessionId, message) {
        try {
            const response = await fetch('/api/reminder/email', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    user_id: userId,
                    session_id: sessionId,
                    message: message
                })
            });

            if (response.ok) {
                return {
                    success: true,
                    message: 'Email inviata'
                };
            } else {
                return {
                    success: false,
                    message: 'Errore invio email'
                };
            }
        } catch (error) {
            return {
                success: false,
                message: 'Errore di connessione'
            };
        }
    }

    /**
     * Ottieni utente per ID
     */
    async getUserById(userId) {
        try {
            const response = await fetch(`/api/users/${userId}`);
            if (response.ok) {
                return await response.json();
            }
        } catch (error) {
            console.error('Errore caricamento utente:', error);
        }
        return null;
    }

    /**
     * Ottieni sessione per ID
     */
    async getSessionById(sessionId) {
        try {
            const response = await fetch(`/api/polls/${sessionId}`);
            if (response.ok) {
                return await response.json();
            }
        } catch (error) {
            console.error('Errore caricamento sessione:', error);
        }
        return null;
    }

    /**
     * Mostra dialog selezione canali
     */
    showChannelSelector(sessionId, userIds, callback) {
        const channels = [];

        // Email sempre disponibile
        channels.push({
            id: 'email',
            name: 'Email',
            icon: '📧',
            enabled: true
        });

        // WhatsApp se configurato
        if (this.whatsappEnabled) {
            channels.push({
                id: 'whatsapp',
                name: 'WhatsApp',
                icon: '💬',
                enabled: true
            });
        }

        // Telegram se configurato
        if (this.telegramEnabled) {
            channels.push({
                id: 'telegram',
                name: 'Telegram',
                icon: '✈️',
                enabled: true
            });
        }

        // Crea HTML dialog
        const html = `
            <div class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4" id="channel-selector-modal">
                <div class="bg-gray-800 rounded-2xl max-w-md w-full p-6">
                    <h3 class="font-cinzel text-xl font-bold text-amber mb-4">
                        Seleziona Canali di Invio
                    </h3>
                    <p class="text-gray-300 mb-6">
                        Scegli come inviare i promemoria ai ${userIds.length} giocatori selezionati:
                    </p>
                    <div class="space-y-3 mb-6">
                        ${channels.map(channel => `
                            <label class="flex items-center p-3 bg-gray-700 rounded-lg cursor-pointer hover:bg-gray-600 transition-colors">
                                <input type="checkbox" 
                                       value="${channel.id}" 
                                       class="mr-3 w-5 h-5 text-amber focus:ring-amber"
                                       ${channel.id === 'email' ? 'checked' : ''}>
                                <span class="text-2xl mr-3">${channel.icon}</span>
                                <span class="text-gray-200">${channel.name}</span>
                            </label>
                        `).join('')}
                    </div>
                    <div class="flex space-x-3">
                        <button onclick="reminderManager.confirmSend('${sessionId}', ${JSON.stringify(userIds)})" 
                                class="flex-1 bg-emerald-600 hover:bg-emerald-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors">
                            Invia Promemoria
                        </button>
                        <button onclick="reminderManager.closeChannelSelector()" 
                                class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors">
                            Annulla
                        </button>
                    </div>
                </div>
            </div>
        `;

        document.body.insertAdjacentHTML('beforeend', html);
    }

    /**
     * Conferma invio
     */
    async confirmSend(sessionId, userIds) {
        const modal = document.getElementById('channel-selector-modal');
        const selectedChannels = Array.from(modal.querySelectorAll('input[type="checkbox"]:checked'))
            .map(cb => cb.value);

        if (selectedChannels.length === 0) {
            alert('Seleziona almeno un canale di invio');
            return;
        }

        this.closeChannelSelector();

        // Mostra loading
        if (window.adminManager) {
            window.adminManager.showNotification(
                'Invio Promemoria',
                'Invio in corso...',
                'info'
            );
        }

        // Invia
        const results = await this.sendReminders(sessionId, userIds, selectedChannels);

        // Mostra risultati
        const successCount = results.success.length;
        const failedCount = results.failed.length;

        if (window.adminManager) {
            window.adminManager.showNotification(
                'Promemoria Inviati',
                `✅ ${successCount} inviati con successo\n❌ ${failedCount} falliti`,
                successCount > 0 ? 'success' : 'error'
            );
        }
    }

    /**
     * Chiudi dialog
     */
    closeChannelSelector() {
        const modal = document.getElementById('channel-selector-modal');
        if (modal) {
            modal.remove();
        }
    }
}

// Inizializza globalmente
window.reminderManager = new ReminderManager();
