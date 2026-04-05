//!
//! 4ever & Beyond — Community Platform Frontend
//!
//! Built with Dioxus 0.7 Fullstack + PostgreSQL + Tailwind CSS
//!
//! View State Machine:
//!   Loading → Onboarding → EventView → Submitted
//!                     ↘ Admin ↗ (toggle from header button)
//!
//! i18n: All user-facing strings come from `i18n.rs` via the `Locale` struct.
//!       Switch languages at runtime with the header toggle.

mod backend;
mod i18n;

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::i18n::{get_locale, Language};
use backend::*;

// =============================================================================
// CONSTANTS
// =============================================================================

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

const INSTAGRAM_URL: &str = "https://www.instagram.com/raknong_4ever";
const MENU_URL: &str = "https://linktr.ee/steakdekuanwattana";

/// Admin dashboard passcode. Change this to your own secret value.
/// In production, this should come from an environment variable.
const ADMIN_PASSCODE: &str = "4ever-admin-2026";

// =============================================================================
// VIEW STATE
// =============================================================================

#[derive(Clone, Debug, PartialEq)]
pub enum ViewState {
    Loading,
    Onboarding,
    EventView,
    Submitted,
    /// Passcode gate before granting admin access
    AdminAuth,
    Admin,
}

/// Shared reactive state passed through Dioxus context.
#[derive(Clone, Copy)]
struct AppState {
    view_state: SyncSignal<ViewState>,
    user_profile: SyncSignal<Option<UserProfile>>,
    active_event: SyncSignal<Option<EventData>>,
    event_questions: SyncSignal<Vec<EventQuestion>>,
    error_message: SyncSignal<Option<String>>,
    language: SyncSignal<Language>,
    all_users: SyncSignal<Vec<UserProfile>>,
    all_responses: SyncSignal<Vec<EventResponse>>,
    session_id: SyncSignal<String>,
    data_loaded: SyncSignal<bool>,
}

// =============================================================================
// ENTRY POINT
// =============================================================================

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        if let Err(e) = backend::init_db().await {
            log::error!("Database init error: {e}");
        }
        Ok(dioxus::server::router(App))
    });
}

// =============================================================================
// SESSION MANAGEMENT
// =============================================================================

/// Generate or retrieve a persistent session ID from localStorage.
/// All ID generation is done in JS to avoid `std::time` (panics in WASM).
#[allow(dead_code)]
async fn get_or_create_session_id() -> String {
    #[cfg(not(feature = "server"))]
    {
        // Everything stays in JS — no Rust `SystemTime` / `Instant` usage.
        let js = r#"
            (function() {
                let id = localStorage.getItem('4ever_session_id');
                if (!id) {
                    // crypto.randomUUID() is available in all modern browsers
                    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
                        id = crypto.randomUUID();
                    } else {
                        // Fallback: random hex string via Math.random
                        id = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
                            var r = Math.random() * 16 | 0, v = c === 'x' ? r : (r & 0x3 | 0x8);
                            return v.toString(16);
                        });
                    }
                    localStorage.setItem('4ever_session_id', id);
                }
                return id;
            })()
        "#;

        match dioxus::document::eval(js).await {
            Ok(v) => v
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| "session-unknown".into()),
            Err(_) => "session-unknown".into(),
        }
    }

    #[cfg(feature = "server")]
    {
        format!("session-server-{}", std::process::id())
    }
}

// =============================================================================
// DATA LOADING HELPERS
// =============================================================================

/// Load initial data: events + user profile. Sets `data_loaded` when done.
#[allow(dead_code)]
async fn load_initial_data(mut state: AppState) {
    let sid = (state.session_id)();

    // Load active events
    match get_events().await {
        Ok(events) => {
            if let Some(ewq) = events.first() {
                state.active_event.set(Some(ewq.event.clone()));
                state.event_questions.set(ewq.questions.clone());
            }
        }
        Err(e) => {
            log::error!("Failed to load events: {e}");
            state
                .error_message
                .set(Some(format!("Failed to load events: {e}")));
        }
    }

    // Load user profile
    match get_user_profile(sid.clone()).await {
        Ok(Some(profile)) => {
            state.user_profile.set(Some(profile));
            // Check if the user already RSVP'd to the active event
            if let Some(event) = (state.active_event)() {
                match check_existing_response(event.id as i64, sid).await {
                    Ok(true) => state.view_state.set(ViewState::Submitted),
                    _ => state.view_state.set(ViewState::EventView),
                }
            } else {
                state.view_state.set(ViewState::EventView);
            }
        }
        Ok(None) => {
            state.view_state.set(ViewState::Onboarding);
        }
        Err(e) => {
            log::error!("Failed to load profile: {e}");
            state.view_state.set(ViewState::Onboarding);
        }
    }

    state.data_loaded.set(true);
}

/// Refresh admin dashboard data (users + responses).
async fn load_admin_data(mut state: AppState) -> Result<(), String> {
    match get_all_users().await {
        Ok(users) => state.all_users.set(users),
        Err(e) => return Err(format!("Failed to load users: {e}")),
    }
    match get_all_responses().await {
        Ok(responses) => state.all_responses.set(responses),
        Err(e) => return Err(format!("Failed to load responses: {e}")),
    }
    Ok(())
}

// =============================================================================
// APP ROOT
// =============================================================================

