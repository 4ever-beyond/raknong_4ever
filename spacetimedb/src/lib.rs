//! 4ever & Beyond — SpacetimeDB Server Module
//!
//! A high-performance Community Platform backend built with SpacetimeDB v2.
//! Implements progressive profiling, dynamic RSVP forms, and passcode security.

use spacetimedb::{Identity, Table};

// =============================================================================
// TABLE DEFINITIONS
// =============================================================================

/// User profile created during "Fast Onboarding".
/// Uses SpacetimeDB's cryptographic `Identity` as the primary key
/// to prevent collisions and track users persistently.
#[spacetimedb::table(accessor = user_profile, public)]
pub struct UserProfile {
    #[primary_key]
    pub identity: Identity,
    /// Display name (required)
    pub nickname: String,
    /// Academic year or "Alumni" (required)
    pub entry_year: String,
    /// Line ID or Instagram handle (required)
    pub contact_channel: String,
    /// Optional — prevents Alumni drop-off
    pub student_id: Option<String>,
    /// Admin can toggle this post-registration
    pub is_verified: bool,
    pub created_at: spacetimedb::Timestamp,
}

/// A community event (e.g., "Welcome Dinner 2025").
#[spacetimedb::table(accessor = event, public)]
pub struct Event {
    #[auto_inc]
    #[primary_key]
    pub id: u64,
    pub title: String,
    pub description: String,
    pub event_date: String,
    /// Higher number = shown first
    pub priority: u32,
    pub is_active: bool,
    /// Shared passcode to prevent unauthorized external access
    pub passcode: String,
    pub created_at: spacetimedb::Timestamp,
}

/// Dynamic questions attached to an event (e.g., "Menu Selection").
#[spacetimedb::table(accessor = event_question, public)]
pub struct EventQuestion {
    #[auto_inc]
    #[primary_key]
    pub id: u64,
    pub event_id: u64,
    /// Question label displayed to the user
    pub label: String,
    /// "text", "select", or "radio"
    pub field_type: String,
    /// JSON array of options for select/radio types
    pub options: Option<String>,
    pub is_required: bool,
}

