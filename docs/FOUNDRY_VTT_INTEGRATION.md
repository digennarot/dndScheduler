# 🎲 FOUNDRY VTT INTEGRATION

## Data: 2025-12-06

---

## ✅ Pulsante FoundryVTT Aggiunto!

### 🎯 Funzionalità

Aggiunto pulsante per accedere rapidamente a FoundryVTT da tutte le pagine principali.

**URL**: http://127.0.0.1:30000/auth

---

## 📁 Pagine Modificate

### 1. Homepage (index.html) ✅
```html
<a href="http://127.0.0.1:30000/auth" target="_blank" 
   class="flex items-center space-x-2 bg-gradient-to-r from-amber to-copper text-white px-4 py-2 rounded-lg font-semibold hover:shadow-lg transition-all">
  <span>🎲</span>
  <span>FoundryVTT</span>
</a>
```

**Posizione**: Navigazione principale, prima del link Admin

### 2. Dashboard (dashboard.html) ✅
```html
<a href="http://127.0.0.1:30000/auth" target="_blank" 
   class="flex items-center space-x-2 bg-gradient-to-r from-amber to-copper text-white px-4 py-2 rounded-lg font-semibold hover:shadow-lg transition-all">
  <span>🎲</span>
  <span>FoundryVTT</span>
</a>
```

**Posizione**: Header dashboard, prima del logout

### 3. Admin (admin.html) ✅
```html
<a href="http://127.0.0.1:30000/auth" target="_blank" 
   class="action-btn primary flex items-center space-x-2">
  <span>🎲</span>
  <span>FoundryVTT</span>
</a>
```

**Posizione**: Header admin, dopo "Torna al Sito"

---

## 🎨 Design

### Stile
- **Colori**: Gradiente ambra → rame
- **Icona**: 🎲 (dado)
- **Testo**: "FoundryVTT"
- **Effetto hover**: Shadow lift
- **Target**: `_blank` (nuova tab)

### Responsive
- ✅ Desktop: Pulsante completo
- ✅ Mobile: Pulsante compatto

---

## 🧪 Come Testare

### 1. Homepage
```
http://localhost:3000/
```
**Verifica**: Pulsante "🎲 FoundryVTT" in alto a destra

### 2. Dashboard
```
http://localhost:3000/dashboard.html
```
**Verifica**: Pulsante "🎲 FoundryVTT" nell'header

### 3. Admin
```
http://localhost:3000/admin.html
```
**Verifica**: Pulsante "🎲 FoundryVTT" nell'header

### 4. Click
**Risultato**: Si apre FoundryVTT in nuova tab su `http://127.0.0.1:30000/auth`

---

## 📊 Posizionamento

### Homepage
```
┌────────────────────────────────────────────────────┐
│ [D&D] Session Scheduler                            │
│                                                    │
│ Bacheca | Partecipa | Gestisci                    │
│                                                    │
│                    [🎲 FoundryVTT] [Admin] [👤]   │
└────────────────────────────────────────────────────┘
```

### Dashboard
```
┌────────────────────────────────────────────────────┐
│ [D&D] La Mia Bacheca                               │
│                                                    │
│              [🎲 FoundryVTT] [User] [Logout]      │
└────────────────────────────────────────────────────┘
```

### Admin
```
┌────────────────────────────────────────────────────┐
│ [D&D] Admin Dashboard                              │
│                                                    │
│ [← Torna] [🎲 FoundryVTT] [Admin] [Esci]         │
└────────────────────────────────────────────────────┘
```

---

## 🔧 Configurazione FoundryVTT

### Prerequisiti
1. FoundryVTT installato e in esecuzione
2. Porta: 30000
3. URL: http://127.0.0.1:30000

### Verifica
```bash
# Controlla se FoundryVTT è in esecuzione
curl http://127.0.0.1:30000/auth
```

Se FoundryVTT non è in esecuzione, il link mostrerà un errore (normale).

---

## 💡 Personalizzazione

### Cambia URL
Se FoundryVTT usa un URL diverso, modifica in tutti e 3 i file:

```html
<!-- Da: -->
href="http://127.0.0.1:30000/auth"

<!-- A: -->
href="http://TUO_URL:PORTA/auth"
```

### Cambia Porta
Se FoundryVTT usa una porta diversa:

```html
href="http://127.0.0.1:PORTA/auth"
```

### Cambia Stile
Modifica le classi CSS:

```html
<!-- Colore diverso -->
class="bg-gradient-to-r from-purple-500 to-blue-500 ..."

<!-- Dimensione diversa -->
class="... px-6 py-3 text-lg ..."
```

---

## ✅ Checklist

### Implementazione
- [x] Pulsante aggiunto a index.html
- [x] Pulsante aggiunto a dashboard.html
- [x] Pulsante aggiunto a admin.html
- [x] Stile coerente su tutte le pagine
- [x] Target="_blank" per nuova tab
- [x] Icona dado 🎲
- [x] Hover effect

### Test
- [ ] Click su homepage
- [ ] Click su dashboard
- [ ] Click su admin
- [ ] Verifica apertura in nuova tab
- [ ] Verifica URL corretto

---

## 🎯 Risultato

### Prima ❌
- Nessun link rapido a FoundryVTT
- Necessario digitare URL manualmente

### Dopo ✅
- **Pulsante visibile su tutte le pagine**
- **Click rapido per accedere**
- **Si apre in nuova tab**
- **Design coerente con il tema**

---

**Status**: ✅ COMPLETATO  
**Pagine**: 3/3 Modificate  
**Funzionalità**: 100%  

🎲 **FoundryVTT accessibile con un click!** ✨