#[component]
fn App() -> Element {
    let mut state = AppState {
        view_state: use_signal_sync(|| ViewState::Loading),
        user_profile: use_signal_sync(|| None),
        active_event: use_signal_sync(|| None),
        event_questions: use_signal_sync(Vec::new),
        error_message: use_signal_sync(|| None),
        language: use_signal_sync(|| Language::Thai),
        all_users: use_signal_sync(Vec::new),
        all_responses: use_signal_sync(Vec::new),
        session_id: use_signal_sync(String::new),
        data_loaded: use_signal_sync(|| false),
    };

    use_context_provider(|| state);

    // ── Initialise session + load data ────────────────────────────────
    // On the server build the future body is empty so SSR always renders
    // the Loading view.  On the client the real session ID is obtained and
    // data is fetched from the backend.
    use_future(move || {
        // `mut` needed on web (SyncSignal calls .set()), appears unused on server
        #[allow(unused_mut, unused_variables)]
        let mut state = state;
        async move {
            #[cfg(not(feature = "server"))]
            {
                let sid = get_or_create_session_id().await;
                state.session_id.set(sid);
                load_initial_data(state).await;
            }
        }
    });

    let lang = (state.language)();
    let locale = get_locale(lang);
    let current_state = (state.view_state)();
    let error = (state.error_message)();
    let is_admin = matches!(current_state, ViewState::Admin | ViewState::AdminAuth);
    let data_loaded = (state.data_loaded)();

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div {
            class: "min-h-screen bg-gradient-to-br from-slate-900 via-indigo-950 to-slate-900 text-white flex flex-col",

            // ── Header ──────────────────────────────────────────────
            header {
                class: "bg-black/30 backdrop-blur-md border-b border-white/10 sticky top-0 z-50",
                div {
                    class: "max-w-4xl mx-auto px-4 py-3 flex items-center justify-between",
                    // Brand
                    div {
                        h1 {
                            class: "text-2xl md:text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent tracking-tight",
                            "{locale.app_name}"
                        }
                        p {
                            class: "text-xs text-indigo-300/70 mt-0.5 font-medium",
                            "{locale.tagline}"
                        }
                    }
                    // Right side: admin, lang toggle, IG link
                    div {
                        class: "flex items-center gap-3",

                        // Admin toggle
                        button {
                            class: if is_admin {
                                "text-xs font-semibold px-2.5 py-1 rounded-full border border-indigo-500/40 bg-indigo-500/20 text-indigo-300 hover:bg-indigo-500/30 transition cursor-pointer"
                            } else {
                                "text-xs font-semibold px-2.5 py-1 rounded-full border border-white/15 bg-white/5 hover:bg-white/10 transition text-slate-300 cursor-pointer"
                            },
                            onclick: move |_| {
                                if is_admin {
                                    state.view_state.set(ViewState::EventView);
                                } else {
                                    state.view_state.set(ViewState::AdminAuth);
                                }
                            },
                            if is_admin { "← {locale.admin_back_to_event}" } else { "{locale.admin_nav_button}" }
                        }

                        // Language toggle
                        button {
                            class: "text-xs font-semibold px-2.5 py-1 rounded-full border border-white/15 bg-white/5 hover:bg-white/10 transition text-slate-300 cursor-pointer",
                            onclick: move |_| {
                                let current = (state.language)();
                                state.language.set(current.toggle());
                            },
                            {match lang {
                                Language::Thai => locale.lang_toggle_en,
                                Language::English => locale.lang_toggle_th,
                            }}
                        }
                        // Instagram link
                        a {
                            href: INSTAGRAM_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "text-slate-400 hover:text-pink-400 transition",
                            aria_label: "Instagram",
                            svg {
                                class: "w-5 h-5",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                stroke_width: "2",
                                rect { x: "2", y: "2", width: "20", height: "20", rx: "5", ry: "5" }
                                circle { cx: "12", cy: "12", r: "5" }
                                circle { cx: "17.5", cy: "6.5", r: "1.5", fill: "currentColor", stroke: "none" }
                            }
                        }
                    }
                }
            }

            // ── Error Banner ────────────────────────────────────────
            {error.map(|msg| rsx! {
                ErrorBanner { message: msg }
            })}

            // ── Main Content ────────────────────────────────────────
            main {
                class: "flex-1 max-w-4xl w-full mx-auto px-4 py-8",
                match (data_loaded, current_state.clone()) {
                    (false, _) => rsx! { LoadingView {} },
                    (true, ViewState::Loading) => rsx! { LoadingView {} },
                    (true, ViewState::Onboarding) => rsx! { OnboardingView {} },
                    (true, ViewState::EventView) => rsx! { EventView {} },
                    (true, ViewState::Submitted) => rsx! { SubmittedView {} },
                    (_, ViewState::AdminAuth) => rsx! { AdminAuthView {} },
                    (true, ViewState::Admin) => rsx! { AdminView {} },
                }
            }

            // ── Footer ──────────────────────────────────────────────
            footer {
                class: "mt-auto py-5 text-center space-y-1",
                p {
                    class: "text-slate-600 text-xs tracking-wide",
                    "{locale.footer_text}"
                }
                a {
                    href: INSTAGRAM_URL,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "inline-flex items-center gap-1.5 text-xs text-slate-500 hover:text-pink-400 transition",
                    svg {
                        class: "w-3.5 h-3.5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        stroke_width: "2",
                        rect { x: "2", y: "2", width: "20", height: "20", rx: "5", ry: "5" }
                        circle { cx: "12", cy: "12", r: "5" }
                    }
                    "@raknong_4ever"
                }
            }
        }
    }
}

// =============================================================================
// LOADING VIEW
// =============================================================================

