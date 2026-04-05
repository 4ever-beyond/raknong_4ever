# 4ever & Beyond — Implementation Plan

> Last updated: April 2026

## Overview

A high-performance community platform built with **Rust** end-to-end:
- **Backend**: SpacetimeDB v2 (tables + reducers)
- **Frontend**: Dioxus 0.7 (WASM) + Tailwind CSS v4
- **Real-time**: WebSocket subscriptions via SpacetimeDB SDK

---

## Completed Milestones

### ✅ Phase 1: Backend Foundation
- [x] SpacetimeDB v2 module with 4 tables (`UserProfile`, `Event`, `EventQuestion`, `EventResponse`)
- [x] 7 reducers: `init`, `register_profile`, `submit_response`, `create_event`, `add_event_question`, `toggle_verification`, `deactivate_event`, `delete_event_response`
- [x] Auto-seeding of default event + questions on `init`
- [x] Passcode security gate for RSVP
- [x] Duplicate RSVP prevention
- [x] Progressive profiling (5 required fields: nickname, year, phone, IG, Line)

### ✅ Phase 2: Frontend UI (Demo Mode)
- [x] Dioxus 0.7 web app with 5 views (Loading → Onboarding → EventView → Submitted)
- [x] Tailwind CSS v4 dark theme with glassmorphism
- [x] Thai/English i18n toggle
- [x] Dynamic RSVP form rendering (text/select/radio)
- [x] Responsive mobile-first design
- [x] Error handling with dismissible banners

### ✅ Phase 3: Database Connection & Testing
- [x] SpacetimeDB server running on `localhost:3000`
- [x] Module build (WASM) + publish verified
- [x] All 7 reducers tested via CLI (`spacetime call`)
- [x] Passcode validation confirmed (wrong passcode → blocked)
- [x] Duplicate RSVP prevention confirmed

### ✅ Phase 4: SpacetimeDB SDK Integration
- [x] `spacetimedb-sdk = "2.1.0"` added to `client/Cargo.toml`
- [x] Auto-generated module bindings regenerated (`spacetime generate`)
- [x] Real WebSocket connection from frontend to SpacetimeDB
- [x] Live table subscriptions (`event`, `event_question`, `user_profile`, `event_response`)
- [x] `register_profile` reducer wired to onboarding form
- [x] `submit_response` reducer wired to RSVP form
- [x] Removed `seed_demo_data()` — real data only
- [x] Reactive signal updates via SpacetimeDB callbacks

### ✅ Phase 5: Admin Dashboard
- [x] New `ViewState::Admin` route
- [x] Stats bar: Total Users, Total Responses, Verified, Pending
- [x] Users tab: Full user directory with toggle verification
- [x] Responses tab: All RSVPs with answer details + delete
- [x] Admin nav button in header (visible to all registered users)
- [x] i18n strings for admin dashboard (Thai + English)
- [x] `toggle_verification` reducer wired
- [x] `delete_event_response` reducer wired

---

## In Progress

### 🔄 Phase 6: Production Readiness
- [ ] Admin authentication (restrict admin access to verified users only)
- [ ] Profile editing (add/edit fields after initial registration)
- [ ] Fix any remaining WASM compilation warnings
- [ ] Production build optimization (`dx build --platform web --release`)

---

## Roadmap

### 🔴 Priority — Stabilization
1. End-to-end testing with multiple browser clients
2. Error recovery for WebSocket disconnects
3. Loading states for reducer calls (spinner while `register_profile` processes)
4. Admin-only guard (check `is_verified` before showing admin panel)

### 🟡 Polish
- Multi-event support with event listing page
- Event creation from admin dashboard (call `create_event` + `add_event_question`)
- Profile editing page
- Notification/toast system for reducer errors
- Better mobile responsiveness for admin tables

