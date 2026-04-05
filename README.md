# 4ever & Beyond — Community Platform

A high-performance, dynamic RSVP and identity management system built entirely in **Rust**. Designed for university communities to manage events, collect responses, and verify members — with a focus on fast onboarding and mobile-first UX.

> **Status:** MVP Demo Mode — UI fully functional, SpacetimeDB backend wired (server-side), frontend SDK integration pending.

---

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Backend | [SpacetimeDB](https://spacetimedb.com/) | v2 |
| Frontend | [Dioxus](https://dioxuslabs.com/) | 0.7 |
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
│  │                                              │ │
│  │  • Progressive Profiling (3 required fields) │ │
│  │  • Dynamic RSVP Forms (text/select/radio)    │ │
│  │  • Passcode Security Gate                    │ │
│  └──────────────┬──────────────────────────────┘ │
│                 │ WebSocket (TODO)                │
└─────────────────┼────────────────────────────────┘
                  │
┌─────────────────▼────────────────────────────────┐
│         SpacetimeDB Server (spacetimedb/)         │
│                                                   │
│  Tables:  UserProfile · Event · EventQuestion     │
│           EventResponse                           │
│                                                   │
│  Reducers:  init · register_profile               │
│             submit_response · toggle_verification  │
│             create_event · add_event_question      │
│             deactivate_event · delete_event_response│
└───────────────────────────────────────────────────┘
```

### Key Design Decisions

- **Progressive Profiling** — Users provide only a nickname, year, and contact handle to get started. Student ID is optional to prevent alumni drop-off.
- **Identity-First** — SpacetimeDB's cryptographic `Identity` serves as the primary key for user profiles, preventing collisions and enabling persistent sessions.
- **Passcode Security** — Each event has a shared passcode that must be entered before an RSVP is accepted. This prevents unauthorized external sign-ups.
- **Dynamic Forms** — Event questions are stored in the database and rendered client-side as text inputs, select dropdowns, or radio button groups.

---

## Project Structure

```
4ever/
├── README.md                              ← You are here
├── global_community_platform_rust.md      ← Architecture specification
├── .gitignore
│
├── spacetimedb/                           ← Backend (SpacetimeDB v2 module)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                         ← 4 tables + 7 reducers + init seeder
│
└── client/                                ← Frontend (Dioxus 0.7 web app)
    ├── Cargo.toml                         ← dioxus = "0.7.1", with feature flags
    ├── Cargo.lock                         ← Pinned dependencies
    ├── Dioxus.toml                        ← Dioxus CLI configuration
    ├── tailwind.css                       ← Root-level Tailwind entry
    ├── assets/
    │   ├── main.css                       ← Custom styles (scrollbar, animations)
    │   └── tailwind.css                   ← @import "tailwindcss"
    └── src/
        ├── main.rs                        ← Full UI: 5 views + App state machine
        ├── i18n.rs                        ← Thai/English locale strings
        └── module_bindings/               ← Auto-generated SpacetimeDB SDK types
            ├── mod.rs
            ├── event_type.rs
            ├── event_table.rs
            ├── event_question_type.rs
            ├── event_question_table.rs
            ├── event_response_type.rs
            ├── event_response_table.rs
            ├── user_profile_type.rs
            ├── user_profile_table.rs
            ├── register_profile_reducer.rs
            ├── submit_response_reducer.rs
            ├── create_event_reducer.rs
            ├── add_event_question_reducer.rs
            ├── deactivate_event_reducer.rs
            ├── delete_event_response_reducer.rs
            └── toggle_verification_reducer.rs
```

---

## Prerequisites

| Tool | Install Command | Notes |
|------|----------------|-------|
| **Rust** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | rustup recommended |
| **wasm32-unknown-unknown** | `rustup target add wasm32-unknown-unknown` | Required for both modules |
| **SpacetimeDB CLI** | `curl -sSf https://install.spacetimedb.com \| bash` | v2.1.0+ |
| **Dioxus CLI** | `curl -sSL https://dioxus.dev/install.sh \| bash` | v0.7.4+ (prebuilt binary recommended) |

> **Tip:** Installing `dioxus-cli` via `cargo install` compiles 774 dependencies and may OOM on machines with <16 GB RAM. Use the prebuilt binary instead.

---

## Getting Started

### 1. Start SpacetimeDB (Backend)

```bash
# Start the in-memory database server
spacetime start --in-memory
```

This starts the server at `http://localhost:3000`.

### 2. Build & Publish the Server Module

```bash
cd spacetimedb/

# Build to WASM
spacetime build --debug

# Publish to local server
spacetime publish community-platform -y -s http://localhost:3000
```

The `init` reducer automatically seeds:
- **Event:** "Welcome Dinner 2025" (passcode: `4ever2025`)
- **3 Questions:** Menu Selection (select), Dietary restrictions (text), Plus-one (radio)

### 3. Verify the Backend

```bash
# Check seeded data
spacetime sql -s http://localhost:3000 community-platform "SELECT * FROM event"
spacetime sql -s http://localhost:3000 community-platform "SELECT * FROM event_question"

# Test registration
spacetime call -s http://localhost:3000 community-platform register_profile '["Alice", "Year 2", "@alice_ig", "66101234"]'

# Test RSVP (correct passcode — should succeed)
spacetime call -s http://localhost:3000 community-platform submit_response '[1, "4ever2025", "{\"1\":\"Standard\",\"2\":\"None\",\"3\":\"No\"}"]'

# Test RSVP (wrong passcode — should be BLOCKED)
spacetime call -s http://localhost:3000 community-platform submit_response '[1, "wrong", "{}"]'
```

### 4. Run the Frontend

```bash
cd client/

# Development server with hot reload
dx serve

# Or build for production
dx build --platform web --release
```

Open `http://127.0.0.1:8080` in your browser.

---

## API Reference — SpacetimeDB Reducers

### `init` *(automatic)*
Called when the module is first published. Seeds the default event and questions.

### `register_profile`
Creates a user profile with minimal required fields.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `nickname` | `String` | ✅ | Display name |
| `entry_year` | `String` | ✅ | "Year 1"–"Year 5+" or "Alumni" |
| `contact_channel` | `String` | ✅ | Line ID or Instagram handle |
| `student_id` | `Option<String>` | ❌ | e.g. "66101234" |

Guards: prevents duplicate registration (checked by `Identity`), validates non-empty required fields.

### `submit_response`
Submits an RSVP response with passcode verification.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `event_id` | `u64` | ✅ | Target event ID |
| `passcode` | `String` | ✅ | Must match event's stored passcode |
| `answers` | `String` | ✅ | JSON mapping question IDs to answers |

Guards: verifies profile exists, event exists, event is active, passcode matches, no duplicate RSVP, non-empty answers.

### `create_event`
Creates a new community event.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | `String` | ✅ | Event name |
| `description` | `String` | ❌ | Event details |
| `event_date` | `String` | ✅ | e.g. "2025-06-15" |
| `priority` | `u32` | ❌ | Higher = shown first |
| `passcode` | `String` | ✅ | Shared access code |

### `add_event_question`
Attaches a dynamic question to an event.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `event_id` | `u64` | ✅ | Target event |
| `label` | `String` | ✅ | Question text |
| `field_type` | `String` | ✅ | "text", "select", or "radio" |
| `options` | `Option<String>` | ❌ | JSON array for select/radio choices |
| `is_required` | `bool` | ✅ | Whether answer is mandatory |

### `toggle_verification`
Toggles a user's `is_verified` status (admin utility).

| Parameter | Type | Description |
|-----------|------|-------------|
| `target_identity` | `Identity` | The user to toggle |

### `deactivate_event`
Soft-deletes an event by setting `is_active = false`.

| Parameter | Type | Description |
|-----------|------|-------------|
| `event_id` | `u64` | Event to deactivate |

### `delete_event_response`
Removes a specific RSVP response (admin utility).

| Parameter | Type | Description |
|-----------|------|-------------|
| `response_id` | `u64` | Response to delete |

---

## Current State: Demo Mode

The frontend is **fully functional in the browser** but operates in **demo mode** — all data lives in Dioxus reactive signals (browser memory). No data is persisted to SpacetimeDB from the frontend.

| User Action | What Happens | Persisted? |
|-------------|-------------|:----------:|
| Fill onboarding form | Creates `UserProfile` in local signal | ❌ |
| Submit RSVP with passcode | Validates passcode locally, shows confirmation | ❌ |
| Events & questions | Hardcoded via `seed_demo_data()` | ❌ |

All locations requiring SpacetimeDB SDK integration are marked with `// TODO:` comments in `client/src/main.rs`.

---

## Roadmap

### 🔴 Priority — Wire SpacetimeDB SDK

1. Add `spacetimedb-sdk` dependency to `client/Cargo.toml`
2. Open WebSocket connection to `ws://localhost:3000`
3. Subscribe to tables (`event`, `event_question`, `user_profile`, `event_response`)
4. Replace `seed_demo_data()` with real subscription callbacks
5. Replace onboarding `spawn()` with `conn.call_reducer("register_profile", ...)`
6. Replace RSVP submit with `conn.call_reducer("submit_response", ...)`

### 🟡 Polish

- Admin panel (toggle verification, create events via UI)
- Profile editing (add `student_id` after initial registration)
- Production build optimization (`dx build --platform web --release`)
- Fix `LoadingView` snake_case warning (rename to `loading_view`)

### 🟢 Future

- Deploy to SpacetimeDB MainCloud for production
- Real authentication beyond Identity-based approach
- Multi-event support with event listing page
- Push notification system for new events
- i18n toggle (Thai/English strings already exist in `i18n.rs`)
- Mobile app build via `dx serve --platform mobile`

---

## Git Commit Strategy

```
Step 1: git add .gitignore global_community_platform_rust.md
        git commit -m "docs: add project spec and gitignore"

Step 2: git add spacetimedb/
        git commit -m "feat(server): SpacetimeDB v2 module with 4 tables and 7 reducers"

Step 3: git add client/
        git commit -m "feat(client): Dioxus 0.7 frontend with demo-mode UI"

Step 4: git add README.md
        git commit -m "docs: add comprehensive README with setup instructions"

Step 5: (after review)
        git remote add origin https://github.com/<user>/4ever.git
        git push -u origin main
```

---

## Development Notes

### SpacetimeDB v2 Syntax (Important)

This project uses SpacetimeDB **v2**, which has breaking syntax differences from v1:

```rust
// v1 (BROKEN)                          // v2 (CORRECT — used in this project)
#[table(public, name = "user_profile")]  #[spacetimedb::table(accessor = user_profile, public)]
ctx.sender                               ctx.sender()
ctx.timestamp()                          ctx.timestamp        // field, not method
// Also: use spacetimedb::Table; must be in scope for .insert(), .iter()
```

### Dioxus 0.7 Conventions

- No `index.html` — Dioxus CLI auto-generates it
- Tailwind via `@import "tailwindcss"` in CSS files, not CDN scripts
- Assets in `assets/` directory, loaded with `asset!()` macro
- Entry point: `dioxus::launch(App)` not `launch(App)`
- Minimal `Dioxus.toml` — only `[application]`, `[web.app]`, `[web.resource]` sections

---

## License

Private project — all rights reserved.