#[component]
fn LoadingView() -> Element {
    let state: AppState = use_context();
    let locale = get_locale((state.language)());

    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-32 space-y-6",
            div {
                class: "relative",
                div {
                    class: "w-16 h-16 border-4 border-indigo-500/30 border-t-indigo-400 rounded-full animate-spin",
                }
            }
            p {
                class: "text-sm text-indigo-400 animate-pulse font-medium tracking-wide",
                "{locale.connecting}"
            }
            div {
                class: "text-center space-y-2",
                h2 {
                    class: "text-xl font-semibold text-slate-200",
                    "{locale.loading_title}"
                }
                p {
                    class: "text-sm text-slate-500",
                    "{locale.loading_subtitle}"
                }
            }
        }
    }
}

// =============================================================================
// ONBOARDING VIEW — 5 required fields
// =============================================================================

#[component]
fn OnboardingView() -> Element {
    let mut state: AppState = use_context();
    let locale = get_locale((state.language)());

    let mut nickname = use_signal(String::new);
    let mut entry_year = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut instagram = use_signal(String::new);
    let mut line_id = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let year_options = locale.year_options();

    let on_submit = move |_| {
        let loc = get_locale((state.language)());

        let nick = nickname.read().clone();
        let year = entry_year.read().clone();
        let ph = phone.read().clone();
        let ig = instagram.read().clone();
        let line = line_id.read().clone();

        // Validate all required fields
        if nick.trim().is_empty() {
            state
                .error_message
                .set(Some(loc.err_nickname_required.to_string()));
            return;
        }
        if year.trim().is_empty() {
            state
                .error_message
                .set(Some(loc.err_year_required.to_string()));
            return;
        }
        if ph.trim().is_empty() {
            state
                .error_message
                .set(Some(loc.err_phone_required.to_string()));
            return;
        }
        if ig.trim().is_empty() {
            state
                .error_message
                .set(Some(loc.err_instagram_required.to_string()));
            return;
        }
        if line.trim().is_empty() {
            state
                .error_message
                .set(Some(loc.err_line_required.to_string()));
            return;
        }

        is_submitting.set(true);

        // Capture for async block
        let mut state = state;
        let mut is_submitting = is_submitting;
        let sid = (state.session_id)();
        let nick = nick.trim().to_string();
        let year = year.trim().to_string();
        let ph = ph.trim().to_string();
        let ig = ig.trim().to_string();
        let line = line.trim().to_string();

        spawn(async move {
            match register_profile(sid, nick, year, ph, ig, line).await {
                Ok(_) => {
                    log::info!("register_profile succeeded.");
                    // Reload profile from server
                    let sid = (state.session_id)();
                    if let Ok(Some(profile)) = get_user_profile(sid).await {
                        state.user_profile.set(Some(profile));
                    }
                    state.view_state.set(ViewState::EventView);
                }
                Err(e) => {
                    state
                        .error_message
                        .set(Some(format!("Registration failed: {e}")));
                }
            }
            is_submitting.set(false);
        });
    };

    rsx! {
        div {
            class: "flex items-center justify-center py-6 px-2",
            div {
                class: "w-full max-w-md",
                div {
                    class: "bg-white/[0.04] backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-2xl",

                    // Header
                    div {
                        class: "text-center mb-8",
                        div {
                            class: "w-16 h-16 bg-indigo-500/20 rounded-full flex items-center justify-center mx-auto mb-4",
                            span { class: "text-3xl", "👋" }
                        }
                        h2 { class: "text-2xl font-bold text-white", "{locale.onboard_title}" }
                        p { class: "text-slate-400 text-sm mt-2 leading-relaxed", "{locale.onboard_subtitle}" }
                    }

                    // Form fields
                    div {
                        class: "space-y-5",

                        // 1. Nickname
                        FormField {
                            label: locale.nickname_label.to_string(),
                            required: true,
                            input_type: "text".to_string(),
                            placeholder: locale.nickname_placeholder.to_string(),
                            value: nickname,
                            on_change: move |v| nickname.set(v),
                        }

                        // 2. Year (select)
                        div {
                            class: "space-y-1.5",
                            label {
                                class: "block text-sm font-medium text-slate-300",
                                "{locale.year_label}"
                                span { class: "text-indigo-400 ml-1", "*" }
                            }
                            select {
                                class: "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition appearance-none cursor-pointer",
                                value: "{entry_year}",
                                onchange: move |e| entry_year.set(e.value()),
                                option { value: "", disabled: true, "{locale.year_placeholder}" }
                                {year_options.iter().map(|opt| rsx! {
                                    option { key: "{opt}", value: "{opt}", "{opt}" }
                                })}
                            }
                        }

                        // 3. Phone
                        FormField {
                            label: locale.phone_label.to_string(),
                            required: true,
                            input_type: "tel".to_string(),
                            placeholder: locale.phone_placeholder.to_string(),
                            value: phone,
                            on_change: move |v| phone.set(v),
                        }

                        // 4. Instagram
                        FormField {
                            label: locale.instagram_label.to_string(),
                            required: true,
                            input_type: "text".to_string(),
                            placeholder: locale.instagram_placeholder.to_string(),
                            value: instagram,
                            on_change: move |v| instagram.set(v),
                        }

                        // 5. Line ID
                        FormField {
                            label: locale.line_label.to_string(),
                            required: true,
                            input_type: "text".to_string(),
                            placeholder: locale.line_placeholder.to_string(),
                            value: line_id,
                            on_change: move |v| line_id.set(v),
                        }

                        // Submit
                        button {
                            class: if is_submitting() {
                                "w-full bg-indigo-500/40 text-indigo-200 font-semibold py-3.5 rounded-xl cursor-not-allowed transition"
                            } else {
                                "w-full bg-indigo-500 hover:bg-indigo-600 active:bg-indigo-700 text-white font-semibold py-3.5 rounded-xl shadow-lg shadow-indigo-500/25 transition-all duration-200 transform hover:scale-[1.01] active:scale-[0.99]"
                            },
                            disabled: is_submitting(),
                            onclick: on_submit,
                            if is_submitting() { "{locale.creating_profile}" } else { "{locale.continue_button}" }
                        }
                    }

                    p {
                        class: "text-center text-xs text-slate-600 mt-5 leading-relaxed",
                        "🔒 {locale.data_safe}"
                    }
                }
            }
        }
    }
}

