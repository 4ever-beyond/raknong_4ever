//! Internationalization (i18n) module for the 4ever & Beyond community platform.
//!
//! Provides a complete set of localized UI strings for Thai and English.
//! All translatable text is centralized here so that views remain locale-agnostic.

// ─── Imports ──────────────────────────────────────────────────────────────────

// ─── Language ─────────────────────────────────────────────────────────────────

/// Supported languages for the application.
///
/// Derives `Clone`, `Copy`, and `PartialEq` so it can be stored efficiently in
/// Dioxus signals and compared in `match` / `if` expressions.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Language {
    Thai,
    English,
}

impl Language {
    /// Returns the opposite language, used by the language toggle button.
    pub fn toggle(&self) -> Self {
        match self {
            Language::Thai => Language::English,
            Language::English => Language::Thai,
        }
    }
}

// ─── Locale ───────────────────────────────────────────────────────────────────

/// A bundle of every translatable string in the application.
///
/// All fields are `&'static str` so the struct is `Copy`-friendly and never
/// allocates at runtime. Construct an instance via [`get_locale`].
///
/// # Sections
///
/// | Section              | Prefix / Group        |
/// |----------------------|-----------------------|
/// | Branding             | `app_name`, `tagline` |
/// | Loading view         | `loading_*`           |
/// | Onboarding form      | `onboard_*`, `*_label` / `*_placeholder` |
/// | Onboarding errors    | `err_*`               |
/// | Event view           | `welcome_back`, `rsvp_*`, etc. |
/// | No-events fallback   | `no_events_*`         |
/// | Passcode errors      | `err_passcode_*`      |
/// | Submitted / success  | `submitted_*`         |
/// | Form utilities       | `optional_label`, etc. |
#[derive(Clone, Debug)]
#[allow(dead_code)] // fields are read from main.rs; LSP reports false positives
pub struct Locale {
    // ── Branding ──────────────────────────────────────────────────────────
    /// Application display name.
    pub app_name: &'static str,
    /// Short marketing tagline beneath the app name.
    pub tagline: &'static str,
    /// Connecting / loading indicator text.
    pub connecting: &'static str,
    /// Footer credits line.
    pub footer_text: &'static str,
    /// Data-safety reassurance message.
    pub data_safe: &'static str,

    // ── Loading View ──────────────────────────────────────────────────────
    /// Primary loading message.
    pub loading_title: &'static str,
    /// Secondary loading message (identity verification).
    pub loading_subtitle: &'static str,

    // ── Onboarding ────────────────────────────────────────────────────────
    /// Onboarding card heading.
    pub onboard_title: &'static str,
    /// Onboarding card subtitle / helper text.
    pub onboard_subtitle: &'static str,
    /// Label for the nickname / name field.
    pub nickname_label: &'static str,
    /// Placeholder inside the nickname input.
    pub nickname_placeholder: &'static str,
    /// Label for the academic-year selector.
    pub year_label: &'static str,
    /// Placeholder inside the year selector.
    pub year_placeholder: &'static str,
    /// Label for the phone-number field.
    pub phone_label: &'static str,
    /// Placeholder inside the phone input.
    pub phone_placeholder: &'static str,
    /// Label for the Instagram handle field.
    pub instagram_label: &'static str,
    /// Placeholder inside the Instagram input.
    pub instagram_placeholder: &'static str,
    /// Label for the Line ID field.
    pub line_label: &'static str,
    /// Placeholder inside the Line ID input.
    pub line_placeholder: &'static str,
    /// Call-to-action button on the onboarding form.
    pub continue_button: &'static str,
    /// Loading state while the profile is being created.
    pub creating_profile: &'static str,

    // ── Onboarding Errors ─────────────────────────────────────────────────
    /// Error when nickname is empty.
    pub err_nickname_required: &'static str,
    /// Error when year is not selected.
    pub err_year_required: &'static str,
    /// Error when phone number is empty.
    pub err_phone_required: &'static str,
    /// Error when Instagram handle is empty.
    pub err_instagram_required: &'static str,
    /// Error when Line ID is empty.
    pub err_line_required: &'static str,

