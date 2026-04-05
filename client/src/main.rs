//! 4ever & Beyond — Community Platform Frontend
//!
//! Built with Dioxus 0.7 + Tailwind CSS
//! Connects to SpacetimeDB backend for persistent state.
//!
//! View State Machine:
//!   Loading → Onboarding → EventView → Submitted

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// ASSETS
// =============================================================================

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// =============================================================================
// MODELS — Mirror the SpacetimeDB tables
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    pub identity: String,
    pub nickname: String,
    pub entry_year: String,
    pub contact_channel: String,
    pub student_id: Option<String>,
    pub is_verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CommunityEvent {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub event_date: String,
    pub priority: u32,
    pub is_active: bool,
    pub passcode: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventQuestion {
    pub id: u64,
    pub event_id: u64,
    pub label: String,
    pub field_type: String,
    pub options: Option<String>,
    pub is_required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventResponse {
    pub id: u64,
    pub event_id: u64,
    pub user_identity: String,
    pub answers: String,
    pub submitted_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ViewState {
    Loading,
    Onboarding,
    EventView,
    Submitted,
}

/// Shared reactive state passed through Dioxus context.
/// All signals are Copy, making them cheap to pass around.
#[derive(Clone, Copy)]
struct AppState {
    view_state: Signal<ViewState>,
    user_profile: Signal<Option<UserProfile>>,
    active_event: Signal<Option<CommunityEvent>>,
    event_questions: Signal<Vec<EventQuestion>>,
    error_message: Signal<Option<String>>,
    is_connecting: Signal<bool>,
}

// =============================================================================
// ENTRY POINT
// =============================================================================

fn main() {
    dioxus::launch(App);
}

// =============================================================================
// APP ROOT
// =============================================================================

#[component]
fn App() -> Element {
    let state = AppState {
        view_state: use_signal(|| ViewState::Loading),
        user_profile: use_signal(|| None),
        active_event: use_signal(|| None),
        event_questions: use_signal(Vec::new),
        error_message: use_signal(|| None),
        is_connecting: use_signal(|| true),
    };

    use_context_provider(|| state);

    use_future(move || async move {
        connect_to_spacetimedb(state).await;
    });

    let current_state = (state.view_state)();
    let error = (state.error_message)();
    let connecting = (state.is_connecting)();

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div {
            class: "min-h-screen bg-gradient-to-br from-slate-900 via-indigo-950 to-slate-900 text-white flex flex-col",

            // Header
            header {
                class: "bg-black/30 backdrop-blur-md border-b border-white/10 sticky top-0 z-50",
                div {
                    class: "max-w-4xl mx-auto px-4 py-4 flex items-center justify-between",
                    h1 {
                        class: "text-2xl md:text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent tracking-tight",
                        "4ever & Beyond"
                    }
                    if connecting {
                        span {
                            class: "text-xs text-indigo-300 animate-pulse font-medium",
                            "Connecting..."
                        }
                    }
                }
            }

            // Error Banner
            {error.map(|msg| rsx! {
                ErrorBanner { message: msg }
            })}

            // Main Content
            main {
                class: "flex-1 max-w-4xl w-full mx-auto px-4 py-8",
                match current_state {
                    ViewState::Loading => rsx! { LoadingView {} },
                    ViewState::Onboarding => rsx! { OnboardingView {} },
                    ViewState::EventView => rsx! { EventView {} },
                    ViewState::Submitted => rsx! { SubmittedView {} },
                }
            }

            // Footer
            footer {
                class: "mt-auto py-6 text-center text-slate-600 text-xs tracking-wide",
                "Built with Rust \u{00b7} SpacetimeDB \u{00b7} Dioxus 0.7"
            }
        }
    }
}

// =============================================================================
// SPACETIMEDB CONNECTION (currently demo mode)
// =============================================================================

/// In production this will connect via WebSocket:
///   let conn = spacetimedb_sdk::connect("ws://localhost:3000", "community-platform", []).await;
///   conn.subscribe("SELECT * FROM event WHERE is_active = true");
///   conn.subscribe("SELECT * FROM event_question");
///   conn.subscribe("SELECT * FROM event_response");
///
/// For now we seed demo data directly into the Dioxus signals.
async fn connect_to_spacetimedb(mut state: AppState) {
    // TODO: Replace with real SpacetimeDB SDK WebSocket connection
    seed_demo_data(state);

    state.is_connecting.set(false);

    let has_profile = (state.user_profile)().is_some();
    state.view_state.set(if has_profile {
        ViewState::EventView
    } else {
        ViewState::Onboarding
    });

    log::info!("SpacetimeDB connection established (demo mode).");
}

fn seed_demo_data(mut state: AppState) {
    state.active_event.set(Some(CommunityEvent {
        id: 1,
        title: "Welcome Dinner 2025 \u{1f35c}".to_string(),
        description: "Join us for the annual welcome dinner! Please select your menu preference and let us know about any dietary restrictions."
            .to_string(),
        event_date: "2025-06-15".to_string(),
        priority: 10,
        is_active: true,
        passcode: "4ever2025".to_string(),
    }));

    state.event_questions.set(vec![
        EventQuestion {
            id: 1,
            event_id: 1,
            label: "Menu Selection".to_string(),
            field_type: "select".to_string(),
            options: Some(
                r#"["Standard","Vegetarian","Halal","Vegan"]"#.to_string(),
            ),
            is_required: true,
        },
        EventQuestion {
            id: 2,
            event_id: 1,
            label: "Any dietary restrictions or allergies?".to_string(),
            field_type: "text".to_string(),
            options: None,
            is_required: false,
        },
        EventQuestion {
            id: 3,
            event_id: 1,
            label: "Will you bring a plus-one?".to_string(),
            field_type: "radio".to_string(),
            options: Some(r#"["Yes","No"]"#.to_string()),
            is_required: true,
        },
    ]);
}

// =============================================================================
// VIEW COMPONENTS
// =============================================================================

fn LoadingView() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-32 space-y-6",
            div {
                class: "relative",
                div {
                    class: "w-16 h-16 border-4 border-indigo-500/30 border-t-indigo-400 rounded-full animate-spin",
                }
            }
            div {
                class: "text-center space-y-2",
                h2 {
                    class: "text-xl font-semibold text-slate-200",
                    "Connecting to the community..."
                }
                p {
                    class: "text-sm text-slate-500",
                    "Verifying your identity with SpacetimeDB"
                }
            }
        }
    }
}

