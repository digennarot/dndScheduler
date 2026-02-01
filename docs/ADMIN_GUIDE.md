# D&D Session Scheduler - Admin Guide

## Overview

The admin backend provides comprehensive management capabilities for the D&D Session Scheduler platform. Built with Google OAuth authentication, it offers secure access to user management, session oversight, analytics, and system configuration.

## Features

### 🔐 Secure Authentication
- **Google OAuth Integration**: Secure login with Google accounts
- **Admin-Only Access**: Restricted to authorized administrator emails
- **Session Management**: Automatic logout and session timeout
- **Audit Trail**: Track admin actions and system changes

### 📊 Dashboard Overview
- **Real-time Statistics**: User counts, session activity, response rates
- **System Health**: Performance metrics and uptime monitoring
- **Activity Charts**: Visual representation of platform usage
- **Quick Actions**: Common admin tasks and shortcuts

### 👥 User Management
- **User List**: Complete overview of all platform users
- **Role Management**: Organizers, participants, and admin roles
- **Status Monitoring**: Online/offline status and last activity
- **User Actions**: Edit, suspend, or view user details
- **Search & Filter**: Find users quickly and efficiently

### 🎲 Session Management
- **Session Overview**: All D&D sessions across the platform
- **Status Tracking**: Active, finalized, and cancelled sessions
- **DM Management**: Track session organizers and their campaigns
- **Session Actions**: View, edit, or delete sessions as needed
- **Moderation Tools**: Handle inappropriate content or violations

### 📈 Analytics & Reporting
- **Usage Trends**: Platform activity over time
- **Performance Metrics**: System health and resource usage
- **User Behavior**: Session creation and participation patterns
- **Export Capabilities**: Data export for external analysis

### ⚙️ System Settings
- **Maintenance Mode**: Enable/disable platform access
- **Email Configuration**: Notification and communication settings
- **Security Settings**: Session timeout and login attempt limits
- **API Configuration**: Google OAuth and external service settings

## Getting Started

### Accessing the Admin Panel

1. **Navigate to Admin Portal**: Click "Admin" in the main navigation
2. **Google Authentication**: Sign in with your Google account
3. **Admin Verification**: Only pre-approved admin emails can access
4. **Dashboard Access**: Successful login redirects to admin dashboard

### First-Time Setup

1. **Configure Google OAuth**:
   - Obtain Google Client ID from Google Cloud Console
   - Add authorized redirect URIs
   - Configure admin email whitelist

2. **Set Up Admin Accounts**:
   - Define admin email addresses
   - Configure role permissions
   - Set up notification preferences

3. **System Configuration**:
   - Adjust security settings
   - Configure email notifications
   - Set performance thresholds

## Admin Interface

### Dashboard Layout

```
┌─────────────────────────────────────────────────────────┐
│  Header: Logo, Admin Info, Logout                      │
├─────────────────────────────────────────────────────────┤
│  Sidebar Navigation                                    │
│  ├─ 📊 Overview                                        │
│  ├─ 👥 Users                                           │
│  ├─ 🎲 Sessions                                        │
│  ├─ 📈 Analytics                                       │
│  └─ ⚙️ Settings                                        │
├─────────────────────────────────────────────────────────┤
│  Main Content Area                                     │
│  ├─ Statistics Cards                                   │
│  ├─ Charts & Graphs                                    │
│  ├─ Data Tables                                        │
│  └─ Action Buttons                                     │
└─────────────────────────────────────────────────────────┘
```

### Navigation Tabs

#### Overview Tab
- **Statistics Cards**: Users, sessions, response rates, system status
- **Activity Chart**: User activity over the past week
- **Session Distribution**: Pie chart of session statuses
- **Quick Actions**: Common admin tasks

#### Users Tab
- **User Table**: Complete list with search and filter
- **User Actions**: Edit, view, suspend options
- **Status Indicators**: Online/offline status
- **Role Management**: Organizer/participant/admin roles

#### Sessions Tab
- **Session Table**: All platform sessions
- **Status Filtering**: Active, finalized, cancelled
- **DM Information**: Session organizers
- **Session Actions**: View, edit, delete options

#### Analytics Tab
- **Usage Trends**: Historical platform usage
- **Performance Metrics**: System health indicators
- **User Behavior**: Participation patterns
- **Export Options**: Data export functionality

#### Settings Tab
- **General Settings**: Maintenance mode, notifications
- **Security Settings**: Session timeout, login attempts
- **API Configuration**: Google OAuth, rate limits
- **System Preferences**: Platform-wide configurations

## User Management

### Viewing Users

The users table displays:
- **User Avatar & Name**: Visual identification
- **Email Address**: Contact information
- **Role**: Organizer, participant, or admin
- **Status**: Online, offline, or suspended
- **Last Active**: Timestamp of last platform activity
- **Actions**: Available management options

### User Actions

#### Edit User
- Modify user roles and permissions
- Update contact information
- Change user status
- Add administrative notes

#### View User
- Detailed user profile
- Session participation history
- Activity logs and statistics
- Communication preferences

#### Suspend User
- Temporarily disable account access
- Preserve user data and history
- Send notification to user
- Set suspension duration and reason

## Session Management

### Session Overview