    // ── Event View ────────────────────────────────────────────────────────
    /// Greeting shown to returning users.
    pub welcome_back: &'static str,
    /// Badge text for an active event.
    pub active_badge: &'static str,
    /// Heading above RSVP questions.
    pub rsvp_title: &'static str,
    /// Label for the event passcode field.
    pub passcode_label: &'static str,
    /// Placeholder inside the passcode input.
    pub passcode_placeholder: &'static str,
    /// Helper / security hint beneath the passcode field.
    pub passcode_hint: &'static str,
    /// Submit button for RSVP confirmation.
    pub submit_rsvp: &'static str,
    /// Loading state while submitting RSVP.
    pub submitting: &'static str,
    /// Badge for a verified RSVP.
    pub verified: &'static str,
    /// Badge for a pending RSVP.
    pub pending: &'static str,
    /// Link / button to view menu and location details.
    pub view_menu: &'static str,

    // ── No Events ─────────────────────────────────────────────────────────
    /// Title shown when there are no active events.
    pub no_events_title: &'static str,
    /// Subtitle / encouragement when there are no events.
    pub no_events_subtitle: &'static str,

    // ── Passcode Errors ───────────────────────────────────────────────────
    /// Error when passcode is empty.
    pub err_passcode_required: &'static str,
    /// Error when passcode does not match.
    pub err_passcode_invalid: &'static str,
    /// Prefix prepended to a required-question error ("Please answer: ").
    pub err_answer_prefix: &'static str,

    // ── Submitted / Success ───────────────────────────────────────────────
    /// Heading after successful RSVP.
    pub submitted_title: &'static str,
    /// Confirmation message after RSVP.
    pub submitted_subtitle: &'static str,
    /// Label preceding the event name.
    pub event_label: &'static str,
    /// Prefix before the user's registered name.
    pub registered_as: &'static str,
    /// Closing congratulatory message.
    pub see_you: &'static str,

    // ── Form Utilities ────────────────────────────────────────────────────
    /// Tag appended to optional fields.
    pub optional_label: &'static str,
    /// Placeholder for generic `<select>` dropdowns.
    pub choose_option: &'static str,

    // ── Language Toggle ───────────────────────────────────────────────────
    /// Label for the Thai language option.
    pub lang_toggle_th: &'static str,
    /// Label for the English language option.
    pub lang_toggle_en: &'static str,

    // ── Required Marker ───────────────────────────────────────────────────
    /// Visual marker appended to required-field labels.
    pub required_marker: &'static str,

    // ── Admin Dashboard ──────────────────────────────────────────────────
    /// Admin dashboard title
    pub admin_title: &'static str,
    /// Total users stat label
    pub admin_total_users: &'static str,
    /// Total responses stat label
    pub admin_total_responses: &'static str,
    /// Verified users label
    pub admin_verified_users: &'static str,
    /// Pending users label
    pub admin_pending_users: &'static str,
    /// Users table column header
    pub admin_col_nickname: &'static str,
    /// Year column
    pub admin_col_year: &'static str,
    /// Phone column
    pub admin_col_phone: &'static str,
    /// Instagram column
    pub admin_col_instagram: &'static str,
    /// Line column
    pub admin_col_line: &'static str,
    /// Status column
    pub admin_col_status: &'static str,
    /// Action column
    pub admin_col_action: &'static str,
    /// Responses tab
    pub admin_responses_tab: &'static str,
    /// Users tab
    pub admin_users_tab: &'static str,
    /// Toggle verify button text
    pub admin_toggle_verify: &'static str,
    /// Revoke verify button text
    pub admin_revoke_verify: &'static str,
    /// Response nickname column
    pub admin_col_responder: &'static str,
    /// Response answers column
    pub admin_col_answers: &'static str,
    /// Response date column
    pub admin_col_submitted_at: &'static str,
    /// Delete response button
    pub admin_delete_response: &'static str,
    /// Admin nav button text (shown in header)
    pub admin_nav_button: &'static str,
    /// Back to event button
    pub admin_back_to_event: &'static str,
    /// No responses yet
    pub admin_no_responses: &'static str,
    /// No users yet
    pub admin_no_users: &'static str,

