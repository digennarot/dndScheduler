# API Contracts

## Overview
The D&D Scheduler API is a RESTful service built with Axum. All API endpoints are prefixed with `/api`.

**Base URL:** `http://localhost:3000/api`

## Authentication
The API supports multiple authentication methods:
- **Admin:** JWT (stateless)
- **User (Player/DM):** Hybrid (Database Session Token or Google JWT)
- **Participant:** Access Token (for specific poll actions)

## Endpoints

### Authentication & User Management

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/auth/register` | Register a new user | No |
| `POST` | `/auth/login` | Login with email/password | No |
| `POST` | `/auth/google/login` | Login with Google ID Token | No |
| `POST` | `/auth/logout/:token` | Logout user | Yes |
| `GET` | `/auth/me/:token` | Get current user profile | Yes |
| `PUT` | `/auth/profile` | Update profile information | Yes |
| `PUT` | `/auth/password` | Change password | Yes |
| `DELETE` | `/auth/account` | Delete account | Yes |

#### Authelia SSO
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/auth/authelia/config` | Get SSO configuration |
| `GET` | `/auth/authelia/session` | Verify authelia session |

### Polls

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/polls` | List all polls | Yes (DM only?) |
| `POST` | `/polls` | Create a new poll | Yes (DM only) |
| `GET` | `/polls/:id` | Get poll details | No (Public/Link) |
| `PUT` | `/polls/:id` | Update poll details | Yes (Owner/DM) |
| `DELETE` | `/polls/:id` | Delete a poll | Yes (Owner/DM) |
| `PUT` | `/polls/:id/finalize` | Finalize a poll time | Yes (Admin/DM) |

### Participation

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/polls/:id/join` | Join a poll | No (Public) |
| `POST` | `/polls/:id/participants/:pid/availability` | Update availability | Yes (Access Token) |
| `DELETE` | `/participants/:id` | Remove participant | Yes (DM) |

### Admin

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/admin/login` | Admin login | No |
| `GET` | `/admin/me` | Get admin profile | Yes (Admin) |
| `GET` | `/admin/users` | List all users | Yes (Admin) |
| `PUT` | `/admin/users/:id/role` | Update user role | Yes (Admin) |
| `GET` | `/admin/stats` | Get system statistics | Yes (Admin) |

### Feature Modules

#### Activity Feed
- `GET /activity/recent` - Get recent system activity

#### Reminders
- `GET /reminder/config` - Get reminder settings
- `POST /reminder/whatsapp` - Send WhatsApp reminder
- `POST /reminder/telegram` - Send Telegram reminder
- `POST /reminder/email` - Send Email reminder

#### GDPR Compliance
- `GET/POST /gdpr/consent` - Manage user consents
- `GET /gdpr/export` - Export user data
- `POST /gdpr/delete` - Confirm account deletion
