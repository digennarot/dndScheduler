# Project Overview

## D&D Scheduler Application

### Purpose
To facilitate the scheduling of D&D sessions by allowing dungeon masters to create polls and players to vote on availability, with a focus on ease of use and a thematic "fantasy" aesthetic.

### Executive Summary
A comprehensive scheduling tool featuring a Rust backend for reliability and a Vanilla JS frontend for a lightweight, interactive user experience. Designed to be self-hosted with minimal dependencies.

### Technology Stack Summary

| Component | Technology | Description |
|-----------|------------|-------------|
| **Backend** | Rust + Axum | High-performance REST API |
| **Database** | SQLite + sqlx | Embedded, zero-conf storage |
| **Frontend** | Vanilla JS | Dependency-free interaction |
| **Styling** | Tailwind CSS | Utility-first design |
| **Auth** | Hybrid | Session + JWT (Google/Admin) |

### Architecture Type
**Multi-Part Monolith**
- **Backend**: API Server + Static File Host
- **Frontend**: Client-side logic served statically

### Repository Structure
- `src/`: Backend logic
- `static/`: Frontend code
- `docs/`: System documentation
- `scripts/`: Maintenance utilities

## Detailed Documentation
- [Backend Architecture](./architecture-backend.md)
- [Frontend Architecture](./architecture-frontend.md)
- [Integration Architecture](./integration-architecture.md)
- [API Contracts](./api-contracts-backend.md)
- [Data Models](./data-models-backend.md)
- [Development Guide](./development-guide.md)
- [Deployment Guide](./deployment-guide.md)
