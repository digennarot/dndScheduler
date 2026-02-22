/**
 * Statistics Module
 * Calcola e mostra statistiche per la homepage
 */

(function () {
    'use strict';

    /**
     * Carica tutte le statistiche
     */
    async function loadStatistics() {
        try {
            // Carica i polls dal localStorage o API
            const polls = await loadPolls();

            if (!polls || polls.length === 0) {
                showNoDataMessage();
                return;
            }

            // Calcola statistiche
            const stats = calculateStatistics(polls);

            // Mostra statistiche
            displaySuccessRate(stats.successRate);
            displayAvgResponseTime(stats.avgResponseTime);
            displayWeeklyActivity(stats.weeklyActivity);

        } catch (error) {
            console.error('Errore caricamento statistiche:', error);
            showErrorMessage();
        }
    }

    /**
     * Carica i polls
     */
    async function loadPolls() {
        // Carica solo da API - nessun mock data
        try {
            const response = await fetch('/api/polls');
            if (response.ok) {
                const polls = await response.json();
                // Carica dettagli per ogni poll per avere i partecipanti
                const pollsWithDetails = await Promise.all(
                    polls.map(async (poll) => {
                        try {
                            const detailResponse = await fetch(`/api/polls/${poll.id}`);
                            if (detailResponse.ok) {
                                const data = await detailResponse.json();
                                return {
                                    ...poll,
                                    status: data.poll.status || 'active',
                                    participants: data.participants || [],
                                    availability: data.availability || []
                                };
                            }
                        } catch (e) {
                            console.warn(`Impossibile caricare dettagli poll ${poll.id}`);
                        }
                        return poll;
                    })
                );
                return pollsWithDetails;
            }
        } catch (e) {
            console.error('Errore caricamento polls:', e);
        }

        // Nessun dato disponibile
        return [];
    }

    /**
     * Calcola statistiche dai polls
     */
    function calculateStatistics(polls) {
        // Tasso di successo: % di polls finalizzati
        const finalizedCount = polls.filter(p => p.status === 'finalized').length;
        const successRate = polls.length > 0 ? Math.round((finalizedCount / polls.length) * 100) : 0;

        // Tempo risposta medio
        let totalResponseTime = 0;
        let responseCount = 0;

        polls.forEach(poll => {
            if (poll.participants) {
                poll.participants.forEach(participant => {
                    if (participant.responded_at && poll.created_at) {
                        const created = new Date(poll.created_at);
                        const responded = new Date(participant.responded_at);
                        const diffDays = Math.floor((responded - created) / (1000 * 60 * 60 * 24));
                        totalResponseTime += diffDays;
                        responseCount++;
                    }
                });
            }
        });

        const avgResponseTime = responseCount > 0 ? Math.round(totalResponseTime / responseCount) : 0;

        // Attività settimanale (ultimi 7 giorni)
        const weeklyActivity = calculateWeeklyActivity(polls);

        return {
            successRate,
            avgResponseTime,
            weeklyActivity
        };
    }

    /**
     * Calcola attività settimanale
     */
    function calculateWeeklyActivity(polls) {
        const now = new Date();
        const weekData = Array(7).fill(0);
        const dayNames = ['Dom', 'Lun', 'Mar', 'Mer', 'Gio', 'Ven', 'Sab'];

        polls.forEach(poll => {
            const created = new Date(poll.created_at);
            const diffDays = Math.floor((now - created) / (1000 * 60 * 60 * 24));

            if (diffDays < 7) {
                weekData[6 - diffDays]++;
            }
        });

        return {
            labels: Array(7).fill(0).map((_, i) => {
                const d = new Date(now);
                d.setDate(d.getDate() - (6 - i));
                return dayNames[d.getDay()];
            }),
            data: weekData
        };
    }

    /**
     * Mostra tasso di successo
     */
    function displaySuccessRate(rate) {
        const element = document.getElementById('success-rate');
        if (element) {
            element.textContent = `${rate}%`;
            element.classList.add('animate-fade-in');
        }
    }

    /**
     * Mostra tempo risposta medio
     */
    function displayAvgResponseTime(days) {
        const element = document.getElementById('avg-response-time');
        if (element) {
            if (days === 0) {
                element.textContent = '< 1g';
            } else if (days === 1) {
                element.textContent = '1 giorno';
            } else {
                element.textContent = `${days} giorni`;
            }
            element.classList.add('animate-fade-in');
        }
    }

    /**
     * Mostra grafico attività settimanale
     */
    function displayWeeklyActivity(activity) {
        const chartElement = document.getElementById('availability-chart');
        if (!chartElement) return;

        // Verifica se ECharts è disponibile
        if (typeof echarts === 'undefined') {
            console.warn('ECharts non disponibile, mostro grafico semplice');
            displaySimpleChart(chartElement, activity);
            return;
        }

        // Usa ECharts
        const chart = echarts.init(chartElement);
        const option = {
            tooltip: {
                trigger: 'axis',
                axisPointer: {
                    type: 'shadow'
                }
            },
            grid: {
                left: '3%',
                right: '4%',
                bottom: '3%',
                containLabel: true
            },
            xAxis: {
                type: 'category',
                data: activity.labels,
                axisTick: {
                    alignWithLabel: true
                }
            },
            yAxis: {
                type: 'value'
            },
            series: [{
                name: 'Sondaggi Creati',
                type: 'bar',
                barWidth: '60%',
                data: activity.data,
                itemStyle: {
                    color: '#dc2626',
                    borderRadius: [4, 4, 0, 0]
                }
            }]
        };

        chart.setOption(option);

        // Responsive
        window.addEventListener('resize', () => chart.resize());
    }

    /**
     * Mostra grafico semplice senza ECharts
     */
    function displaySimpleChart(container, activity) {
        const maxValue = Math.max(...activity.data, 1);

        let html = '<div class="flex items-end justify-between h-full gap-2">';

        activity.data.forEach((value, index) => {
            const height = (value / maxValue) * 100;
            html += `
                <div class="flex-1 flex flex-col items-center gap-2">
                    <div class="text-xs text-gray-600">${value}</div>
                    <div class="w-full bg-dnd-red rounded-t" style="height: ${height}%"></div>
                    <div class="text-xs text-gray-500">${activity.labels[index]}</div>
                </div>
            `;
        });

        html += '</div>';
        container.innerHTML = html;
    }

    /**
     * Mostra messaggio "nessun dato"
     */
    function showNoDataMessage() {
        document.getElementById('success-rate').textContent = '0%';
        document.getElementById('avg-response-time').textContent = '-';

        const chartElement = document.getElementById('availability-chart');
        if (chartElement) {
            chartElement.innerHTML = '<div class="flex items-center justify-center h-full text-gray-400">Nessun dato disponibile</div>';
        }
    }

    /**
     * Mostra messaggio di errore
     */
    function showErrorMessage() {
        document.getElementById('success-rate').textContent = 'N/D';
        document.getElementById('avg-response-time').textContent = 'N/D';

        const chartElement = document.getElementById('availability-chart');
        if (chartElement) {
            chartElement.innerHTML = '<div class="flex items-center justify-center h-full text-red-400">Errore caricamento dati</div>';
        }
    }

    /**
     * Inizializza quando il DOM è pronto
     */
    function init() {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', loadStatistics);
        } else {
            loadStatistics();
        }
    }

    // Avvia
    init();

    // Espone funzione per ricaricare
    window.reloadStatistics = loadStatistics;

})();
