# D&D Session Scheduler

A sophisticated web application for scheduling Dungeons & Dragons sessions, inspired by LettuceMeet but enhanced with mystical aesthetics and advanced features.

## 🎲 Features

### Core Functionality
- **Interactive Scheduling Polls**: Create polls with date ranges and time preferences
- **Real-time Availability Tracking**: Click-and-drag interface for marking availability
- **Overlap Visualization**: Heat maps showing optimal meeting times
- **Group Management**: Track participant responses and send reminders
- **Session Finalization**: Select best times and notify all players

### Design & User Experience
- **Mystical Aesthetic**: Fantasy-inspired design with particle effects and aurora backgrounds
- **Responsive Design**: Works seamlessly on desktop and mobile devices
- **Intuitive Interface**: Clean, modern UI with D&D-themed visual elements
- **Real-time Updates**: Live notifications and status updates

### Technical Features
- **Multi-step Form Wizard**: Guided poll creation process
- **Advanced Analytics**: Response tracking and availability analysis
- **Interactive Calendar Grid**: Visual availability marking system
- **Data Visualization**: Charts and heat maps for overlap analysis
- **Local Storage**: Draft saving and session persistence

## 🏗️ Architecture

### Frontend Technologies
- **HTML5**: Semantic markup with accessibility features
- **Tailwind CSS**: Utility-first styling framework
- **JavaScript ES6+**: Modern JavaScript with class-based architecture
- **Animation Libraries**: 
  - Anime.js for smooth transitions
  - p5.js for particle effects
  - ECharts.js for data visualization

### Design System
- **Color Palette**: Deep forest greens, mystical purples, warm ambers
- **Typography**: Cinzel for headings, Inter for body text, JetBrains Mono for code
- **Visual Effects**: Aurora gradients, floating particles, mystical glows
- **Interactive Elements**: Hover states, click animations, loading indicators

## 📁 Project Structure

```
/
├── index.html              # Main dashboard
├── create-poll.html        # Poll creation wizard
├── participate.html        # Player availability interface
├── manage.html            # Session management dashboard
├── js/
│   ├── app.js             # Main application controller
│   ├── particles.js       # Particle effects system
│   ├── dashboard.js       # Dashboard functionality
│   ├── poll-creator.js    # Poll creation wizard
│   ├── availability-manager.js  # Availability tracking
│   └── session-manager.js # Session management
├── resources/             # Images and assets
│   ├── hero-main.jpg      # Main hero image
│   ├── interface-magic.jpg # Interface mockup
│   ├── collaboration-scene.jpg # Collaboration visual
│   └── notifications-magic.jpg # Notification system visual
├── design.md              # Design system documentation
├── interaction.md         # User interaction specifications
├── outline.md             # Project architecture outline
└── README.md              # This file
```

## 🚀 Getting Started

### Prerequisites
- Modern web browser (Chrome, Firefox, Safari, Edge)
- Local web server (optional but recommended)
- Google account (for admin authentication)

### Installation
1. Clone or download the project files
2. Navigate to the project directory
3. Start a local web server:
   ```bash
   python -m http.server 8000
   ```
4. Open your browser and navigate to `http://localhost:8000`

### Usage
1. **Create a Session**: Use the poll creation wizard to set up your campaign
2. **Invite Players**: Add participant email addresses
3. **Mark Availability**: Players use the interactive grid to indicate their availability
4. **Analyze Overlap**: View heat maps and recommendations for optimal times
5. **Finalize Session**: Select the best time and notify all participants

### Admin Setup (Optional)

For production deployment with Google OAuth:

1. **Set up Google OAuth** (see [GOOGLE_OAUTH_SETUP.md](./GOOGLE_OAUTH_SETUP.md))
   - Create Google Cloud project
   - Configure OAuth credentials
   - Add authorized redirect URIs
   - Update Client ID in admin-manager.js

2. **Configure Admin Access**
   - Add admin email addresses to whitelist
   - Set up proper domain and HTTPS
   - Test authentication flow

**For immediate testing**: The admin panel includes a demo authentication system that works without Google OAuth setup. Use the "Sign in with Google (Demo)" button for admin access.

## 🎯 Key Interactions

### For Organizers (Dungeon Masters)
- **Dashboard Overview**: See all active campaigns and response rates
- **Poll Creation**: Step-by-step wizard for setting up scheduling polls
- **Session Management**: Track responses, send reminders, and finalize times
- **Analytics**: View overlap analysis and participant availability patterns

### For Players
- **Session Selection**: Browse available campaigns to join
- **Availability Grid**: Click and drag to mark available, tentative, or busy times
- **Bulk Actions**: Quickly mark weekends, weekdays, or clear all
- **Real-time Feedback**: See group overlap and best time recommendations

### For Administrators
- **Admin Dashboard**: Comprehensive platform management
- **User Management**: Monitor and manage all platform users
- **Session Oversight**: View and moderate all D&D sessions
- **System Analytics**: Platform usage and performance insights
- **Google Authentication**: Secure admin access with OAuth

## 🎨 Design Philosophy

The application embraces a "Mystical Modernism" aesthetic that combines:
- **Fantasy Elements**: Ancient scrolls, magical particles, mystical symbols
- **Modern Usability**: Clean interfaces, intuitive interactions, accessibility
- **Professional Polish**: Smooth animations, responsive design, error handling
- **D&D Flavor**: Dice motifs, parchment textures, arcane color schemes

## 🔐 Admin Backend

### Features
- **Google OAuth Integration**: Secure admin authentication
- **User Management**: Complete user oversight and control
- **Session Moderation**: Monitor and manage all D&D sessions
- **System Analytics**: Platform usage and performance metrics
- **Admin Controls**: System settings and maintenance tools

### Access
- Navigate to the "Admin" link in the main navigation
- Sign in with a Google account that has admin privileges
- Manage users, sessions, and system settings from the admin dashboard

See [ADMIN_GUIDE.md](./ADMIN_GUIDE.md) for detailed admin documentation.

## 🔧 Customization

### Styling
- Modify colors in the Tailwind config object
- Update particle effects in `particles.js`
- Adjust animations in individual component files

### Functionality
- Extend the data models in `app.js`
- Add new features to the respective manager classes
- Integrate with backend APIs for persistent storage

## 🌟 Advanced Features

### Real-time Updates
- Live response tracking
- Instant notifications
- Dynamic overlap calculations

### Data Visualization
- Availability heat maps
- Response rate charts
- Overlap analysis graphs

### User Experience
- Keyboard shortcuts
- Mobile-optimized touch interactions
- Draft saving and recovery
- Contextual help and tips

## 🎮 Demo Data

The application includes sample data for demonstration:
- **Campaigns**: Tomb of Annihilation, Curse of Strahd, Waterdeep Dragon Heist
- **Players**: Diverse set of participants with different availability patterns
- **Responses**: Realistic response data showing various participation levels

## 🔮 Future Enhancements

- **Backend Integration**: Database persistence and user authentication
- **Calendar Integration**: Google Calendar, Outlook, and Apple Calendar sync
- **Advanced Analytics**: Participation patterns and scheduling insights
- **Mobile App**: Native iOS and Android applications
- **Recurring Sessions**: Support for ongoing campaign scheduling
- **Notification System**: Email and push notifications

## 🤝 Contributing

This project welcomes contributions! Areas for improvement:
- Additional scheduling algorithms
- Enhanced mobile interactions
- Accessibility improvements
- Performance optimizations
- New visual effects and animations

## 📄 License

This project is created for educational and demonstration purposes. Feel free to use and modify for your own D&D scheduling needs.

---

**Crafted with magical precision for the D&D community** 🎲✨