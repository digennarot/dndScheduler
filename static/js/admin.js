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

                <!-- Polls Table -->
                <div class="bg-dnd-dark rounded-xl border border-gray-700 shadow-lg overflow-hidden">
                    <div class="p-6 border-b border-gray-700 flex justify-between items-center">
                        <h3 class="font-cinzel text-lg text-amber">Gestione Sondaggi</h3>
                        <button onclick="location.reload()" class="text-gray-400 hover:text-white transition-colors">
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
                                <tr>
                                    <td colspan="5" class="p-8 text-center text-gray-500">Caricamento sondaggi evocati...</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        `;

        this.loadPolls();
    }

    async loadPolls() {
        try {
            const response = await fetch('/api/polls');
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

            tbody.innerHTML = polls.map(poll => `
                <tr class="hover:bg-white/5 transition-colors group">
                    <td class="p-4">
                        <div class="font-medium text-parchment truncate max-w-xs" title="${poll.title}">${poll.title}</div>
                        <div class="text-xs text-gray-500 truncate max-w-xs">${poll.description || '-'}</div>
                    </td>
                    <td class="p-4 text-gray-300">
                        ${poll.organizer || 'Sconosciuto'}
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
                            <a href="/p/${poll.id}" target="_blank" class="p-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-300 transition-colors" title="Visualizza">
                                👁️
                            </a>
                            <button onclick="dashboard.deletePoll('${poll.id}')" 
                                class="p-2 bg-deep-red/20 hover:bg-deep-red text-deep-red hover:text-white rounded transition-colors" title="Elimina Definitivamente">
                                🗑️
                            </button>
                        </div>
                    </td>
                </tr>
            `).join('');

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
                alert('Errore durante l\'eliminazione. Poteri insufficienti?');
            }
        } catch (error) {
            console.error('Delete failed:', error);
            alert('Errore di connessione.');
        }
    }
}

// Global instance for inline event handlers
window.dashboard = new AdminDashboard();