// =============================================================================
// EVENT VIEW — Dynamic RSVP with passcode
// =============================================================================

#[component]
fn EventView() -> Element {
    let mut state: AppState = use_context();
    let locale = get_locale((state.language)());

    let event = (state.active_event)();
    let questions = (state.event_questions)();
    let profile = (state.user_profile)();

    let mut answers: Signal<Vec<String>> =
        use_signal(|| questions.iter().map(|_| String::new()).collect());
    let mut passcode = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut passcode_error = use_signal(|| false);

    let on_submit = move |_| {
        let loc = get_locale((state.language)());
        let pc = passcode.read().clone();
        let current_answers = answers.read().clone();
        let qs = (state.event_questions)();

        if pc.trim().is_empty() {
            state
                .error_message
                .set(Some(loc.err_passcode_required.to_string()));
            passcode_error.set(true);
            return;
        }

        // Validate required answers
        for (i, q) in qs.iter().enumerate() {
            if q.is_required {
                if let Some(ans) = current_answers.get(i) {
                    if ans.trim().is_empty() {
                        state
                            .error_message
                            .set(Some(format!("{}{}", loc.err_answer_prefix, q.label)));
                        return;
                    }
                }
            }
        }

        passcode_error.set(false);
        is_submitting.set(true);

        // Build answers JSON: { "question_id": "answer" }
        let answers_map: HashMap<String, String> = qs
            .iter()
            .enumerate()
            .filter_map(|(i, q)| {
                current_answers
                    .get(i)
                    .map(|a| (q.id.to_string(), a.clone()))
            })
            .collect();
        let answers_json = serde_json::to_string(&answers_map).unwrap_or_default();

        // Get event ID
        let event_id = (state.active_event)().map_or(0, |e| e.id as i64);
        let sid = (state.session_id)();

        let mut state = state;
        let mut is_submitting = is_submitting;

        spawn(async move {
            match submit_response(event_id, pc.trim().to_string(), sid, answers_json).await {
                Ok(()) => {
                    log::info!("submit_response succeeded.");
                    state.view_state.set(ViewState::Submitted);
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("Invalid passcode") {
                        state
                            .error_message
                            .set(Some(loc.err_passcode_invalid.to_string()));
                        passcode_error.set(true);
                    } else {
                        state.error_message.set(Some(format!("RSVP failed: {e}")));
                    }
                }
            }
            is_submitting.set(false);
        });
    };

    // No active event
    let event_data = match event {
        Some(e) => e,
        None => {
            return rsx! {
                div {
                    class: "text-center py-24",
                    div {
                        class: "w-20 h-20 bg-slate-800 rounded-full flex items-center justify-center mx-auto mb-6",
                        span { class: "text-4xl", "📭" }
                    }
                    h2 { class: "text-xl text-slate-300 font-semibold", "{locale.no_events_title}" }
                    p { class: "text-slate-500 mt-2 text-sm", "{locale.no_events_subtitle}" }
                }
            };
        }
    };

    rsx! {
        div {
            class: "space-y-5",

            // ── Welcome Bar ─────────────────────────────────────────
            {profile.map(|p| rsx! {
                div {
                    class: "flex items-center gap-4 bg-white/[0.03] rounded-xl px-5 py-4 border border-white/10",
                    div {
                        class: "w-11 h-11 bg-indigo-500/20 rounded-full flex items-center justify-center text-lg shrink-0",
                        "👤"
                    }
                    div {
                        class: "min-w-0",
                        p { class: "text-xs text-slate-500 truncate", "{locale.welcome_back}" }
                        p { class: "font-semibold text-white truncate", "{p.nickname}" }
                    }
                    if p.is_verified {
                        span {
                            class: "ml-auto text-xs bg-emerald-500/15 text-emerald-400 px-2.5 py-1 rounded-full shrink-0 font-medium",
                            "{locale.verified}"
                        }
                    } else {
                        span {
                            class: "ml-auto text-xs bg-amber-500/15 text-amber-400 px-2.5 py-1 rounded-full shrink-0 font-medium",
                            "{locale.pending}"
                        }
                    }
                }
            })}

            // ── Event Card ──────────────────────────────────────────
            div {
                class: "bg-gradient-to-br from-indigo-500/[0.07] to-purple-500/[0.07] border border-indigo-500/20 rounded-2xl overflow-hidden shadow-xl",

                // Event header
                div {
                    class: "bg-indigo-500/[0.08] px-6 py-5 border-b border-indigo-500/10",
                    div {
                        class: "flex items-start justify-between gap-4 flex-wrap",
                        div {
                            h2 {
                                class: "text-2xl font-bold text-white leading-tight",
                                "{event_data.title}"
                            }
                            p {
                                class: "text-slate-400 text-sm mt-2 leading-relaxed max-w-lg",
                                white_space: "pre-line",
                                "{event_data.description}"
                            }
                        }
                        span {
                            class: "flex-shrink-0 bg-emerald-500/15 text-emerald-400 text-xs font-semibold px-3 py-1.5 rounded-full tracking-wide",
                            "{locale.active_badge}"
                        }
                    }
                    div {
                        class: "mt-4 flex items-center justify-between flex-wrap gap-3",
                        div {
                            class: "flex items-center gap-2 text-sm text-slate-500",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z",
                                }
                            }
                            "{event_data.event_date}"
                        }
                        a {
                            href: MENU_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "inline-flex items-center gap-1.5 text-xs font-medium text-indigo-300 hover:text-indigo-200 bg-indigo-500/10 hover:bg-indigo-500/20 px-3 py-1.5 rounded-full transition",
                            svg {
                                class: "w-3.5 h-3.5",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z",
                                }
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M15 11a3 3 0 11-6 0 3 3 0 016 0z",
                                }
                            }
                            "{locale.view_menu}"
                        }
                    }
                }

                // ── Dynamic Questions ────────────────────────────────
                div {
                    class: "px-6 py-6 space-y-6",

                    h3 {
                        class: "text-lg font-semibold text-white flex items-center gap-2",
                        span { "📝" }
                        "{locale.rsvp_title}"
                    }

                    {questions.iter().enumerate().map(|(index, question)| {
                        let current_val = answers.read().get(index).cloned().unwrap_or_default();
                        let parsed_options: Vec<String> = question.options.as_ref()
                            .and_then(|opts| serde_json::from_str(opts).ok())
                            .unwrap_or_default();

                        rsx! {
                            div {
                                key: "q_{question.id}",
                                class: "space-y-2",

                                label {
                                    class: "block text-sm font-medium text-slate-300",
                                    "{question.label}"
                                    if question.is_required {
                                        span { class: "text-red-400", "{locale.required_marker}" }
                                    } else {
                                        span { class: "text-slate-600 text-xs ml-1", "{locale.optional_label}" }
                                    }
                                }

                                {
                                    let field_type = question.field_type.clone();
                                    let idx = index;

                                    match field_type.as_str() {
                                        "select" => {
                                            let opts = parsed_options.clone();
                                            rsx! {
                                                select {
                                                    class: "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition appearance-none cursor-pointer",
                                                    value: "{current_val}",
                                                    onchange: move |e| {
                                                        let mut a = answers.write();
                                                        if a.len() > idx { a[idx] = e.value(); }
                                                    },
                                                    option { value: "", disabled: true, "{locale.choose_option}" }
                                                    {opts.into_iter().map(|opt| rsx! {
                                                        option { key: "{opt}", value: "{opt}", "{opt}" }
                                                    })}
                                                }
                                            }
                                        }
                                        "radio" => {
                                            let opts = parsed_options.clone();
                                            rsx! {
                                                div {
                                                    class: "flex gap-3 flex-wrap",
                                                    {opts.into_iter().map(|opt| {
                                                        let is_selected = current_val == opt;
                                                        let opt_clone = opt.clone();
                                                        rsx! {
                                                            label {
                                                                key: "radio_{question.id}_{opt}",
                                                                class: if is_selected {
                                                                    "flex items-center gap-2.5 bg-indigo-500/20 border-2 border-indigo-500/40 rounded-xl px-5 py-2.5 cursor-pointer transition-all"
                                                                } else {
                                                                    "flex items-center gap-2.5 bg-white/5 border border-white/10 rounded-xl px-5 py-2.5 cursor-pointer hover:bg-white/10 transition-all"
                                                                },
                                                                input {
                                                                    r#type: "radio",
                                                                    name: "question_{question.id}",
                                                                    value: "{opt}",
                                                                    checked: is_selected,
                                                                    onchange: move |_| {
                                                                        let mut a = answers.write();
                                                                        if a.len() > idx { a[idx] = opt_clone.clone(); }
                                                                    },
                                                                }
                                                                span { class: "text-sm text-slate-200 font-medium", "{opt}" }
                                                            }
                                                        }
                                                    })}
                                                }
                                            }
                                        }
                                        _ => {
                                            rsx! {
                                                input {
                                                    class: "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition",
                                                    r#type: "text",
                                                    placeholder: "Type your answer...",
                                                    value: "{current_val}",
                                                    oninput: move |e| {
                                                        let mut a = answers.write();
                                                        if a.len() > idx { a[idx] = e.value(); }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })}

                    // ── Passcode Security Gate ────────────────────────
                    div {
                        class: "pt-5 border-t border-white/10 mt-2",

                        div {
                            class: "flex items-center gap-2 mb-3",
                            svg {
                                class: "w-5 h-5 text-amber-400 shrink-0",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z",
                                }
                            }
                            label {
                                class: "text-sm font-semibold text-amber-300",
                                "{locale.passcode_label}"
                                span { class: "text-red-400 ml-1", "*" }
                            }
                        }

                        input {
                            class: if passcode_error() {
                                "w-full bg-red-500/10 border-2 border-red-500/40 rounded-xl px-4 py-3 text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-red-500 transition"
                            } else {
                                "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition"
                            },
                            r#type: "text",
                            placeholder: "{locale.passcode_placeholder}",
                            value: "{passcode}",
                            oninput: move |e| {
                                passcode.set(e.value());
                                passcode_error.set(false);
                                state.error_message.set(None);
                            },
                        }
                        p {
                            class: "text-xs text-slate-600 mt-2 leading-relaxed",
                            "{locale.passcode_hint}"
                        }
                    }

                    // ── Submit Button ─────────────────────────────────
                    button {
                        class: if is_submitting() {
                            "w-full bg-indigo-500/40 text-indigo-200 font-bold py-4 rounded-xl cursor-not-allowed text-lg transition"
                        } else {
                            "w-full bg-gradient-to-r from-indigo-500 to-purple-500 hover:from-indigo-600 hover:to-purple-600 active:from-indigo-700 active:to-purple-700 text-white font-bold py-4 rounded-xl shadow-lg shadow-indigo-500/25 transition-all duration-200 transform hover:scale-[1.01] active:scale-[0.99] text-lg"
                        },
                        disabled: is_submitting(),
                        onclick: on_submit,
                        if is_submitting() { "{locale.submitting}" } else { "{locale.submit_rsvp}" }
                    }

                    p {
                        class: "text-center text-xs text-slate-600 leading-relaxed",
                        "🔒 {locale.data_safe}"
                    }
                }
            }
        }
    }
}

// =============================================================================
// SUBMITTED VIEW — Confirmation
// =============================================================================

#[component]
fn SubmittedView() -> Element {
    let state: AppState = use_context();
    let locale = get_locale((state.language)());
    let event = (state.active_event)();
    let profile = (state.user_profile)();

    rsx! {
        div {
            class: "flex items-center justify-center py-20 px-4",
            div {
                class: "text-center max-w-md",
                div {
                    class: "w-28 h-28 bg-emerald-500/15 rounded-full flex items-center justify-center mx-auto mb-8",
                    span { class: "text-6xl", "🎉" }
                }
                h2 {
                    class: "text-3xl font-extrabold text-white mb-3 tracking-tight",
                    "{locale.submitted_title}"
                }
                p {
                    class: "text-slate-400 text-lg mb-8",
                    "{locale.submitted_subtitle}"
                }

                {event.map(|e| rsx! {
                    div {
                        class: "bg-white/[0.04] rounded-2xl px-8 py-6 inline-block border border-white/10",
                        p {
                            class: "text-xs text-slate-500 uppercase tracking-widest mb-1",
                            "{locale.event_label}"
                        }
                        p { class: "font-bold text-white text-xl", "{e.title}" }
                        div {
                            class: "flex items-center justify-center gap-2 mt-2 text-sm text-slate-400",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z",
                                }
                            }
                            "{e.event_date}"
                        }

                        a {
                            href: MENU_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "inline-flex items-center gap-1.5 text-xs font-medium text-indigo-300 hover:text-indigo-200 mt-3 transition",
                            "📋 {locale.view_menu}"
                        }
                    }
                })}

                {profile.map(|p| rsx! {
                    div {
                        class: "mt-6 text-sm text-slate-500",
                        "{locale.registered_as}"
                        span { class: "text-slate-300 font-medium", "{p.nickname}" }
                        " ({p.entry_year})"
                    }
                })}

                div {
                    class: "mt-10",
                    p { class: "text-slate-500 text-base", "{locale.see_you}" }
                }
            }
        }
    }
}

// =============================================================================
// ADMIN AUTHENTICATION VIEW
// =============================================================================

/// Passcode gate that must be cleared before accessing the admin dashboard.
#[component]
fn AdminAuthView() -> Element {
    let mut state: AppState = use_context();
    let locale = get_locale((state.language)());
    let mut admin_passcode = use_signal(String::new);
    let mut auth_error = use_signal(|| false);

    rsx! {
        div {
            class: "flex items-center justify-center py-20 px-2",
            div {
                class: "w-full max-w-sm",
                div {
                    class: "bg-white/[0.04] backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-2xl",

                    // Header
                    div {
                        class: "text-center mb-8",
                        div {
                            class: "w-16 h-16 bg-amber-500/20 rounded-full flex items-center justify-center mx-auto mb-4",
                            span { class: "text-3xl", "🔒" }
                        }
                        h2 { class: "text-2xl font-bold text-white", "{locale.admin_auth_title}" }
                        p {
                            class: "text-slate-400 text-sm mt-2",
                            "{locale.admin_nav_button}"
                        }
                    }

                    // Passcode field
                    div {
                        class: "space-y-4",
                        div {
                            class: "space-y-1.5",
                            label {
                                class: "block text-sm font-medium text-amber-300",
                                "{locale.admin_auth_passcode_label}"
                            }
                            input {
                                class: if auth_error() {
                                    "w-full bg-red-500/10 border-2 border-red-500/40 rounded-xl px-4 py-3 text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-red-500 transition"
                                } else {
                                    "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent transition"
                                },
                                r#type: "password",
                                placeholder: "{locale.admin_auth_passcode_placeholder}",
                                value: "{admin_passcode}",
                                oninput: move |e| {
                                    admin_passcode.set(e.value());
                                    auth_error.set(false);
                                    state.error_message.set(None);
                                },
                            }
                        }

                        // Error message
                        {auth_error().then(|| rsx! {
                            p {
                                class: "text-red-400 text-xs font-medium",
                                "{locale.admin_auth_error}"
                            }
                        })}

                        // Submit
                        button {
                            class: "w-full bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-600 hover:to-orange-600 active:from-amber-700 active:to-orange-700 text-white font-semibold py-3 rounded-xl shadow-lg shadow-amber-500/25 transition-all duration-200 transform hover:scale-[1.01] active:scale-[0.99] cursor-pointer",
                            onclick: move |_| {
                                let pc = admin_passcode.read().clone();
                                if pc.trim() == ADMIN_PASSCODE {
                                    auth_error.set(false);
                                    let mut state = state;
                                    spawn(async move {
                                        let _ = load_admin_data(state).await;
                                    });
                                    state.view_state.set(ViewState::Admin);
                                } else {
                                    auth_error.set(true);
                                    state.error_message.set(None);
                                }
                                admin_passcode.set(String::new());
                            },
                            "{locale.admin_auth_submit}"
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// ADMIN DASHBOARD VIEW
// =============================================================================

#[component]
fn AdminView() -> Element {
    let state: AppState = use_context();
    let locale = get_locale((state.language)());
    let users = (state.all_users)();
    let responses = (state.all_responses)();
    let questions = (state.event_questions)();

    let mut show_users_tab = use_signal(|| true);

    let total_users = users.len();
    let total_responses = responses.len();
    let verified_count = users.iter().filter(|u| u.is_verified).count();
    let pending_count = total_users.saturating_sub(verified_count);

    rsx! {
        div {
            class: "space-y-6",

            // ── Title ───────────────────────────────────────────────
            div {
                class: "flex items-center justify-between",
                h2 {
                    class: "text-2xl font-bold text-white flex items-center gap-3",
                    span { "🛡" }
                    "{locale.admin_title}"
                }
            }

            // ── Stats Grid ──────────────────────────────────────────
            div {
                class: "grid grid-cols-2 md:grid-cols-4 gap-4",

                // Total Users
                div {
                    class: "bg-white/[0.04] backdrop-blur-xl border border-white/10 rounded-2xl p-5",
                    p { class: "text-xs text-slate-500 uppercase tracking-wider font-medium", "{locale.admin_total_users}" }
                    p { class: "text-3xl font-bold text-white mt-1", "{total_users}" }
                }

                // Total Responses
                div {
                    class: "bg-white/[0.04] backdrop-blur-xl border border-white/10 rounded-2xl p-5",
                    p { class: "text-xs text-slate-500 uppercase tracking-wider font-medium", "{locale.admin_total_responses}" }
                    p { class: "text-3xl font-bold text-white mt-1", "{total_responses}" }
                }

                // Verified
                div {
                    class: "bg-white/[0.04] backdrop-blur-xl border border-emerald-500/20 rounded-2xl p-5",
                    p { class: "text-xs text-emerald-400 uppercase tracking-wider font-medium", "{locale.admin_verified_users}" }
                    p { class: "text-3xl font-bold text-emerald-300 mt-1", "{verified_count}" }
                }

                // Pending
                div {
                    class: "bg-white/[0.04] backdrop-blur-xl border border-amber-500/20 rounded-2xl p-5",
                    p { class: "text-xs text-amber-400 uppercase tracking-wider font-medium", "{locale.admin_pending_users}" }
                    p { class: "text-3xl font-bold text-amber-300 mt-1", "{pending_count}" }
                }
            }

            // ── Tab Bar ─────────────────────────────────────────────
            div {
                class: "flex gap-2",
                button {
                    class: if show_users_tab() {
                        "px-4 py-2 rounded-xl font-medium bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 cursor-pointer transition"
                    } else {
                        "px-4 py-2 rounded-xl font-medium bg-white/5 text-slate-400 border border-white/10 hover:bg-white/10 cursor-pointer transition"
                    },
                    onclick: move |_| show_users_tab.set(true),
                    "{locale.admin_users_tab} ({total_users})"
                }
                button {
                    class: if !show_users_tab() {
                        "px-4 py-2 rounded-xl font-medium bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 cursor-pointer transition"
                    } else {
                        "px-4 py-2 rounded-xl font-medium bg-white/5 text-slate-400 border border-white/10 hover:bg-white/10 cursor-pointer transition"
                    },
                    onclick: move |_| show_users_tab.set(false),
                    "{locale.admin_responses_tab} ({total_responses})"
                }
            }

            // ── Tab Content ─────────────────────────────────────────
            if show_users_tab() {
                // Users Tab
                if users.is_empty() {
                    div {
                        class: "text-center py-16 text-slate-500",
                        p { "{locale.admin_no_users}" }
                    }
                } else {
                    div {
                        class: "bg-white/[0.03] border border-white/10 rounded-xl overflow-hidden",
                        div {
                            class: "overflow-x-auto",
                            table {
                                class: "w-full text-sm text-left",
                                thead {
                                    tr {
                                        class: "bg-white/[0.04] border-b border-white/10",
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "#" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "{locale.admin_col_nickname}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "{locale.admin_col_year}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium hidden md:table-cell", "{locale.admin_col_phone}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium hidden md:table-cell", "{locale.admin_col_instagram}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium hidden lg:table-cell", "{locale.admin_col_line}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "{locale.admin_col_status}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "{locale.admin_col_action}" }
                                    }
                                }
                                tbody {
                                    {users.iter().enumerate().map(|(i, user)| {
                                        let user_sid = user.session_id.clone();
                                        let is_verified = user.is_verified;
                                        let nickname = user.nickname.clone();
                                        let state = state;

                                        rsx! {
                                            tr {
                                                key: "user_{i}",
                                                class: "border-b border-white/5 hover:bg-white/[0.02] transition",
                                                td { class: "px-4 py-3 text-slate-500", "{i + 1}" }
                                                td { class: "px-4 py-3 text-white font-medium", "{user.nickname}" }
                                                td { class: "px-4 py-3 text-slate-300", "{user.entry_year}" }
                                                td { class: "px-4 py-3 text-slate-400 hidden md:table-cell", "{user.phone}" }
                                                td { class: "px-4 py-3 text-slate-400 hidden md:table-cell", "{user.instagram}" }
                                                td { class: "px-4 py-3 text-slate-400 hidden lg:table-cell", "{user.line_id}" }
                                                td {
                                                    class: "px-4 py-3",
                                                    if is_verified {
                                                        span {
                                                            class: "text-xs bg-emerald-500/15 text-emerald-400 px-2.5 py-1 rounded-full font-medium",
                                                            "{locale.verified}"
                                                        }
                                                    } else {
                                                        span {
                                                            class: "text-xs bg-amber-500/15 text-amber-400 px-2.5 py-1 rounded-full font-medium",
                                                            "{locale.pending}"
                                                        }
                                                    }
                                                }
                                                td {
                                                    class: "px-4 py-3",
                                                    button {
                                                        class: if is_verified {
                                                            "text-xs px-3 py-1.5 rounded-lg bg-red-500/15 text-red-400 hover:bg-red-500/25 border border-red-500/20 cursor-pointer transition font-medium"
                                                        } else {
                                                            "text-xs px-3 py-1.5 rounded-lg bg-emerald-500/15 text-emerald-400 hover:bg-emerald-500/25 border border-emerald-500/20 cursor-pointer transition font-medium"
                                                        },
                                                        onclick: move |_| {
                                                            let sid = user_sid.clone();
                                                            let nick_log = nickname.clone();
                                                            let state = state;
                                                            spawn(async move {
                                                                let _ = toggle_verification(sid).await;
                                                                log::info!("toggle_verification called for {nick_log}");
                                                                let _ = load_admin_data(state).await;
                                                            });
                                                        },
                                                        if is_verified { "{locale.admin_revoke_verify}" } else { "{locale.admin_toggle_verify}" }
                                                    }
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
            } else {
                // Responses Tab
                if responses.is_empty() {
                    div {
                        class: "text-center py-16 text-slate-500",
                        p { "{locale.admin_no_responses}" }
                    }
                } else {
                    div {
                        class: "bg-white/[0.03] border border-white/10 rounded-xl overflow-hidden",
                        div {
                            class: "overflow-x-auto",
                            table {
                                class: "w-full text-sm text-left",
                                thead {
                                    tr {
                                        class: "bg-white/[0.04] border-b border-white/10",
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "#" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "{locale.admin_col_responder}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "{locale.admin_col_answers}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium hidden md:table-cell", "{locale.admin_col_submitted_at}" }
                                        th { class: "px-4 py-3 text-slate-400 font-medium", "{locale.admin_col_action}" }
                                    }
                                }
                                tbody {
                                    {responses.iter().enumerate().map(|(i, resp)| {
                                        // Look up responder nickname
                                        let responder_name = users.iter()
                                            .find(|u| u.session_id == resp.session_id)
                                            .map(|u| u.nickname.clone())
                                            .unwrap_or_else(|| "Unknown".to_string());

                                        // Format answers
                                        let formatted_answers = format_answers(&resp.answers, &questions);
                                        let response_id = resp.id;
                                        let display_time = resp.submitted_at.clone();
                                        let state = state;

                                        rsx! {
                                            tr {
                                                key: "resp_{i}",
                                                class: "border-b border-white/5 hover:bg-white/[0.02] transition",
                                                td { class: "px-4 py-3 text-slate-500", "{i + 1}" }
                                                td { class: "px-4 py-3 text-white font-medium", "{responder_name}" }
                                                td {
                                                    class: "px-4 py-3 text-slate-300 max-w-xs",
                                                    div {
                                                        class: "text-xs leading-relaxed whitespace-pre-line",
                                                        "{formatted_answers}"
                                                    }
                                                }
                                                td { class: "px-4 py-3 text-slate-500 text-xs hidden md:table-cell", "{display_time}" }
                                                td {
                                                    class: "px-4 py-3",
                                                    button {
                                                        class: "text-xs px-3 py-1.5 rounded-lg bg-red-500/15 text-red-400 hover:bg-red-500/25 border border-red-500/20 cursor-pointer transition font-medium",
                                                        onclick: move |_| {
                                                            let state = state;
                                                            spawn(async move {
                                                                let _ = delete_event_response(response_id as i64).await;
                                                                log::info!("delete_event_response called for id={response_id}");
                                                                let _ = load_admin_data(state).await;
                                                            });
                                                        },
                                                        "{locale.admin_delete_response}"
                                                    }
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// HELPER: Format answers JSON into readable text
// =============================================================================

fn format_answers(answers_json: &str, questions: &[EventQuestion]) -> String {
    let parsed: HashMap<String, String> = serde_json::from_str(answers_json).unwrap_or_default();

    if parsed.is_empty() {
        return answers_json.to_string();
    }

    questions
        .iter()
        .filter_map(|q| {
            parsed
                .get(&q.id.to_string())
                .map(|answer| format!("{}: {}", q.label, answer))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// REUSABLE COMPONENTS
// =============================================================================

#[component]
fn FormField(
    label: String,
    required: bool,
    input_type: String,
    placeholder: String,
    value: Signal<String>,
    on_change: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "space-y-1.5",
            label {
                class: "block text-sm font-medium text-slate-300",
                "{label}"
                if required {
                    span { class: "text-indigo-400 ml-1", "*" }
                }
            }
            input {
                class: "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition",
                r#type: "{input_type}",
                placeholder: "{placeholder}",
                value: "{value}",
                oninput: move |e| on_change.call(e.value()),
            }
        }
    }
}

#[component]
fn ErrorBanner(message: String) -> Element {
    let mut state: AppState = use_context();
    rsx! {
        div {
            class: "max-w-4xl mx-auto px-4 mt-4",
            div {
                class: "bg-red-500/15 border border-red-500/30 rounded-xl p-4 text-red-200 text-sm flex items-start gap-3",
                svg {
                    class: "w-5 h-5 shrink-0 mt-0.5",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                    }
                }
                span { class: "flex-1 leading-relaxed", "{message}" }
                button {
                    class: "text-red-400 hover:text-white transition shrink-0",
                    onclick: move |_| state.error_message.set(None),
                    "✕"
                }
            }
        }
    }
}
