class AdminDashboard {
    constructor() {
        this.container = document.getElementById('app-container');
        this.init();
    }

    async init() {
        try {
            await this.checkAuth();
        } catch (e) {
            this.renderLogin();
        }
    }

    async checkAuth() {
        const response = await fetch('/api/admin/me');
        if (response.ok) {
            const admin = await response.json();
            this.renderDashboard(admin);
        } else {
            throw new Error('Not authenticated');
        }
    }

    renderLogin() {
        this.container.innerHTML = `
            <div class="max-w-md mx-auto bg-dnd-dark p-8 rounded-xl shadow-2xl border border-gray-700 mt-12">
                <div class="text-center mb-8">
                    <h2 class="font-cinzel text-2xl font-bold text-parchment">Accesso Admin</h2>
                    <p class="text-gray-400 mt-2">Area riservata ai Dungeon Master Supremi</p>
                </div>
                
                <form id="admin-login-form" class="space-y-6">
                    <div>
                        <label class="block text-sm font-semibold text-gray-400 mb-2">Token di Accesso Supremo</label>
                        <input type="password" id="admin-token" required placeholder="Inserisci il token segreto..."
                            class="w-full px-4 py-3 bg-dnd-black border border-gray-600 rounded-lg focus:ring-2 focus:ring-amber focus:border-transparent text-white placeholder-gray-500 transition-all font-mono text-center tracking-widest">
                    </div>

                    <div id="login-error" class="hidden text-deep-red text-sm text-center bg-deep-red/10 p-2 rounded"></div>

                    <button type="submit" 
                        class="w-full bg-amber text-dnd-black py-3 rounded-lg font-bold font-cinzel hover:bg-yellow-500 transform hover:scale-[1.02] transition-all shadow-lg">
                        Invoca Potere
                    </button>
                    
                    <p class="text-xs text-center text-gray-500 mt-4">
                        Solo chi possiede la Parola del Potere può entrare.
                    </p>
                </form>
            </div>
        `;

        document.getElementById('admin-login-form').addEventListener('submit', (e) => this.handleLogin(e));
    }