### 🟢 Future
- Deploy to SpacetimeDB MainCloud
- Real authentication beyond Identity-based approach
- Push notification system for new events
- QR code check-in at events
- Mobile app build (`dx serve --platform mobile`)
- Photo gallery for past events
- Line/Discord bot integration for notifications

---

## Technical Debt

| Item | Priority | Notes |
|------|----------|-------|
| Admin auth | High | Currently any registered user can access admin |
| WebSocket reconnect | High | No auto-reconnect on disconnect |
| Loading spinners | Medium | Reducer calls have no feedback |
| i18n coverage | Low | Some admin strings may be missing edge cases |
| Error messages | Medium | Server reducer errors not surfaced to UI |

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     Browser (WASM)                            │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │           Dioxus 0.7 Frontend (client/)                 │ │
│  │                                                          │ │
│  │  Views: Loading · Onboarding · EventView · Submitted     │ │
│  │         · Admin Dashboard (Users + Responses)            │ │
│  │                                                          │ │
│  │  State: Dioxus Signals ← SpacetimeDB SDK callbacks       │ │
│  └────────────────┬────────────────────────────────────────┘ │
│                   │ WebSocket (spacetimedb-sdk 2.1.0)         │
└───────────────────┼──────────────────────────────────────────┘
                    │
┌───────────────────▼──────────────────────────────────────────┐
│          SpacetimeDB Server v2.1.0 (localhost:3000)           │
│                                                              │
│  Tables:  UserProfile · Event · EventQuestion                │
│           EventResponse                                      │
│                                                              │
│  Reducers:  init · register_profile · submit_response        │
│             create_event · add_event_question                │
│             toggle_verification · deactivate_event            │
│             delete_event_response                            │
└──────────────────────────────────────────────────────────────┘
```

---

## Database Schema

### UserProfile
| Column | Type | Notes |
|--------|------|-------|
| identity | Identity (PK) | SpacetimeDB cryptographic identity |
| nickname | String | Display name |
| entry_year | String | "Year 1"–"Year 5+" or "Alumni" |
| phone | String | Phone number |
| instagram | String | Instagram handle |
| line_id | String | Line ID |
| is_verified | bool | Admin-toggled |
| created_at | Timestamp | Auto-set |

### Event
| Column | Type | Notes |
|--------|------|-------|
| id | u64 (PK, auto) | Auto-incremented |
| title | String | Event name |
| description | String | Event details |
| event_date | String | Human-readable date |
| priority | u32 | Higher = shown first |
| is_active | bool | Soft delete flag |
| passcode | String | Shared access code |
| created_at | Timestamp | Auto-set |

### EventQuestion
| Column | Type | Notes |
|--------|------|-------|
| id | u64 (PK, auto) | Auto-incremented |
| event_id | u64 | FK → Event |
| label | String | Question text |
| field_type | String | "text", "select", or "radio" |
| options | Option<String> | JSON array for select/radio |
| is_required | bool | Mandatory answer |

### EventResponse
| Column | Type | Notes |
|--------|------|-------|
| id | u64 (PK, auto) | Auto-incremented |
| event_id | u64 | FK → Event |
| user_identity | Identity | FK → UserProfile |
| answers | String | JSON {question_id: answer} |
| submitted_at | Timestamp | Auto-set |

---

## Development Setup

```bash
# Terminal 1: Start SpacetimeDB
spacetime start --in-memory

# Terminal 2: Publish the module
cd spacetimedb/ && spacetime publish community-platform --delete-data -y -s http://localhost:3000

# Terminal 3: Run the frontend
cd client/ && dx serve
```

Open `http://127.0.0.1:8080` in your browser.

---

## Git Commit Strategy

```
Phase 4: git add client/ && git commit -m "feat(client): wire SpacetimeDB SDK with real WebSocket connection"
Phase 5: git add client/ && git commit -m "feat(client): add admin dashboard with user management and response viewer"
Phase 6: git add . && git commit -m "docs: add implementation plan and update README"
```