    // ── Menu Selector ──────────────────────────────────────────────────
    /// Search placeholder for menu dropdown
    pub menu_search_placeholder: &'static str,
    /// "items selected" suffix
    pub menu_items_selected: &'static str,
    /// Total label
    pub menu_total: &'static str,
    /// Currency symbol (Thai Baht)
    pub menu_currency: &'static str,
    /// No results found text
    pub menu_no_results: &'static str,

    // ── Menu Management (Admin) ────────────────────────────────────────
    /// Menu management tab title
    pub admin_menu_tab: &'static str,
    /// Menu management title
    pub admin_menu_title: &'static str,
    /// Menu item name label
    pub admin_menu_name_label: &'static str,
    /// Menu item name placeholder
    pub admin_menu_name_placeholder: &'static str,
    /// Menu item price label
    pub admin_menu_price_label: &'static str,
    /// Menu item price placeholder
    pub admin_menu_price_placeholder: &'static str,
    /// Add menu item button
    pub admin_menu_add: &'static str,
    /// Save menu item button
    pub admin_menu_save: &'static str,
    /// Cancel edit button
    pub admin_menu_cancel: &'static str,
    /// Edit menu item button
    pub admin_menu_edit: &'static str,
    /// Delete menu item button
    pub admin_menu_delete: &'static str,
    /// No menu items text
    pub admin_menu_no_items: &'static str,
    /// Menu item active status
    pub admin_menu_active: &'static str,
    /// Menu item inactive status
    pub admin_menu_inactive: &'static str,
    /// Total menu items count label
    pub admin_menu_total_items: &'static str,
    /// Menu item category label
    pub admin_menu_category_label: &'static str,
    /// Menu item category placeholder
    pub admin_menu_category_placeholder: &'static str,

    // ── Admin Authentication ───────────────────────────────────────────
    /// Admin auth dialog title
    pub admin_auth_title: &'static str,
    /// Admin passcode label
    pub admin_auth_passcode_label: &'static str,
    /// Admin passcode placeholder
    pub admin_auth_passcode_placeholder: &'static str,
    /// Admin auth submit button
    pub admin_auth_submit: &'static str,
    /// Admin auth error message
    pub admin_auth_error: &'static str,

    // ── Delete Confirmation ───────────────────────────────────────────
    /// Delete confirmation dialog title
    pub admin_delete_confirm_title: &'static str,
    /// Delete confirmation message (use '{name}' placeholder)
    pub admin_delete_confirm_message: &'static str,
    /// Delete confirmation button (destructive)
    pub admin_delete_confirm_button: &'static str,
    /// Delete confirmation cancel button
    pub admin_delete_cancel_button: &'static str,

    // ── Loading States ────────────────────────────────────────────────
    /// Generic menu loading indicator text
    pub admin_menu_loading: &'static str,

    // ── CSV Export ────────────────────────────────────────────────────
    /// Export CSV button label (includes emoji)
    pub admin_export_csv: &'static str,
    /// Export error message prefix
    pub admin_export_error: &'static str,

    // ── Event Management (Admin) ──────────────────────────────────────
    /// Events tab label
    pub admin_events_tab: &'static str,
    /// Event management section title
    pub admin_events_title: &'static str,
    /// Create event button text
    pub admin_events_add: &'static str,
    /// Event title field label
    pub admin_events_title_label: &'static str,
    /// Event description field label
    pub admin_events_desc_label: &'static str,
    /// Event date field label
    pub admin_events_date_label: &'static str,
    /// Event passcode field label
    pub admin_events_passcode_label: &'static str,
    /// No events placeholder text
    pub admin_events_no_events: &'static str,
    /// Create new event form heading
    pub admin_events_create_title: &'static str,
    /// Event selector label (shown in EventView when multiple events)
    pub admin_event_select_label: &'static str,
}