The sessions table provides:
- **Session Title**: Campaign name
- **DM Information**: Session organizer
- **Player Count**: Number of participants
- **Status**: Active, finalized, or cancelled
- **Creation Date**: When the session was created
- **Actions**: Available management options

### Session Actions

#### View Session
- Complete session details
- Participant responses
- Availability overlap analysis
- Session history and changes

#### Edit Session
- Modify session parameters
- Update participant list
- Change session status
- Add administrative notes

#### Delete Session
- Permanently remove session
- Preserve user data
- Notify participants
- Log deletion reason

## Analytics & Reporting

### Usage Trends
- **Time-based Analysis**: Daily, weekly, monthly usage
- **User Growth**: New user registration trends
- **Session Activity**: Creation and participation patterns
- **Engagement Metrics**: Response rates and completion

### Performance Metrics
- **System Health**: CPU, memory, and response times
- **Uptime Monitoring**: Service availability tracking
- **Error Rates**: Platform stability indicators
- **Load Balancing**: Traffic distribution analysis

### User Behavior
- **Participation Patterns**: How users engage with sessions
- **Scheduling Preferences**: Popular times and days
- **Campaign Types**: Most popular adventure types
- **Geographic Distribution**: User location analysis

## System Settings

### General Settings

#### Maintenance Mode
- **Enable/Disable**: Control platform availability
- **User Notification**: Inform users of maintenance
- **Admin Access**: Allow admin login during maintenance
- **Scheduled Maintenance**: Set automatic maintenance windows

#### Email Notifications
- **User Communications**: Session updates and reminders
- **Admin Alerts**: System issues and important events
- **Broadcast Messages**: Platform-wide announcements
- **Notification Preferences**: User opt-in/out settings

### Security Settings

#### Session Timeout
- **Duration**: How long before automatic logout
- **Warning Period**: Alert before session expiration
- **Remember Me**: Extended session options
- **Concurrent Sessions**: Multiple login handling

#### Login Attempts
- **Maximum Attempts**: Failed login threshold
- **Lockout Duration**: Temporary account suspension
- **IP Blocking**: Prevent brute force attacks
- **Two-Factor Authentication**: Enhanced security option

### API Configuration

#### Google OAuth
- **Client ID**: Google Cloud Console credentials
- **Redirect URIs**: Authorized callback URLs
- **Scopes**: Requested user permissions
- **Admin Whitelist**: Authorized admin email addresses

#### Rate Limiting
- **API Limits**: Requests per minute/hour
- **User Limits**: Individual user restrictions
- **IP Limits**: Address-based throttling
- **Burst Protection**: Handle traffic spikes

#### Authelia SSO
- **Configuration**: Managed via Caddy headers (ForwardAuth)
- **Role Sync**: Automatic admin role assignment based on groups
- **Groups**: `admins`, `admin`, or `administrators`
- **Setup Guide**: See `docs/CADDY_AUTHELIA_CONFIG.md`

## Best Practices

### Security
1. **Regular Updates**: Keep all components current
2. **Strong Passwords**: Use complex admin passwords
3. **Two-Factor Auth**: Enable when possible
4. **Access Logs**: Monitor admin activity
5. **Backup Data**: Regular system backups

### User Management
1. **Respect Privacy**: Handle user data responsibly
2. **Clear Communication**: Explain actions to users
3. **Consistent Policies**: Apply rules fairly
4. **Documentation**: Keep records of changes
5. **Escalation**: Have procedures for issues

### System Maintenance
1. **Monitor Performance**: Watch system metrics
2. **Regular Backups**: Protect against data loss
3. **Test Updates**: Verify changes in staging
4. **User Communication**: Inform about maintenance
5. **Emergency Procedures**: Plan for outages

## Troubleshooting

### Common Issues

#### Login Problems
- **Invalid Credentials**: Check Google account access
- **Admin Access**: Verify email in admin whitelist
- **OAuth Errors**: Check Google Client ID configuration
- **Session Expired**: Clear browser cache and cookies

#### Performance Issues
- **Slow Loading**: Check server resources
- **Chart Errors**: Verify data source connectivity
- **UI Glitches**: Clear browser cache
- **Mobile Issues**: Test responsive design

#### User Management
- **Data Not Loading**: Check network connectivity
- **Action Failures**: Verify admin permissions
- **Search Issues**: Check filter configurations
- **Export Problems**: Verify file permissions

### Getting Help

1. **Check Documentation**: Review this guide
2. **System Logs**: Check error logs for issues
3. **Community Support**: Contact development team
4. **Emergency Contact**: Escalate critical issues

## Future Enhancements

### Planned Features
- **Advanced Analytics**: Machine learning insights
- **User Communication**: Built-in messaging system
- **Automated Moderation**: AI-powered content review
- **Mobile App**: Native admin application
- **Integration APIs**: Connect with external tools

### Scalability Improvements
- **Database Optimization**: Handle larger user bases
- **Caching Systems**: Improve performance
- **Load Balancing**: Distribute traffic
- **CDN Integration**: Faster global access
- **Microservices**: Modular architecture

---

This admin backend provides comprehensive management capabilities for the D&D Session Scheduler platform. With secure authentication, detailed analytics, and intuitive interfaces, administrators can effectively manage users, sessions, and system configuration while maintaining platform security and performance.