    async handleLogin(e) {
        e.preventDefault();
        const token = document.getElementById('admin-token').value;
        const errorDiv = document.getElementById('login-error');
        const btn = e.target.querySelector('button');

        btn.disabled = true;
        btn.innerHTML = '<span class="animate-spin inline-block w-4 h-4 border-2 border-dnd-black border-t-transparent rounded-full mr-2"></span> Evocazione...';
        errorDiv.classList.add('hidden');

        try {
            const response = await fetch('/api/admin/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ token })
            });

            if (response.ok) {
                await this.checkAuth();
            } else {
                const data = await response.json();
                throw new Error(data.error || 'Il token non è valido.');
            }
        } catch (error) {
            errorDiv.textContent = error.message;
            errorDiv.classList.remove('hidden');
            btn.disabled = false;
            btn.textContent = 'Invoca Potere';
        }
    }

    async renderDashboard(admin) {
        this.container.innerHTML = `
            <div class="space-y-8 fade-in">
                <!-- Welcome Banner -->
                <div class="bg-dnd-dark p-6 rounded-xl border border-gray-700 shadow-lg flex justify-between items-center">
                    <div>
                        <h2 class="font-cinzel text-xl text-parchment">Benvenuto, ${admin.username}</h2>
                        <p class="text-gray-400 text-sm">Sessione attiva. Potere illimitato.</p>
                    </div>
                    <div class="text-right">
                        <div class="text-3xl font-bold text-forest" id="total-polls">-</div>
                        <div class="text-xs text-gray-500 uppercase tracking-wider">Sondaggi Attivi</div>
                    </div>
                </div>

                <!-- Tabs -->
                <div class="flex gap-4 border-b border-gray-700 pb-2">
                    <button onclick="dashboard.switchTab('polls')" id="tab-polls" class="px-4 py-2 font-cinzel text-amber border-b-2 border-amber transition-colors">
                        Gestione Sondaggi
                    </button>
                    <button onclick="dashboard.switchTab('users')" id="tab-users" class="px-4 py-2 font-cinzel text-gray-400 hover:text-white transition-colors">
                        Utenti & Ruoli
                    </button>
                    <button onclick="dashboard.switchTab('stats')" id="tab-stats" class="px-4 py-2 font-cinzel text-gray-400 hover:text-white transition-colors">
                        Statistiche
                    </button>
                </div>

                <!-- Dynamic Content Area -->
                <div id="dashboard-content">
                    <!-- Default to Polls -->
                    ${this.getPollsTemplate()}
                </div>
            </div>
        `;

        this.currentTab = 'polls';
        this.loadPolls();
    }

    switchTab(tab) {
        if (this.currentTab === tab) return;
        this.currentTab = tab;

        // Update Tab Styles
        document.querySelectorAll('[id^="tab-"]').forEach(el => {
            el.classList.remove('text-amber', 'border-b-2', 'border-amber');
            el.classList.add('text-gray-400');
        });
        const activeTab = document.getElementById(`tab-${tab}`);
        activeTab.classList.remove('text-gray-400');
        activeTab.classList.add('text-amber', 'border-b-2', 'border-amber');

        // Update Content
        const contentDiv = document.getElementById('dashboard-content');
        if (tab === 'polls') {
            contentDiv.innerHTML = this.getPollsTemplate();
            this.loadPolls();
        } else if (tab === 'users') {
            contentDiv.innerHTML = this.getUsersTemplate();
            this.loadUsers();
        } else if (tab === 'stats') {
            contentDiv.innerHTML = '<div class="p-8 text-center text-gray-500">Statistiche in lavorazione...</div>';
        }
    }

    getPollsTemplate() {
        return `
            <div class="bg-dnd-dark rounded-xl border border-gray-700 shadow-lg overflow-hidden">
                <div class="p-6 border-b border-gray-700 flex justify-between items-center">
                    <h3 class="font-cinzel text-lg text-amber">Gestione Sondaggi</h3>
                    <button onclick="dashboard.loadPolls()" class="text-gray-400 hover:text-white transition-colors">
                        🔄
                    </button>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-black/30 text-gray-400 text-sm uppercase tracking-wider">
                                <th class="p-4 font-semibold">Titolo</th>
                                <th class="p-4 font-semibold">Organizzatore</th>
                                <th class="p-4 font-semibold">Stato</th>
                                <th class="p-4 font-semibold">Creato il</th>
                                <th class="p-4 font-semibold text-right">Azioni</th>
                            </tr>
                        </thead>
                        <tbody id="polls-table-body" class="divide-y divide-gray-700">
                            <tr><td colspan="5" class="p-8 text-center text-gray-500">Caricamento...</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
        `;
    }

    getUsersTemplate() {
        return `
            <div class="bg-dnd-dark rounded-xl border border-gray-700 shadow-lg overflow-hidden">
                 <div class="p-6 border-b border-gray-700 flex justify-between items-center">
                    <h3 class="font-cinzel text-lg text-amber">Lista Utenti</h3>
                    <button onclick="dashboard.loadUsers()" class="text-gray-400 hover:text-white transition-colors">
                        🔄
                    </button>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-black/30 text-gray-400 text-sm uppercase tracking-wider">
                                <th class="p-4 font-semibold">Utente</th>
                                <th class="p-4 font-semibold">Email</th>
                                <th class="p-4 font-semibold">Ruolo</th>
                                <th class="p-4 font-semibold">Registrato il</th>
                                <th class="p-4 font-semibold text-right">Azioni</th>
                            </tr>
                        </thead>
                        <tbody id="users-table-body" class="divide-y divide-gray-700">
                            <tr><td colspan="5" class="p-8 text-center text-gray-500">Evocazione utenti...</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
        `;
    }

    async loadUsers() {
        try {
            const response = await fetch('/api/admin/users');
            if (response.status === 401) return location.reload(); // Re-login if expired

            const users = await response.json();
            const tbody = document.getElementById('users-table-body');

            if (users.length === 0) {
                tbody.innerHTML = '<tr><td colspan="5" class="p-8 text-center text-gray-500">Nessun utente trovato.</td></tr>';
                return;
            }

            tbody.innerHTML = users.map(user => `
                <tr class="hover:bg-white/5 transition-colors group">
                    <td class="p-4">
                        <div class="font-medium text-parchment">${user.name}</div>
                    </td>
                    <td class="p-4 text-gray-400">${user.email}</td>
                    <td class="p-4">
                        <select onchange="dashboard.updateRole('${user.id}', this.value)" 
                                class="bg-dnd-black border border-gray-600 text-xs rounded px-2 py-1 text-gray-300 focus:border-amber focus:ring-1 focus:ring-amber outline-none">
                            <option value="player" ${user.role === 'player' ? 'selected' : ''}>Player</option>
                            <option value="dm" ${user.role === 'dm' ? 'selected' : ''}>Dungeon Master</option>
                            <option value="admin" ${user.role === 'admin' ? 'selected' : ''}>Admin</option>
                        </select>
                    </td>
                    <td class="p-4 text-gray-500 text-sm">
                        ${new Date(user.created_at * 1000).toLocaleDateString('it-IT')}
                    </td>
                    <td class="p-4 text-right">
                         <button onclick="dashboard.deleteUser('${user.id}', '${user.email}')" 
                                class="p-2 bg-deep-red/20 hover:bg-deep-red text-deep-red hover:text-white rounded transition-colors" title="Elimina Utente">
                                🗑️
                         </button>
                    </td>
                </tr>
            `).join('');

        } catch (error) {
            console.error('Failed to load users:', error);
            document.getElementById('users-table-body').innerHTML = `<tr><td colspan="5" class="p-8 text-center text-deep-red">Errore nel caricamento.</td></tr>`;
        }
    }

    async updateRole(userId, newRole) {
        try {
            const response = await fetch(`/api/admin/users/${userId}/role`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ role: newRole })
            });

            if (response.ok) {
                // Success feedback (optional, maybe a toast)
            } else {
                alert('Errore nell\'aggiornamento del ruolo.');
                this.loadUsers(); // Revert UI
            }
        } catch (e) {
            console.error(e);
            alert('Errore di connessione.');
        }
    }

    async deleteUser(userId, email) {
        if (!confirm(`SEI SICURO? Stai per eliminare l'utente ${email}. Questa azione è irreversibile e cancellerà anche tutte le sue sessioni e voti.`)) {
            return;
        }

        try {
            const response = await fetch(`/api/admin/users/${userId}`, {
                method: 'DELETE'
            });

            if (response.ok) {
                this.loadUsers();
            } else {
                const data = await response.json();
                alert(data.error || 'Errore durante l\'eliminazione.');
            }
        } catch (e) {
            console.error(e);
            alert('Errore di connessione.');
        }
    }

    async loadPolls() {
        try {
            const response = await fetch('/api/admin/polls');
            if (response.status === 401) return location.reload();

            const polls = await response.json();

            // Update stats
            document.getElementById('total-polls').textContent = polls.length;

            const tbody = document.getElementById('polls-table-body');

            if (polls.length === 0) {
                tbody.innerHTML = `
                <tr>
                <td colspan="5" class="p-8 text-center text-gray-500 italic">
                    Nessun sondaggio trovato nel reame.
                </td>
                    </tr>
                `;
                return;
            }

            tbody.innerHTML = polls.map(item => {
                const poll = item.poll || item; // Handle both enriched and raw if fallback needed
                const organizer = item.organizer_name || 'Sconosciuto';
                const partCount = item.participant_count || 0;

                return `
                <tr class="hover:bg-white/5 transition-colors group">
                    <td class="p-4">
                        <div class="font-medium text-parchment truncate max-w-xs" title="${poll.title}">${poll.title}</div>
                        <div class="text-xs text-gray-500 truncate max-w-xs">${poll.description || '-'}</div>
                    </td>
                    <td class="p-4 text-gray-300">
                        <div class="font-medium text-amber">${organizer}</div>
                        <div class="text-xs text-gray-500">${partCount} Partecipanti</div>
                    </td>
                    <td class="p-4">
                        <span class="px-2 py-1 rounded text-xs font-semibold ${poll.status === 'active' ? 'bg-forest/20 text-forest' : 'bg-gray-700 text-gray-300'}">
                            ${poll.status === 'active' ? 'ATTIVO' : 'CHIUSO'}
                        </span>
                    </td>
                    <td class="p-4 text-gray-400 text-sm">
                        ${new Date(poll.created_at * 1000).toLocaleDateString('it-IT')}
                    </td>
                    <td class="p-4 text-right">
                        <div class="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                            <button onclick="dashboard.renderPollDetailsModal('${poll.id}')" 
                                class="p-2 bg-amber/20 hover:bg-amber text-amber hover:text-dnd-black rounded transition-colors" 
                                title="Gestisci Dettagli">
                                ⚙️
                            </button>
                            <button onclick="dashboard.finalizePoll('${poll.id}')" 
                                class="p-2 bg-blue-900/40 hover:bg-blue-800 text-blue-200 rounded transition-colors ${poll.status !== 'active' ? 'hidden' : ''}" 
                                title="Finalizza (Chiudi)">
                                🏁
                            </button>
                            <a href="/p/${poll.id}" target="_blank" class="p-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-300 transition-colors" title="Visualizza Pubblico">
                                👁️
                            </a>
                            <button onclick="dashboard.deletePoll('${poll.id}')" 
                                class="p-2 bg-deep-red/20 hover:bg-deep-red text-deep-red hover:text-white rounded transition-colors" title="Elimina Definitivamente">
                                🗑️
                            </button>
                        </div>
                    </td>
                </tr>
                `}).join('');

        } catch (error) {
            console.error('Failed to load polls:', error);
            document.getElementById('polls-table-body').innerHTML = `
                <tr>
                <td colspan="5" class="p-8 text-center text-deep-red">
                    Errore nel recupero dei dati arcani.
                </td>
                </tr>
                `;
        }
    }

    async renderPollDetailsModal(pollId) {
        try {
            const response = await fetch(`/api/polls/${pollId}`);
            if (!response.ok) throw new Error("Errore nel recupero dettagli");
            const data = await response.json();
            const { poll, participants, availability } = data;

            // Remove existing modal if any
            const existingModal = document.getElementById('poll-details-modal');
            if (existingModal) existingModal.remove();

            // Create Modal HTML
            const modal = document.createElement('div');
            modal.id = 'poll-details-modal';
            modal.className = 'fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4 fade-in';

            const dates = JSON.parse(poll.dates || '[]');

            modal.innerHTML = `
                <div class="bg-dnd-dark w-full max-w-4xl max-h-[90vh] rounded-xl border border-gray-700 shadow-2xl flex flex-col">
                    <!-- Header -->
                    <div class="p-6 border-b border-gray-700 flex justify-between items-start bg-black/20">
                        <div>
                            <h3 class="font-cinzel text-2xl text-amber mb-1">${poll.title}</h3>
                            <div class="flex gap-4 text-sm text-gray-400">
                                <span>📍 ${poll.location}</span>
                                <span>📅 ${dates.length} date proposte</span>
                                <span>👥 ${participants.length} partecipanti</span>
                            </div>
                        </div>
                        <button onclick="document.getElementById('poll-details-modal').remove()" class="text-gray-400 hover:text-white text-2xl leading-none">&times;</button>
                    </div>

                    <!-- Content (Scrollable) -->
                    <div class="p-6 overflow-y-auto space-y-8">
                        
                        <!-- Participants List -->
                        <section>
                            <h4 class="font-cinzel text-lg text-parchment mb-4 border-b border-gray-700 pb-2">Partecipanti</h4>
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                ${participants.length > 0 ? participants.map(p => `
                                    <div class="bg-black/30 p-3 rounded border border-gray-700 flex justify-between items-center">
                                        <span class="text-gray-300 font-medium">${p.name}</span>
                                        <div class="flex gap-2">
                                            <button onclick="dashboard.deleteParticipant('${p.id}', '${p.name}')" class="text-deep-red hover:text-red-400 text-sm" title="Rimuovi">✕</button>
                                        </div>
                                    </div>
                                `).join('') : '<p class="text-gray-500 italic">Nessun avventuriero ha risposto alla chiamata.</p>'}
                            </div>
                        </section>

                        <!-- Availability Summary (Simple) -->
                        <section>
                            <h4 class="font-cinzel text-lg text-parchment mb-4 border-b border-gray-700 pb-2">Riepilogo Disponibilità</h4>
                            <div class="overflow-x-auto">
                                <table class="w-full text-sm text-left">
                                    <thead>
                                        <tr class="text-gray-500 border-b border-gray-700">
                                            <th class="py-2">Data</th>
                                            <th class="py-2 text-center">✅ Sì</th>
                                            <th class="py-2 text-center">⚠️ Forse</th>
                                            <th class="py-2 text-center">❌ No</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-gray-800">
                                        ${dates.map(date => {
                // Calculate counts for this date
                let yes = 0, maybe = 0, no = 0;
                availability.forEach(a => {
                    if (a.date === date) {
                        if (a.status === 'yes') yes++;
                        else if (a.status === 'maybe') maybe++;
                        else if (a.status === 'no') no++;
                    }
                });
                // Handle cases where availability records might be missing (implicit no/unknown)
                // Logic simplified for admin view

                return `
                                            <tr>
                                                <td class="py-2 text-gray-300">${new Date(date).toLocaleDateString()}</td>
                                                <td class="py-2 text-center text-forest font-bold">${yes}</td>
                                                <td class="py-2 text-center text-yellow-500">${maybe}</td>
                                                <td class="py-2 text-center text-deep-red">${no}</td>
                                            </tr>
                                            `;
            }).join('')}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        <!-- JSON Raw Data (Toggle) -->
                        <details class="group">
                             <summary class="cursor-pointer text-xs text-gray-600 hover:text-amber transition-colors list-none">
                                ▶ Debug Utente & JSON
                             </summary>
                             <pre class="mt-2 bg-black/50 p-4 rounded text-xs text-green-400 overflow-x-auto font-mono">${JSON.stringify(poll, null, 2)}</pre>
                        </details>
                    </div>

                    <!-- Footer Actions -->
                    <div class="p-6 border-t border-gray-700 bg-black/20 flex justify-end gap-3">
                        <button onclick="document.getElementById('poll-details-modal').remove()" class="px-4 py-2 text-gray-400 hover:text-white transition-colors">Chiudi</button>
                        ${poll.status === 'active' ? `
                            <button onclick="dashboard.finalizePoll('${poll.id}'); document.getElementById('poll-details-modal').remove()" class="px-4 py-2 bg-blue-900 hover:bg-blue-800 text-white rounded font-cinzel transition-colors">
                                Finalizza
                            </button>
                        ` : ''}
                    </div>
                </div>
            `;

            document.body.appendChild(modal);

        } catch (e) {
            console.error(e);
            alert("Errore nel caricamento dei dettagli.");
        }
    }

    async deleteParticipant(participantId, name) {
        if (!confirm(`Rimuovere ${name} da questo sondaggio?`)) return;
        try {
            const response = await fetch(`/api/participants/${participantId}`, { method: 'DELETE' });
            if (response.ok) {
                // Refresh modal content (hacky reuse of renderPollDetailsModal with current ID?)
                // Actually need pollId. But checking DOM for context or just reload dashboard?
                // Ideally reload modal. Let's close and reload dashboard for now.
                document.getElementById('poll-details-modal').remove();
                alert(`${name} rimosso.`);
                // We should ideally reload the modal, but we need pollId.
                // It was not passed to this function.
                // Optimization: Just dashboard.loadPolls() and user has to reopen.
                this.loadPolls();
            } else {
                alert("Errore nella rimozione.");
            }
        } catch (e) {
            alert("Errore di connessione.");
        }
    }

    async finalizePoll(pollId) {
        // Simple prompt for now, can be improved to a modal later
        const time = prompt("Inserisci l'orario finale (es. 'Lunedì 20:30'):");
        if (!time) return;

        try {
            const response = await fetch(`/api/polls/${pollId}/finalize`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    finalized_time: time,
                    notes: "Finalizzato da Admin"
                })
            });

            if (response.ok) {
                this.loadPolls();
            } else {
                const data = await response.json();
                alert(data.error || 'Errore durante la finalizzazione.');
            }
        } catch (e) {
            console.error(e);
            alert('Errore di connessione.');
        }
    }

    async deletePoll(pollId) {
        if (!confirm('Sei sicuro? Questa azione è irreversibile e cancellerà il sondaggio e tutti i voti associati.')) {
            return;
        }

        try {
            const response = await fetch(`/api/polls/${pollId}`, {
                method: 'DELETE'
            });

            if (response.ok) {
                // Remove row with animation
                this.loadPolls();
            } else {
                const data = await response.json();
                alert(data.error || 'Errore durante l\'eliminazione. Poteri insufficienti?');
            }
        } catch (error) {
            console.error('Delete failed:', error);
            alert('Errore di connessione.');
        }
    }
}

// Global instance for inline event handlers
window.dashboard = new AdminDashboard();