impl Locale {
    /// Returns the list of academic-year options for the onboarding form.
    ///
    /// Thai: `["ปี 1", "ปี 2", "ปี 3", "ปี 4", "ปี 5+", "ศิษย์เก่า"]`
    ///
    /// English: `["Year 1", "Year 2", "Year 3", "Year 4", "Year 5+", "Alumni"]`
    pub fn year_options(&self) -> Vec<&'static str> {
        // The Thai locale carries Thai year labels; otherwise use English.
        if self.year_placeholder == "เลือกชั้นปี..." {
            vec!["ปี 1", "ปี 2", "ปี 3", "ปี 4", "ปี 5+", "ศิษย์เก่า"]
        } else {
            vec!["Year 1", "Year 2", "Year 3", "Year 4", "Year 5+", "Alumni"]
        }
    }
}

// ─── Locale Factory ───────────────────────────────────────────────────────────

/// Build a [`Locale`] for the given [`Language`].
///
/// Every UI string in the application flows through this function, making it
/// the single source of truth for translations.
pub fn get_locale(lang: Language) -> Locale {
    match lang {
        // ──────────────────────────────────────────────────────────────────
        //  Thai (TH)
        // ──────────────────────────────────────────────────────────────────
        Language::Thai => Locale {
            // Branding
            app_name: "4ever & Beyond",
            tagline: "รวมตัว สังสรรค์ กินสเต็ก!",
            connecting: "กำลังเชื่อมต่อ...",
            footer_text: "สร้างด้วย Rust · PostgreSQL · Dioxus",
            data_safe: "ข้อมูลปลอดภัยแน่นอนจ้ะ พี่โอโซนรับประกัน",

            // Loading View
            loading_title: "กำลังเชื่อมต่อกับชุมชน...",
            loading_subtitle: "กำลังโหลดข้อมูล...",

            // Onboarding
            onboard_title: "ลงทะเบียนเข้าร่วม",
            onboard_subtitle: "ข้อมูลพื้นฐาน — แก้ไขได้ภายหลัง!",
            nickname_label: "ชื่อเล่นจ้า",
            nickname_placeholder: "Four",
            year_label: "ตอนนี้เรียนปีไหนเอ่ย",
            year_placeholder: "เลือกชั้นปี...",
            phone_label: "ขอเบอร์หน่อยจ้า",
            phone_placeholder: "0xx-xxx-xxxx",
            instagram_label: "ไอจีๆๆ",
            instagram_placeholder: "@your_ig",
            line_label: "ไลน์ด้วยเผื่อตามตัว",
            line_placeholder: "Line ID",
            continue_button: "ดำเนินการต่อ →",
            creating_profile: "กำลังสร้างโปรไฟล์...",

            // Onboarding Errors
            err_nickname_required: "กรุณาใส่ชื่อจ้า",
            err_year_required: "เลือกชั้นปีด้วยจ้า",
            err_phone_required: "ขอเบอร์ด้วยน้า",
            err_instagram_required: "ใส่ไอจีด้วยจ้า",
            err_line_required: "ใส่ไลน์ไอดีด้วยน้า",

            // Event View
            welcome_back: "สวัสดีจ้า,",
            active_badge: "● กำลังดำเนินการ",
            rsvp_title: "คำถามลงทะเบียน",
            passcode_label: "รหัสผ่านกิจกรรม",
            passcode_placeholder: "ใส่รหัสที่แชร์ในกลุ่ม",
            passcode_hint: "🔒 รหัสนี้ป้องกันการลงทะเบียนไม่ได้รับอนุญาต ถามผู้จัดถ้าไม่มีรหัส",
            submit_rsvp: "ยืนยันลงทะเบียน 🎉",
            submitting: "กำลังส่งข้อมูล...",
            verified: "✓ ยืนยันแล้ว",
            pending: "⏳ รอตรวจสอบ",
            view_menu: "📋 ดูเมนูและแผนที่",

            // No Events
            no_events_title: "ไม่มีกิจกรรมในขณะนี้",
            no_events_subtitle: "กลับมาตรวจสอบอีกครั้ง — มีอะไรดีๆ กำลังจะมา!",

            // Passcode Errors
            err_passcode_required: "กรุณาใส่รหัสผ่านกิจกรรม",
            err_passcode_invalid: "รหัสผ่านไม่ถูกต้อง ตรวจสอบในกลุ่มแชท",
            err_answer_prefix: "กรุณาตอบ: ",

            // Submitted
            submitted_title: "ลงทะเบียนสำเร็จ!",
            submitted_subtitle: "การลงทะเบียนของคุณได้รับการยืนยันแล้ว",
            event_label: "กิจกรรม",
            registered_as: "ลงทะเบียนในชื่อ: ",
            see_you: "แล้วเจอกันใหม่! 🚀",

            // Form
            optional_label: "(ไม่จำเป็น)",
            choose_option: "เลือกตัวเลือก...",

            // Language Toggle
            lang_toggle_th: "ภาษาไทย",
            lang_toggle_en: "English",

            // Required Marker
            required_marker: " *",

            // Admin Dashboard
            admin_title: "แผงควบคุมผู้ดูแล",
            admin_total_users: "สมาชิกทั้งหมด",
            admin_total_responses: "การตอบรับทั้งหมด",
            admin_verified_users: "ยืนยันแล้ว",
            admin_pending_users: "รอตรวจสอบ",
            admin_col_nickname: "ชื่อเล่น",
            admin_col_year: "ชั้นปี",
            admin_col_phone: "เบอร์โทร",
            admin_col_instagram: "IG",
            admin_col_line: "Line",
            admin_col_status: "สถานะ",
            admin_col_action: "การกระทำ",
            admin_responses_tab: "การตอบรับ",
            admin_users_tab: "สมาชิก",
            admin_toggle_verify: "✓ ยืนยัน",
            admin_revoke_verify: "✕ เพิกถอน",
            admin_col_responder: "ผู้ตอบ",
            admin_col_answers: "คำตอบ",
            admin_col_submitted_at: "ส่งเมื่อ",
            admin_delete_response: "🗑 ลบ",
            admin_nav_button: "🔧 แอดมิน",
            admin_back_to_event: "← กลับไปกิจกรรม",
            admin_no_responses: "ยังไม่มีการตอบรับ",
            admin_no_users: "ยังไม่มีสมาชิก",

            // Menu Selector
            menu_search_placeholder: "ค้นหาเมนู...",
            menu_items_selected: "รายการที่เลือก",
            menu_total: "รวมทั้งหมด",
            menu_currency: "฿",
            menu_no_results: "ไม่พบเมนูที่ค้นหา",

            // Menu Management (Admin)
            admin_menu_tab: "เมนูอาหาร",
            admin_menu_title: "จัดการเมนูอาหาร",
            admin_menu_name_label: "ชื่อเมนู",
            admin_menu_name_placeholder: "สเต็กหมู ขนาด M",
            admin_menu_price_label: "ราคา (฿)",
            admin_menu_price_placeholder: "139",
            admin_menu_add: "+ เพิ่มเมนู",
            admin_menu_save: "บันทึก",
            admin_menu_cancel: "ยกเลิก",
            admin_menu_edit: "✏️ แก้ไข",
            admin_menu_delete: "🗑 ลบ",
            admin_menu_no_items: "ยังไม่มีเมนูอาหาร",
            admin_menu_active: "พร้อมให้บริการ",
            admin_menu_inactive: "ไม่พร้อมให้บริการ",
            admin_menu_total_items: "เมนูทั้งหมด",
            admin_menu_category_label: "หมวดหมู่",
            admin_menu_category_placeholder: "เช่น Steaks, Drinks, Desserts",

            // Admin Authentication
            admin_auth_title: "เข้าถึงแผงควบคุม",
            admin_auth_passcode_label: "ใส่รหัสผ่านแอดมิน",
            admin_auth_passcode_placeholder: "รหัสผ่านแอดมิน",
            admin_auth_submit: "เข้าสู่ระบบ",
            admin_auth_error: "รหัสผ่านแอดมินไม่ถูกต้อง",

            // Delete Confirmation
            admin_delete_confirm_title: "ยืนยันการลบ",
            admin_delete_confirm_message: "คุณแน่ใจหรือไม่ว่าต้องการลบ '{name}'?",
            admin_delete_confirm_button: "ลบถาวร",
            admin_delete_cancel_button: "ยกเลิก",

            // Loading States
            admin_menu_loading: "กำลังดำเนินการ...",

            // CSV Export
            admin_export_csv: "📥 ส่งออก CSV",
            admin_export_error: "ส่งออกล้มเหลว",

            // Event Management (Admin)
            admin_events_tab: "กิจกรรม",
            admin_events_title: "จัดการกิจกรรม",
            admin_events_add: "+ สร้างกิจกรรม",
            admin_events_title_label: "ชื่อกิจกรรม",
            admin_events_desc_label: "รายละเอียด",
            admin_events_date_label: "วันที่",
            admin_events_passcode_label: "รหัสผ่านกิจกรรม",
            admin_events_no_events: "ยังไม่มีกิจกรรม",
            admin_events_create_title: "สร้างกิจกรรมใหม่",
            admin_event_select_label: "เลือกกิจกรรม",
        },

        // ──────────────────────────────────────────────────────────────────
        //  English (EN)
        // ──────────────────────────────────────────────────────────────────
        Language::English => Locale {
            // Branding
            app_name: "4ever & Beyond",
            tagline: "Gather. Connect. Feast!",
            connecting: "Connecting...",
            footer_text: "Built with Rust · PostgreSQL · Dioxus",
            data_safe: "Your data is safe — guaranteed by P'Ozone",

            // Loading View
            loading_title: "Connecting to the community...",
            loading_subtitle: "Loading data...",

            // Onboarding
            onboard_title: "Sign Up to Join",
            onboard_subtitle: "Just the basics — you can edit later!",
            nickname_label: "Name (nickname, first name, or full name — anything works!)",
            nickname_placeholder: "Four",
            year_label: "What year are you in?",
            year_placeholder: "Select your year...",
            phone_label: "Phone number",
            phone_placeholder: "0xx-xxx-xxxx",
            instagram_label: "Instagram",
            instagram_placeholder: "@your_ig",
            line_label: "Line ID",
            line_placeholder: "Line ID",
            continue_button: "Continue →",
            creating_profile: "Creating Profile...",

            // Onboarding Errors
            err_nickname_required: "Please enter your name.",
            err_year_required: "Please select your year.",
            err_phone_required: "Phone number is required.",
            err_instagram_required: "Instagram handle is required.",
            err_line_required: "Line ID is required.",

            // Event View
            welcome_back: "Welcome back,",
            active_badge: "● ACTIVE",
            rsvp_title: "RSVP Questions",
            passcode_label: "Event Passcode",
            passcode_placeholder: "Enter the code shared in the group chat",
            passcode_hint:
                "🔒 This code prevents unauthorized RSVPs. Ask an organizer if you don't have it.",
            submit_rsvp: "Confirm RSVP 🎉",
            submitting: "Submitting RSVP...",
            verified: "✓ Verified",
            pending: "⏳ Pending",
            view_menu: "📋 View Menu & Location",

            // No Events
            no_events_title: "No active events right now",
            no_events_subtitle: "Check back soon — something awesome is coming!",

            // Passcode Errors
            err_passcode_required: "Please enter the event passcode.",
            err_passcode_invalid: "Invalid passcode. Check the group chat for the correct code.",
            err_answer_prefix: "Please answer: ",

            // Submitted
            submitted_title: "You're In!",
            submitted_subtitle: "Your RSVP has been confirmed.",
            event_label: "Event",
            registered_as: "Registered as: ",
            see_you: "See you there! 🚀",

            // Form
            optional_label: "(optional)",
            choose_option: "Choose an option...",

            // Language Toggle
            lang_toggle_th: "ภาษาไทย",
            lang_toggle_en: "English",

            // Required Marker
            required_marker: " *",

            // Admin Dashboard
            admin_title: "Admin Dashboard",
            admin_total_users: "Total Users",
            admin_total_responses: "Total Responses",
            admin_verified_users: "Verified",
            admin_pending_users: "Pending",
            admin_col_nickname: "Nickname",
            admin_col_year: "Year",
            admin_col_phone: "Phone",
            admin_col_instagram: "IG",
            admin_col_line: "Line",
            admin_col_status: "Status",
            admin_col_action: "Action",
            admin_responses_tab: "Responses",
            admin_users_tab: "Users",
            admin_toggle_verify: "✓ Verify",
            admin_revoke_verify: "✕ Revoke",
            admin_col_responder: "Responder",
            admin_col_answers: "Answers",
            admin_col_submitted_at: "Submitted",
            admin_delete_response: "🗑 Delete",
            admin_nav_button: "🔧 Admin",
            admin_back_to_event: "← Back to Event",
            admin_no_responses: "No responses yet",
            admin_no_users: "No users yet",

            // Menu Selector
            menu_search_placeholder: "Search menu...",
            menu_items_selected: "selected",
            menu_total: "Total",
            menu_currency: "฿",
            menu_no_results: "No menu items found",

            // Menu Management (Admin)
            admin_menu_tab: "Menu",
            admin_menu_title: "Menu Management",
            admin_menu_name_label: "Menu Item",
            admin_menu_name_placeholder: "Pork Steak M",
            admin_menu_price_label: "Price (฿)",
            admin_menu_price_placeholder: "139",
            admin_menu_add: "+ Add Item",
            admin_menu_save: "Save",
            admin_menu_cancel: "Cancel",
            admin_menu_edit: "✏️ Edit",
            admin_menu_delete: "🗑 Delete",
            admin_menu_no_items: "No menu items yet",
            admin_menu_active: "Available",
            admin_menu_inactive: "Unavailable",
            admin_menu_total_items: "Total Items",
            admin_menu_category_label: "Category",
            admin_menu_category_placeholder: "e.g. Steaks, Drinks, Desserts",

            // Admin Authentication
            admin_auth_title: "Admin Access",
            admin_auth_passcode_label: "Enter admin passcode",
            admin_auth_passcode_placeholder: "Admin passcode",
            admin_auth_submit: "Enter",
            admin_auth_error: "Invalid admin passcode",

            // Delete Confirmation
            admin_delete_confirm_title: "Confirm Delete",
            admin_delete_confirm_message: "Are you sure you want to delete '{name}'?",
            admin_delete_confirm_button: "Delete Permanently",
            admin_delete_cancel_button: "Cancel",

            // Loading States
            admin_menu_loading: "Processing...",

            // CSV Export
            admin_export_csv: "📥 Export CSV",
            admin_export_error: "Export failed",

            // Event Management (Admin)
            admin_events_tab: "Events",
            admin_events_title: "Event Management",
            admin_events_add: "+ Create Event",
            admin_events_title_label: "Event Title",
            admin_events_desc_label: "Description",
            admin_events_date_label: "Date",
            admin_events_passcode_label: "Event Passcode",
            admin_events_no_events: "No events yet",
            admin_events_create_title: "Create New Event",
            admin_event_select_label: "Select Event",
        },
    }
}
