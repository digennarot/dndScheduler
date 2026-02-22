// Admin Manager Configuration
// Production-ready configuration for admin panel

const AdminConfig = {
    // Environment detection
    isDevelopment: window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1',

    // API Configuration
    api: {
        baseUrl: '', // Same origin
        endpoints: {
            login: '/api/admin/login',
            googleLogin: '/api/admin/google-login',
            currentUser: '/api/admin/me',
            users: '/api/admin/users',
            polls: '/api/polls',
        },
        timeout: 30000, // 30 seconds
    },

    // Security Configuration
    security: {
        tokenKey: 'adminToken',
        userKey: 'adminUser',
        tokenRefreshInterval: 5 * 60 * 1000, // Check token every 5 minutes
        sessionTimeout: 24 * 60 * 60 * 1000, // 24 hours in ms
    },

    // Feature Flags
    features: {
        enableDebugLogs: false, // Set to true in development
        enableGoogleAuth: true,
        enableEmailNotifications: true,
        autoRefreshData: true,
        autoRefreshInterval: 60000, // 1 minute
    },

    // UI Configuration
    ui: {
        defaultAvatar: 'AD',
        dateFormat: 'en-US',
        timeFormat: '24h',
        notificationDuration: 5000, // 5 seconds
    },

    // Logging
    log: function(...args) {
        if (this.features.enableDebugLogs || this.isDevelopment) {
            console.log('[AdminManager]', ...args);
        }
    },

    warn: function(...args) {
        console.warn('[AdminManager]', ...args);
    },

    error: function(...args) {
        console.error('[AdminManager]', ...args);
    },
};

// Freeze configuration to prevent tampering
if (typeof Object.freeze === 'function') {
    Object.freeze(AdminConfig.api.endpoints);
    Object.freeze(AdminConfig.security);
    Object.freeze(AdminConfig.features);
    Object.freeze(AdminConfig.ui);
}

// Export for use in other scripts
window.AdminConfig = AdminConfig;
