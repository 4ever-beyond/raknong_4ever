//!
//! 4ever & Beyond — Community Platform Frontend
//!
//! Built with Dioxus 0.7 + Tailwind CSS
//! Connects to SpacetimeDB backend for persistent state.
//!
//! View State Machine:
//!   Loading → Onboarding → EventView → Submitted
//!
//! i18n: All user-facing strings come from `i18n.rs` via the `Locale` struct.
//!       Switch languages at runtime with the header toggle.

mod i18n;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{get_locale, Language};

// =============================================================================
// CONSTANTS
// =============================================================================

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

const INSTAGRAM_URL: &str = "https://www.instagram.com/raknong_4ever";
const MENU_URL: &str = "https://linktr.ee/steakdekuanwattana";

// =============================================================================
// MODELS — Mirror the SpacetimeDB tables
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    pub identity: String,
    pub nickname: String,
    pub entry_year: String,
    pub phone: String,
    pub instagram: String,
    pub line_id: String,
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
#[derive(Clone, Copy)]
struct AppState {
    view_state: Signal<ViewState>,
    user_profile: Signal<Option<UserProfile>>,
    active_event: Signal<Option<CommunityEvent>>,
    event_questions: Signal<Vec<EventQuestion>>,
    error_message: Signal<Option<String>>,
    is_connecting: Signal<bool>,
    language: Signal<Language>,
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
    let mut state = AppState {
        view_state: use_signal(|| ViewState::Loading),
        user_profile: use_signal(|| None),
        active_event: use_signal(|| None),
        event_questions: use_signal(Vec::new),
        error_message: use_signal(|| None),
        is_connecting: use_signal(|| true),
        language: use_signal(|| Language::Thai),
    };

    use_context_provider(|| state);

    use_future(move || async move {
        connect_to_spacetimedb(state).await;
    });

    let lang = (state.language)();
    let locale = get_locale(lang);
    let current_state = (state.view_state)();
    let error = (state.error_message)();
    let connecting = (state.is_connecting)();

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
                    // Right side: lang toggle, IG link, status
                    div {
                        class: "flex items-center gap-3",
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
                        // Connecting indicator
                        if connecting {
                            span {
                                class: "text-xs text-indigo-300 animate-pulse font-medium",
                                "{locale.connecting}"
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
                match current_state {
                    ViewState::Loading => rsx! { LoadingView {} },
                    ViewState::Onboarding => rsx! { OnboardingView {} },
                    ViewState::EventView => rsx! { EventView {} },
                    ViewState::Submitted => rsx! { SubmittedView {} },
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
// SPACETIMEDB CONNECTION (currently demo mode)
// =============================================================================

async fn connect_to_spacetimedb(mut state: AppState) {
    // TODO: Replace with real SpacetimeDB SDK WebSocket connection
    //   let conn = spacetimedb_sdk::connect("ws://localhost:3000", "community-platform", []).await;
    //   conn.subscribe("SELECT * FROM event WHERE is_active = true");
    //   conn.subscribe("SELECT * FROM event_question");
    //   conn.subscribe("SELECT * FROM event_response");
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
        title: "4EVER รวมตัวกินสเต็กเด็กอ้วน 🥩".to_string(),
        description:
            "รวมตัวกินสเต็กเด็กอ้วน ศาลายา ซอย 11\n\n🚗 มีที่จอดรถ ร้านอยู่ท้ายซอย\n\n{locale.data_safe}"
                .replace(
                    "{locale.data_safe}",
                    get_locale((state.language)()).data_safe,
                ),
        event_date: "08-04-2569".to_string(),
        priority: 10,
        is_active: true,
        passcode: "4ever2026".to_string(),
    }));

    state.event_questions.set(vec![
        EventQuestion {
            id: 1,
            event_id: 1,
            label: "เห็นข่าวการเรียกรวมตัวจากที่ไหนเอ่ย".to_string(),
            field_type: "select".to_string(),
            options: Some(
                r#"["กลุ่มไลน์","อินสตาแกรม","เพื่อนบอก","Facebook","อื่นๆ"]"#.to_string(),
            ),
            is_required: true,
        },
        EventQuestion {
            id: 2,
            event_id: 1,
            label: "เมนูที่จะกินค่าาา".to_string(),
            field_type: "select".to_string(),
            options: Some(
                r#"["สเต็กหมู S","สเต็กหมู M","สเต็กหมู L","สเต็กไก่ S","สเต็กไก่ M","สเต็กไก่ L","สเต็กปลาแซลมอน","เมนูอื่นๆ"]"#
                    .to_string(),
            ),
            is_required: true,
        },
    ]);
}

// =============================================================================
// VIEW COMPONENTS
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

        // TODO: In production call SpacetimeDB reducer:
        //   conn.call_reducer("register_profile", &[&nick, &year, &ph, &ig, &line]);
        spawn(async move {
            let profile = UserProfile {
                identity: "demo_identity_001".to_string(),
                nickname: nick.trim().to_string(),
                entry_year: year.trim().to_string(),
                phone: ph.trim().to_string(),
                instagram: ig.trim().to_string(),
                line_id: line.trim().to_string(),
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

                        // 1. Nickname / Name
                        FormField {
                            label: locale.nickname_label.to_string(),
                            required: true,
                            input_type: "text".to_string(),
                            placeholder: locale.nickname_placeholder.to_string(),
                            value: nickname,
                            on_change: move |v| nickname.set(v),
                        }

                        // 2. Year / Alumni (select)
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

                        // Submit button
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

                    // Safety note
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

        if let Some(ref evt) = (state.active_event)() {
            if pc.trim() != evt.passcode.trim() {
                state
                    .error_message
                    .set(Some(loc.err_passcode_invalid.to_string()));
                passcode_error.set(true);
                return;
            }
        }

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

        // TODO: In production call SpacetimeDB reducer:
        //   conn.call_reducer("submit_response", &[&event_id, &pc, &answers_json]);
        spawn(async move {
            is_submitting.set(false);
            state.view_state.set(ViewState::Submitted);
            log::info!("RSVP submitted successfully.");
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
                        // Date
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
                        // Menu & Location link
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

                    // Safety reassurance
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

                        // Menu link
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