/// A user's RSVP response to a specific event.
#[spacetimedb::table(accessor = event_response, public)]
pub struct EventResponse {
    #[auto_inc]
    #[primary_key]
    pub id: u64,
    pub event_id: u64,
    pub user_identity: Identity,
    /// JSON string mapping question IDs to answers
    pub answers: String,
    pub submitted_at: spacetimedb::Timestamp,
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Called when the module is first published.
/// Seeds a default event and questions so the platform is usable immediately.
#[spacetimedb::reducer(init)]
pub fn init(ctx: &spacetimedb::ReducerContext) {
    log::info!("4ever & Beyond — Module initialized.");

    // Seed a default active event
    ctx.db.event().insert(Event {
        id: 0,
        title: "Welcome Dinner 2025".to_string(),
        description: "Join us for the annual welcome dinner! Please select your menu preference and let us know about dietary restrictions.".to_string(),
        event_date: "2025-06-15".to_string(),
        priority: 10,
        is_active: true,
        passcode: "4ever2025".to_string(),
        created_at: ctx.timestamp,
    });

    // Seed dynamic questions for the default event (id will be 1 after auto-inc)
    ctx.db.event_question().insert(EventQuestion {
        id: 0,
        event_id: 1,
        label: "Menu Selection".to_string(),
        field_type: "select".to_string(),
        options: Some(r#"["Standard","Vegetarian","Halal","Vegan"]"#.to_string()),
        is_required: true,
    });

    ctx.db.event_question().insert(EventQuestion {
        id: 0,
        event_id: 1,
        label: "Any dietary restrictions or allergies?".to_string(),
        field_type: "text".to_string(),
        options: None,
        is_required: false,
    });

    ctx.db.event_question().insert(EventQuestion {
        id: 0,
        event_id: 1,
        label: "Will you bring a plus-one?".to_string(),
        field_type: "radio".to_string(),
        options: Some(r#"["Yes","No"]"#.to_string()),
        is_required: true,
    });

    log::info!("Default event 'Welcome Dinner 2025' seeded with 3 questions.");
}

// =============================================================================
// REDUCERS — USER FLOWS
// =============================================================================

/// **Fast Onboarding**: Registers a minimal user profile.
///
/// Only requires `nickname`, `entry_year`, and `contact_channel`.
/// `student_id` is optional to reduce friction (especially for Alumni).
/// The profile is created with `is_verified: false` — admin toggles later.
#[spacetimedb::reducer]
pub fn register_profile(
    ctx: &spacetimedb::ReducerContext,
    nickname: String,
    entry_year: String,
    contact_channel: String,
    student_id: Option<String>,
) {
    let identity = ctx.sender();
    let table = ctx.db.user_profile();

    // Guard: Prevent duplicate registration
    if table.identity().find(&identity).is_some() {
        log::warn!(
            "[register_profile] User {:?} already has a profile. Skipping.",
            identity
        );
        return;
    }

    // Validate required fields
    if nickname.trim().is_empty() {
        log::error!(
            "[register_profile] Rejected: nickname is empty (caller: {:?}).",
            identity
        );
        return;
    }
    if entry_year.trim().is_empty() {
        log::error!(
            "[register_profile] Rejected: entry_year is empty (caller: {:?}).",
            identity
        );
        return;
    }
    if contact_channel.trim().is_empty() {
        log::error!(
            "[register_profile] Rejected: contact_channel is empty (caller: {:?}).",
            identity
        );
        return;
    }

    let profile = UserProfile {
        identity,
        nickname: nickname.trim().to_string(),
        entry_year: entry_year.trim().to_string(),
        contact_channel: contact_channel.trim().to_string(),
        student_id: student_id.and_then(|s| {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }),
        is_verified: false,
        created_at: ctx.timestamp,
    };

    table.insert(profile);
    log::info!(
        "[register_profile] Profile created for {:?} — nickname: '{}', year: '{}'.",
        identity,
        nickname.trim(),
        entry_year.trim()
    );
}

/// **Dynamic RSVP with Passcode Security**:
///
/// Submits an event response. The reducer verifies that the provided `passcode`
/// matches the event's stored passcode **before** saving — this prevents
/// unauthorized external access (spam protection).
///
/// Also prevents duplicate RSVPs from the same identity.
#[spacetimedb::reducer]
pub fn submit_response(
    ctx: &spacetimedb::ReducerContext,
    event_id: u64,
    passcode: String,
    answers: String,
) {
    let identity = ctx.sender();
    let user_table = ctx.db.user_profile();
    let event_table = ctx.db.event();
    let response_table = ctx.db.event_response();

    // Step 1: Verify user has a profile
    let user = match user_table.identity().find(&identity) {
        Some(p) => p,
        None => {
            log::error!(
                "[submit_response] Rejected: No profile for {:?}. Register first.",
                identity
            );
            return;
        }
    };

    // Step 2: Verify event exists
    let event = match event_table.id().find(&event_id) {
        Some(e) => e,
        None => {
            log::error!(
                "[submit_response] Rejected: Event id={} not found (caller: {:?}).",
                event_id,
                identity
            );
            return;
        }
    };

    // Step 3: Verify event is active
    if !event.is_active {
        log::error!(
            "[submit_response] Rejected: Event '{}' is no longer active (caller: {:?}).",
            event.title,
            identity
        );
        return;
    }

    // Step 4: SECURITY GATE — Verify passcode
    if passcode.trim() != event.passcode.trim() {
        log::warn!(
            "[submit_response] BLOCKED: Invalid passcode for '{}' by '{}' ({:?}).",
            event.title,
            user.nickname,
            identity
        );
        return;
    }

    // Step 5: Prevent duplicate responses
    let already_submitted = response_table
        .iter()
        .any(|r| r.event_id == event_id && r.user_identity == identity);

    if already_submitted {
        log::warn!(
            "[submit_response] Rejected: '{}' ({:?}) already RSVP'd to event id={}.",
            user.nickname,
            identity,
            event_id
        );
        return;
    }

    // Step 6: Validate answers JSON is not empty
    if answers.trim().is_empty() {
        log::error!(
            "[submit_response] Rejected: answers payload is empty (caller: {:?}).",
            identity
        );
        return;
    }

    // Step 7: Persist the response
    let new_response = EventResponse {
        id: 0, // auto-incremented
        event_id,
        user_identity: identity,
        answers: answers.trim().to_string(),
        submitted_at: ctx.timestamp,
    };

    response_table.insert(new_response);

    log::info!(
        "[submit_response] RSVP confirmed: '{}' ({:?}) -> '{}' (event id={}).",
        user.nickname,
        identity,
        event.title,
        event_id
    );
}

// =============================================================================
// REDUCERS — ADMIN UTILITIES
// =============================================================================

/// Toggle a user's verification status (Role-Based Access).
/// In production, add an admin check before proceeding.
#[spacetimedb::reducer]
pub fn toggle_verification(ctx: &spacetimedb::ReducerContext, target_identity: Identity) {
    let admin_identity = ctx.sender();
    let table = ctx.db.user_profile();

    // TODO: Verify admin_identity has admin privileges before allowing this.
    match table.identity().find(&target_identity) {
        Some(mut profile) => {
            let previous = profile.is_verified;
            profile.is_verified = !previous;
            table.identity().update(profile);

            log::info!(
                "[toggle_verification] User {:?} verification: {} -> {} (by {:?}).",
                target_identity,
                previous,
                !previous,
                admin_identity
            );
        }
        None => {
            log::error!(
                "[toggle_verification] Failed: User {:?} not found (called by {:?}).",
                target_identity,
                admin_identity
            );
        }
    }
}

/// Create a new community event.
#[spacetimedb::reducer]
pub fn create_event(
    ctx: &spacetimedb::ReducerContext,
    title: String,
    description: String,
    event_date: String,
    priority: u32,
    passcode: String,
) {
    let caller = ctx.sender();

    if title.trim().is_empty() {
        log::error!(
            "[create_event] Rejected: title is empty (caller: {:?}).",
            caller
        );
        return;
    }
    if passcode.trim().is_empty() {
        log::error!(
            "[create_event] Rejected: passcode is empty (caller: {:?}).",
            caller
        );
        return;
    }
    if event_date.trim().is_empty() {
        log::error!(
            "[create_event] Rejected: event_date is empty (caller: {:?}).",
            caller
        );
        return;
    }

    let new_event = Event {
        id: 0,
        title: title.trim().to_string(),
        description: description.trim().to_string(),
        event_date: event_date.trim().to_string(),
        priority,
        is_active: true,
        passcode: passcode.trim().to_string(),
        created_at: ctx.timestamp,
    };

    ctx.db.event().insert(new_event);

    log::info!(
        "[create_event] New event '{}' created (priority={}, caller: {:?}).",
        title.trim(),
        priority,
        caller
    );
}

/// Add a dynamic question to an event.
#[spacetimedb::reducer]
pub fn add_event_question(
    ctx: &spacetimedb::ReducerContext,
    event_id: u64,
    label: String,
    field_type: String,
    options: Option<String>,
    is_required: bool,
) {
    if label.trim().is_empty() {
        log::error!("[add_event_question] Rejected: label is empty.");
        return;
    }

    // Verify the event exists
    if ctx.db.event().id().find(&event_id).is_none() {
        log::error!(
            "[add_event_question] Rejected: Event id={} not found.",
            event_id
        );
        return;
    }

    let question = EventQuestion {
        id: 0,
        event_id,
        label: label.trim().to_string(),
        field_type: field_type.trim().to_string(),
        options,
        is_required,
    };

    ctx.db.event_question().insert(question);

    log::info!(
        "[add_event_question] Question '{}' added to event id={} (required={}).",
        label.trim(),
        event_id,
        is_required
    );
}

/// Deactivate an event (soft delete).
#[spacetimedb::reducer]
pub fn deactivate_event(ctx: &spacetimedb::ReducerContext, event_id: u64) {
    match ctx.db.event().id().find(&event_id) {
        Some(mut event) => {
            event.is_active = false;
            let title = event.title.clone();
            ctx.db.event().id().update(event);
            log::info!(
                "[deactivate_event] Event '{}' (id={}) deactivated.",
                title,
                event_id
            );
        }
        None => {
            log::error!(
                "[deactivate_event] Failed: Event id={} not found.",
                event_id
            );
        }
    }
}

/// Delete an event response (admin can remove a specific RSVP).
#[spacetimedb::reducer]
pub fn delete_event_response(ctx: &spacetimedb::ReducerContext, response_id: u64) {
    let table = ctx.db.event_response();

    match table.id().find(&response_id) {
        Some(response) => {
            table.id().delete(response_id);
            log::info!(
                "[delete_event_response] Response id={} deleted (user: {:?}, event: {}).",
                response_id,
                response.user_identity,
                response.event_id
            );
        }
        None => {
            log::error!(
                "[delete_event_response] Failed: Response id={} not found.",
                response_id
            );
        }
    }
}