#[component]
fn OnboardingView() -> Element {
    let mut state: AppState = use_context();
    let mut nickname = use_signal(String::new);
    let mut entry_year = use_signal(String::new);
    let mut contact_channel = use_signal(String::new);
    let mut student_id = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let on_submit = move |_| {
        let nick = nickname.read().clone();
        let year = entry_year.read().clone();
        let contact = contact_channel.read().clone();
        let sid = student_id.read().clone();

        if nick.trim().is_empty() {
            state.error_message.set(Some("Nickname is required.".to_string()));
            return;
        }
        if year.trim().is_empty() {
            state.error_message.set(Some("Please select your Year or Alumni status.".to_string()));
            return;
        }
        if contact.trim().is_empty() {
            state.error_message.set(Some("Contact channel (Line ID / IG) is required.".to_string()));
            return;
        }

        is_submitting.set(true);

        // TODO: In production call SpacetimeDB reducer:
        //   conn.call_reducer("register_profile", &[&nick, &year, &contact, &sid_opt]);
        // For now, save to local Dioxus signal only.
        spawn(async move {
            let profile = UserProfile {
                identity: "demo_identity_001".to_string(),
                nickname: nick.trim().to_string(),
                entry_year: year.trim().to_string(),
                contact_channel: contact.trim().to_string(),
                student_id: if sid.trim().is_empty() {
                    None
                } else {
                    Some(sid.trim().to_string())
                },
                is_verified: false,
            };

            let nick_for_log = profile.nickname.clone();
            state.user_profile.set(Some(profile));
            is_submitting.set(false);
            state.view_state.set(ViewState::EventView);
            log::info!("Profile registered: {:?}", nick_for_log);
        });
    };

    rsx! {
        div {
            class: "flex items-center justify-center py-8 px-2",
            div {
                class: "w-full max-w-md",
                div {
                    class: "bg-white/[0.04] backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-2xl",

                    div {
                        class: "text-center mb-8",
                        div {
                            class: "w-16 h-16 bg-indigo-500/20 rounded-full flex items-center justify-center mx-auto mb-4",
                            span { class: "text-3xl", "\u{1f44b}" }
                        }
                        h2 { class: "text-2xl font-bold text-white", "Quick Sign-Up" }
                        p { class: "text-slate-400 text-sm mt-2 leading-relaxed", "Just the basics \u{2014} you can complete your profile later!" }
                    }

                    div {
                        class: "space-y-5",

                        FormField {
                            label: "Nickname",
                            required: true,
                            input_type: "text",
                            placeholder: "What should we call you?",
                            value: nickname,
                            on_change: move |v| nickname.set(v),
                        }

                        div {
                            class: "space-y-1.5",
                            label {
                                class: "block text-sm font-medium text-slate-300",
                                "Year / Alumni"
                                span { class: "text-indigo-400 ml-1", "*" }
                            }
                            select {
                                class: "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition appearance-none cursor-pointer",
                                value: "{entry_year}",
                                onchange: move |e| entry_year.set(e.value()),
                                option { value: "", disabled: true, "Select your year..." }
                                option { value: "Year 1", "Year 1" }
                                option { value: "Year 2", "Year 2" }
                                option { value: "Year 3", "Year 3" }
                                option { value: "Year 4", "Year 4" }
                                option { value: "Year 5+", "Year 5+" }
                                option { value: "Alumni", "Alumni" }
                            }
                        }

                        FormField {
                            label: "Line ID or Instagram",
                            required: true,
                            input_type: "text",
                            placeholder: "@your_handle or Line ID",
                            value: contact_channel,
                            on_change: move |v| contact_channel.set(v),
                        }

                        div {
                            class: "space-y-1.5",
                            label {
                                class: "block text-sm font-medium text-slate-300",
                                "Student ID"
                                span { class: "text-slate-600 ml-1 text-xs", "(optional)" }
                            }
                            input {
                                class: "w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition",
                                r#type: "text",
                                placeholder: "Not required for Alumni",
                                value: "{student_id}",
                                oninput: move |e| student_id.set(e.value()),
                            }
                        }

                        button {
                            class: if is_submitting() {
                                "w-full bg-indigo-500/40 text-indigo-200 font-semibold py-3.5 rounded-xl cursor-not-allowed transition"
                            } else {
                                "w-full bg-indigo-500 hover:bg-indigo-600 active:bg-indigo-700 text-white font-semibold py-3.5 rounded-xl shadow-lg shadow-indigo-500/25 transition-all duration-200 transform hover:scale-[1.01] active:scale-[0.99]"
                            },
                            disabled: is_submitting(),
                            onclick: on_submit,
                            if is_submitting() { "Creating Profile..." } else { "Continue \u{2192}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EventView() -> Element {
    let mut state: AppState = use_context();
    let event = (state.active_event)();
    let questions = (state.event_questions)();
    let profile = (state.user_profile)();

    let mut answers: Signal<Vec<String>> = use_signal(|| {
        questions.iter().map(|_| String::new()).collect()
    });
    let mut passcode = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut passcode_error = use_signal(|| false);

    let on_submit = move |_| {
        let pc = passcode.read().clone();
        let current_answers = answers.read().clone();
        let qs = (state.event_questions)();

        if pc.trim().is_empty() {
            state.error_message.set(Some("Please enter the event passcode.".to_string()));
            passcode_error.set(true);
            return;
        }

        if let Some(ref evt) = (state.active_event)() {
            if pc.trim() != evt.passcode.trim() {
                state.error_message.set(Some(
                    "Invalid passcode. Check the group chat for the correct code.".to_string(),
                ));
                passcode_error.set(true);
                return;
            }
        }

        for (i, q) in qs.iter().enumerate() {
            if q.is_required {
                if let Some(ans) = current_answers.get(i) {
                    if ans.trim().is_empty() {
                        state.error_message.set(Some(format!("Please answer: {}", q.label)));
                        return;
                    }
                }
            }
        }

        passcode_error.set(false);
        is_submitting.set(true);

        // TODO: In production call SpacetimeDB reducer:
        //   conn.call_reducer("submit_response", &[&event_id, &pc, &answers_json]);
        // For now, just flip to submitted view.
        spawn(async move {
            is_submitting.set(false);
            state.view_state.set(ViewState::Submitted);
            log::info!("RSVP submitted successfully.");
        });
    };

    let event_data = match event {
        Some(e) => e,
        None => {
            return rsx! {
                div {
                    class: "text-center py-24",
                    div {
                        class: "w-20 h-20 bg-slate-800 rounded-full flex items-center justify-center mx-auto mb-6",
                        span { class: "text-4xl", "\u{1f4ed}" }
                    }
                    h2 { class: "text-xl text-slate-300 font-semibold", "No active events right now" }
                    p { class: "text-slate-500 mt-2 text-sm", "Check back soon \u{2014} something awesome is coming!" }
                }
            };
        }
    };

    rsx! {
        div {
            class: "space-y-5",

            // Welcome Bar
            {profile.map(|p| rsx! {
                div {
                    class: "flex items-center gap-4 bg-white/[0.03] rounded-xl px-5 py-4 border border-white/10",
                    div {
                        class: "w-11 h-11 bg-indigo-500/20 rounded-full flex items-center justify-center text-lg shrink-0",
                        "\u{1f464}"
                    }
                    div {
                        class: "min-w-0",
                        p { class: "text-xs text-slate-500 truncate", "Welcome back," }
                        p { class: "font-semibold text-white truncate", "{p.nickname}" }
                    }
                    if p.is_verified {
                        span {
                            class: "ml-auto text-xs bg-emerald-500/15 text-emerald-400 px-2.5 py-1 rounded-full shrink-0 font-medium",
                            "\u{2713} Verified"
                        }
                    } else {
                        span {
                            class: "ml-auto text-xs bg-amber-500/15 text-amber-400 px-2.5 py-1 rounded-full shrink-0 font-medium",
                            "\u{23f3} Pending"
                        }
                    }
                }
            })}

            // Event Card
            div {
                class: "bg-gradient-to-br from-indigo-500/[0.07] to-purple-500/[0.07] border border-indigo-500/20 rounded-2xl overflow-hidden shadow-xl",

                div {
                    class: "bg-indigo-500/[0.08] px-6 py-5 border-b border-indigo-500/10",
                    div {
                        class: "flex items-start justify-between gap-4 flex-wrap",
                        div {
                            h2 { class: "text-2xl font-bold text-white leading-tight", "{event_data.title}" }
                            p { class: "text-slate-400 text-sm mt-2 leading-relaxed max-w-lg", "{event_data.description}" }
                        }
                        span {
                            class: "flex-shrink-0 bg-emerald-500/15 text-emerald-400 text-xs font-semibold px-3 py-1.5 rounded-full tracking-wide",
                            "\u{25cf} ACTIVE"
                        }
                    }
                    div {
                        class: "mt-4 flex items-center gap-2 text-sm text-slate-500",
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
                }

                // Dynamic Questions
                div {
                    class: "px-6 py-6 space-y-6",

                    h3 {
                        class: "text-lg font-semibold text-white flex items-center gap-2",
                        span { "\u{1f4dd}" }
                        "RSVP Questions"
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
                                        span { class: "text-red-400 ml-1", "*" }
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
                                                    option { value: "", disabled: true, "Choose an option..." }
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

                    // Passcode Security Gate
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
                                "Event Passcode"
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
                            placeholder: "Enter the code shared in the group chat",
                            value: "{passcode}",
                            oninput: move |e| {
                                passcode.set(e.value());
                                passcode_error.set(false);
                                state.error_message.set(None);
                            },
                        }
                        p { class: "text-xs text-slate-600 mt-2 leading-relaxed", "\u{1f512} This code prevents unauthorized RSVPs. Ask an organizer if you don't have it." }
                    }

                    // Submit Button
                    button {
                        class: if is_submitting() {
                            "w-full bg-indigo-500/40 text-indigo-200 font-bold py-4 rounded-xl cursor-not-allowed text-lg transition"
                        } else {
                            "w-full bg-gradient-to-r from-indigo-500 to-purple-500 hover:from-indigo-600 hover:to-purple-600 active:from-indigo-700 active:to-purple-700 text-white font-bold py-4 rounded-xl shadow-lg shadow-indigo-500/25 transition-all duration-200 transform hover:scale-[1.01] active:scale-[0.99] text-lg"
                        },
                        disabled: is_submitting(),
                        onclick: on_submit,
                        if is_submitting() { "Submitting RSVP..." } else { "Confirm RSVP \u{1f389}" }
                    }
                }
            }
        }
    }
}

#[component]
fn SubmittedView() -> Element {
    let state: AppState = use_context();
    let event = (state.active_event)();
    let profile = (state.user_profile)();

    rsx! {
        div {
            class: "flex items-center justify-center py-20 px-4",
            div {
                class: "text-center max-w-md",
                div {
                    class: "w-28 h-28 bg-emerald-500/15 rounded-full flex items-center justify-center mx-auto mb-8",
                    span { class: "text-6xl", "\u{1f389}" }
                }
                h2 { class: "text-3xl font-extrabold text-white mb-3 tracking-tight", "You're In!" }
                p { class: "text-slate-400 text-lg mb-8", "Your RSVP has been confirmed." }

                {event.map(|e| rsx! {
                    div {
                        class: "bg-white/[0.04] rounded-2xl px-8 py-6 inline-block border border-white/10",
                        p { class: "text-xs text-slate-500 uppercase tracking-widest mb-1", "Event" }
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
                    }
                })}

                {profile.map(|p| rsx! {
                    div {
                        class: "mt-6 text-sm text-slate-500",
                        "Registered as: "
                        span { class: "text-slate-300 font-medium", "{p.nickname}" }
                        " ({p.entry_year})"
                    }
                })}

                div {
                    class: "mt-10",
                    p { class: "text-slate-500 text-base", "See you there! \u{1f680}" }
                }
            }
        }
    }
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
                    "\u{2715}"
                }
            }
        }
    }
}
