# Project: 4ever & Beyond - Universal Community Platform
**A High-Performance, Dynamic Survey & Identity System built with Rust.**

## 1. Vision & Core Logic
- **Progressive Profiling:** Fast onboarding. Users provide only essential info first to RSVP quickly. They can complete their full profile later.
- **Identity-First:** Strict use of SpacetimeDB's cryptographic `Identity` as the Primary Key to prevent collisions (same nickname/year) and track users persistently.
- **Security & Anti-Spam (MVP):** Use a simple "Shared Passcode" for events to prevent unauthorized external access. Admin verification is done post-registration (Role-Based Access).

## 2. Technical Stack
- **Backend:** SpacetimeDB
- **Frontend:** Dioxus **0.7** (Rust WASM)
- **Styling:** Tailwind CSS

## 3. Data Architecture (SpacetimeDB)
### Tables:
- **UserProfile:** - `identity: Identity` (Primary Key)
  - `nickname: String` (Required)
  - `entry_year: String` (Required - e.g., "Year 1", "Year 2", "Alumni")
  - `contact_channel: String` (Required - e.g., Line ID or IG)
  - `student_id: Option<String>` (Optional - prevents Alumni drop-off)
  - `is_verified: bool` (Default: false. Admin can toggle this later)
- **Event:** `id`, `title`, `description`, `event_date`, `priority`, `is_active`, `passcode: String` (Anti-spam measure).
- **EventQuestion:** `id`, `event_id`, `label`, `field_type`, `options`, `is_required`.
- **EventResponse:** `event_id`, `user_identity`, `answers`.

## 4. User Flow (Speed & Security)
1. **Authentication:** SpacetimeDB SDK automatically assigns/retrieves the user's `Identity`.
2. **Fast Onboarding:** If `UserProfile` doesn't exist, show a minimal form (Nickname, Year/Alumni, Contact). Save as `is_verified: false`.
3. **Dynamic Form & RSVP:** - User sees the highest-priority active event.
   - User answers dynamic questions (e.g., Menu Selection).
   - **Security Gate:** User must enter the Event's `passcode` (e.g., distributed in the Line group) to successfully submit the `EventResponse`.

---

## 🤖 AI Instructions (Prompt for Code Generation)

"I am building a scalable Community Platform MVP using **Rust (Dioxus 0.7 + SpacetimeDB)**. The deadline is tomorrow noon. Please act as a Lead Rust Engineer and generate the code.

### 1. SpacetimeDB Module (`lib.rs`)
- Implement the schemas defined above (`UserProfile`, `Event`, `EventQuestion`, `EventResponse`). Use `Identity` strictly for relationships.
- Create a `register_profile` Reducer for fast onboarding.
- Create a `submit_response` Reducer that takes `(event_id, passcode, answers)`. The Reducer MUST verify that the provided `passcode` matches the event's passcode before saving to prevent spam.

### 2. Dioxus 0.7 Frontend Logic
- Use **Dioxus 0.7** syntax (Signals, `rsx!`, updated hooks).
- Connect to SpacetimeDB.
- **View States:**
  - `Loading`: Checking Identity.
  - `Onboarding`: Show short form (Nickname, Year, Contact). Skip `student_id` to reduce friction.
  - `Event View`: Display dynamic questions + a required field for the 'Event Passcode'.
- Ensure the UI is mobile-first, utilizing Tailwind CSS for a clean, modern look.

Please output complete, compilable Rust code for both the module and the frontend components."
