# 4ever & Beyond — Community Platform

A high-performance, dynamic RSVP and identity management system built entirely in **Rust**. Designed for university communities to manage events, collect responses, and verify members — with a focus on fast onboarding and mobile-first UX.

> **Status:** **Live** — Dioxus 0.7 fullstack app with PostgreSQL backend, deployed via `dx serve`.

---

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Backend | [Dioxus Server Functions](https://dioxuslabs.com/) + [SQLx](https://github.com/launchbadge/sqlx) | 0.7 / 0.8 |
| Frontend | [Dioxus](https://dioxuslabs.com/) (WASM) | 0.7 |
| Database | [PostgreSQL](https://www.postgresql.org/) | 15+ |
| Styling | [Tailwind CSS](https://tailwindcss.com/) | v4 (via Dioxus CLI) |
| Language | Rust | 2021 Edition |

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│                  Browser (WASM)                   │
│  ┌─────────────────────────────────────────────┐ │
│  │        Dioxus 0.7 Frontend (client/)        │ │
│  │                                              │ │
│  │  Loading → Onboarding → EventView → Submitted│ │
│  │         · Admin Dashboard (Users + Responses) │ │
│  │                                              │ │
│  │  • Progressive Profiling (4 required fields) │ │
│  │  • Dynamic RSVP Forms (text/select)          │ │
│  │  • Passcode Security Gate                    │ │
│  └──────────────┬──────────────────────────────┘ │
│                 │ HTTP POST (server functions)    │
└─────────────────┼────────────────────────────────┘
                  │
┌─────────────────▼────────────────────────────────┐
│        Dioxus Server (same binary, server build)  │
│                                                   │
│  #[server] Functions:                             │
│    register_profile · submit_response             │
│    get_events · get_user_profile                  │
│    get_all_users · get_all_responses              │
│    toggle_verification · delete_event_response    │
│    check_existing_response                        │
│                                                   │
│  Auto-registered at POST /api/<name>              │
└─────────────────┬────────────────────────────────┘
                  │ sqlx::PgPool
┌─────────────────▼────────────────────────────────┐
│              PostgreSQL Database                  │
│                                                   │
│  Tables:  user_profile · event                    │
│           event_question · event_response         │
└───────────────────────────────────────────────────┘
```

### Key Design Decisions

- **Session-based Identity** — Uses `crypto.randomUUID()` stored in localStorage as session ID. No login system required; users are identified by their browser session.
- **Progressive Profiling** — Users provide nickname, entry year, phone, Instagram, and Line ID to get started. All fields required for a complete profile.
- **Passcode Security** — Each event has a shared passcode that must be entered before an RSVP is accepted. This prevents unauthorized external sign-ups.
- **Dynamic Forms** — Event questions are stored in the database and rendered client-side as text inputs or select dropdowns.
- **Single Binary Fullstack** — The `#[server]` macro compiles function bodies only on the server build, and auto-generates HTTP client stubs for the WASM build.

---

## Project Structure

```
4ever/
├── README.md                              ← You are here
├── global_community_platform_rust.md      ← Architecture specification
├── implement_plan.md                      ← Implementation plan
├── .gitignore
│
├── spacetimedb/                           ← Legacy (no longer used, safe to delete)
│
└── client/                                ← Fullstack Dioxus 0.7 app
    ├── Cargo.toml                         ← dioxus 0.7 + sqlx + chrono (feature-gated)
    ├── Dioxus.toml                        ← Dioxus CLI configuration
    ├── assets/
    │   ├── main.css                       ← Custom styles (scrollbar, animations)
    │   └── tailwind.css                   ← @import "tailwindcss"
    └── src/
        ├── main.rs                        ← UI: 5 views + Admin + server entry point
        ├── backend.rs                     ← Shared types + #[server] functions + DB pool
        └── i18n.rs                        ← Thai/English locale strings
```

---

## Prerequisites

| Tool | Install Command | Notes |
|------|----------------|-------|
| **Rust** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | rustup recommended |
| **wasm32-unknown-unknown** | `rustup target add wasm32-unknown-unknown` | Required for WASM build |
| **Dioxus CLI** | `curl -sSL https://dioxus.dev/install.sh \| bash` | v0.7.4+ (prebuilt binary recommended) |
| **PostgreSQL** | `brew install postgresql@15` (macOS) | v15+ recommended |

> **Tip:** Installing `dioxus-cli` via `cargo install` compiles 774 dependencies and may OOM on machines with <16 GB RAM. Use the prebuilt binary instead.

---

## Getting Started

### 1. Start PostgreSQL

```bash
# macOS (Homebrew)
brew services start postgresql@15

# Linux (systemd)
sudo systemctl start postgresql
```

### 2. Create the Database

```bash
createdb forever
```

The application will automatically create tables and seed default data on first start.

### 3. Run the App

```bash
cd client/

# Development server with hot reload
dx serve --platform web

# Or with a custom database URL
DATABASE_URL="postgres://user@localhost:5432/forever" dx serve --platform web
```

Open `http://localhost:8080` in your browser.

### 4. (Optional) Set DATABASE_URL permanently

```bash
# Create .env in the client directory
echo 'DATABASE_URL=postgres://localhost:5432/forever' > client/.env
```

The default connection string is `postgres://localhost:5432/forever` (uses current OS user, no password on local socket).

---

## Seeded Data

On first server start, `init_db()` automatically seeds:

- **Event:** "4EVER รวมตัวกินสเต็กเด็กอ้วน" (passcode: `4ever2026`)
  - Date: 08-04-2569
  - Location: ศาลายา ซอย 11
- **Question 1:** "เห็นข่าวการเรียกรวมตัวจากที่ไหนเอ่ย" (select: กลุ่มไลน์, อินสตาแกรม, เพื่อนบอก, Facebook, อื่นๆ)
- **Question 2:** "เมนูที่จะกินค่าาา" (select: สเต็กหมู/ไก่ S/M/L, สเต็กปลาแซลมอน, เมนูอื่นๆ)

---

## API Reference — Server Functions

All endpoints are registered as `POST /api/<function_name>`.

### `register_profile`
Creates a user profile.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | `String` | ✅ | Browser session UUID |
| `nickname` | `String` | ✅ | Display name |
| `entry_year` | `String` | ✅ | e.g. "2560" |
| `phone` | `String` | ✅ | Phone number |
| `instagram` | `String` | ✅ | Instagram handle |
| `line_id` | `String` | ✅ | LINE ID |

Guards: prevents duplicate session, validates all fields non-empty.

### `submit_response`
Submits an RSVP response with passcode verification.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `event_id` | `i64` | ✅ | Target event ID |
| `session_id` | `String` | ✅ | User's session UUID |
| `passcode` | `String` | ✅ | Must match event's stored passcode |
| `answers` | `String` | ✅ | JSON mapping question labels to answers |

Guards: verifies profile exists, event exists, event is active, passcode matches, no duplicate RSVP, non-empty answers.

### `get_events`
Returns all active events with their questions.

**Response:** `Vec<EventWithQuestions>` — array of event objects, each containing an `event` and `questions` array.

### `get_user_profile`
Returns a user profile by session ID.

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | `String` | User's session UUID |

**Response:** `Option<UserProfile>` — the profile if found, otherwise `null`.

### `get_all_users`
Returns all user profiles (admin).

**Response:** `Vec<UserProfile>`

### `get_all_responses`
Returns all RSVP responses (admin).

**Response:** `Vec<EventResponse>`

### `toggle_verification`
Toggles a user's `is_verified` status.

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | `String` | Target user's session UUID |

### `delete_event_response`
Deletes a specific RSVP response.

| Parameter | Type | Description |
|-----------|------|-------------|
| `response_id` | `i64` | Response ID to delete |

### `check_existing_response`
Checks if a user already submitted an RSVP for an event.

| Parameter | Type | Description |
|-----------|------|-------------|
| `event_id` | `i64` | Target event |
| `session_id` | `String` | User's session UUID |

**Response:** `bool` — `true` if response exists.

---

## Database Schema

```sql
-- Auto-created by init_db() on first server start

CREATE TABLE user_profile (
    id            SERIAL PRIMARY KEY,
    session_id    TEXT UNIQUE NOT NULL,
    nickname      TEXT NOT NULL,
    entry_year    TEXT NOT NULL,
    phone         TEXT NOT NULL,
    instagram     TEXT NOT NULL,
    line_id       TEXT NOT NULL,
    is_verified   BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE event (
    id            SERIAL PRIMARY KEY,
    title         TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    event_date    TEXT NOT NULL DEFAULT '',
    priority      INTEGER NOT NULL DEFAULT 0,
    is_active     BOOLEAN NOT NULL DEFAULT TRUE,
    passcode      TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE event_question (
    id            SERIAL PRIMARY KEY,
    event_id      INTEGER NOT NULL REFERENCES event(id),
    label         TEXT NOT NULL,
    field_type    TEXT NOT NULL DEFAULT 'text',
    options       TEXT,
    is_required   BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE event_response (
    id            SERIAL PRIMARY KEY,
    event_id      INTEGER NOT NULL REFERENCES event(id),
    session_id    TEXT NOT NULL,
    answers       TEXT NOT NULL,
    submitted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## Feature Flags

```toml
[features]
default = []
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]
server = ["dioxus/server", "dep:sqlx", "dep:chrono"]
```

| Flag | Effect |
|------|--------|
| `web` | Builds WASM client for browsers |
| `server` | Builds server binary with PostgreSQL + all server function bodies |
| `desktop` | Desktop app (WebView) |
| `mobile` | Mobile app (iOS/Android WebView) |

When using `dx serve --platform web`, the CLI auto-detects the `fullstack` feature and builds both the server binary (with `server` feature) and the WASM client (with `web` feature).

---

## Testing API Endpoints

```bash
# Get all active events
curl -X POST http://localhost:8080/api/get_events

# Register a profile
curl -X POST http://localhost:8080/api/register_profile \
  -H "Content-Type: application/json" \
  -d '{"session_id":"test-001","nickname":"Alice","entry_year":"2560","phone":"0812345678","instagram":"@alice","line_id":"alice_line"}'

# Submit RSVP (correct passcode)
curl -X POST http://localhost:8080/api/submit_response \
  -H "Content-Type: application/json" \
  -d '{"event_id":1,"session_id":"test-001","passcode":"4ever2026","answers":"{\"เห็นข่าว\":\"กลุ่มไลน์\",\"เมนู\":\"สเต็กหมู M\"}"}'

# Check if already submitted
curl -X POST http://localhost:8080/api/check_existing_response \
  -H "Content-Type: application/json" \
  -d '{"event_id":1,"session_id":"test-001"}'

# Get all users (admin)
curl -X POST http://localhost:8080/api/get_all_users

# Toggle verification
curl -X POST http://localhost:8080/api/toggle_verification \
  -H "Content-Type: application/json" \
  -d '{"session_id":"test-001"}'
```

---

## Current State: Live

The frontend communicates with the server via auto-generated HTTP client stubs from the `#[server]` macro.

| User Action | What Happens | Persisted? |
|-------------|-------------|:----------:|
| Fill onboarding form | Calls `register_profile` → profile stored in PostgreSQL | ✅ |
| Submit RSVP with passcode | Calls `submit_response` → server validates passcode | ✅ |
| Events & questions | Fetched via `get_events` server function | ✅ |
| Admin dashboard | Fetches users + responses via server functions | ✅ |
| Toggle verification | Calls `toggle_verification` | ✅ |
| Delete response | Calls `delete_event_response` | ✅ |

---

## Roadmap

See [`implement_plan.md`](./implement_plan.md) for the full implementation plan with phase tracking.

### 🔴 Priority — Stabilization
1. End-to-end testing with multiple browser clients
2. Admin authentication (restrict to verified users)
3. Loading spinners for server function calls
4. Error handling UI for database connection failures

### 🟡 Polish
- Multi-event support with event listing page
- Event creation from admin dashboard
- Profile editing page
- Notification/toast system for errors
- Better mobile responsiveness for admin tables
- `use_server_future` for data fetching with SSR support

### 🟢 Future
- Deploy to production (Docker + PostgreSQL)
- Real authentication (OAuth / Line Login)
- Push notification system for new events
- QR code check-in at events
- Mobile app build (`dx serve --platform mobile`)
- Photo gallery for past events
- Line/Discord bot integration
- Server-sent events for real-time admin dashboard

---

## Migration History

This project was originally built with **SpacetimeDB** (WebSocket-based real-time database). It was migrated to **Dioxus Fullstack + PostgreSQL** to resolve persistent WebSocket disconnection issues and to align with the Dioxus 0.7 recommended architecture.

| Aspect | Before (SpacetimeDB) | After (PostgreSQL) |
|--------|---------------------|-------------------|
| Connection | WebSocket (disconnects) | HTTP POST (stateless) |
| Backend | SpacetimeDB reducers | Dioxus `#[server]` functions |
| Database | In-memory (SpacetimeDB) | PostgreSQL (persistent) |
| Identity | Cryptographic `Identity` | Session UUID (localStorage) |
| Real-time | Table subscriptions | Polling (future: SSE) |
| Module bindings | Auto-generated SDK | Auto-generated HTTP stubs |

---

## Development Notes

### Dioxus 0.7 Fullstack Conventions

- `#[server(endpoint = "name")]` — defines a server function; the macro auto-generates client stubs for WASM
- Server function bodies are only compiled when the `server` feature is active
- `dioxus::serve(|| async { Ok(dioxus::server::router(App)) })` — launches the fullstack server
- `dioxus::launch(App)` — launches the client-only app
- All `sqlx` / `chrono` dependencies are feature-gated behind `server`
- `SyncSignal<T>` provides interior mutability that works on both server and client

### PostgreSQL Connection

- Connection pool is thread-local (`PgPool` behind `RefCell`)
- Tables are auto-created via `CREATE TABLE IF NOT EXISTS` on server start
- Seed data is only inserted when the `event` table is empty
- No migrations framework — schema is managed inline in `init_db()`

---

## License

Private project — all rights reserved.