use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Language {
    Thai,
    English,
}

impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::Thai => "ภาษาไทย",
            Language::English => "English",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::Thai => Language::English,
            Language::English => Language::Thai,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Locale {
    // App
    pub app_name: &'static str,
    pub connecting: &'static str,
    pub footer_text: &'static str,

    // Loading
    pub loading_title: &'static str,
    pub loading_subtitle: &'static str,

    // Onboarding
    pub onboard_title: &'static str,
    pub onboard_subtitle: &'static str,
    pub nickname_label: &'static str,
    pub nickname_placeholder: &'static str,
    pub year_label: &'static str,
    pub year_placeholder: &'static str,
    pub student_id_label: &'static str,
    pub student_id_hint: &'static str,
    pub student_id_placeholder: &'static str,
    pub continue_button: &'static str,
    pub creating_profile: &'static str,

    // Errors
    pub err_nickname_required: &'static str,
    pub err_year_required: &'static str,
    pub err_passcode_required: &'static str,
    pub err_passcode_invalid: &'static str,
    pub err_answer_prefix: &'static str,

    // Event
    pub welcome_back: &'static str,
    pub active_badge: &'static str,
    pub rsvp_title: &'static str,
    pub passcode_label: &'static str,
    pub passcode_placeholder: &'static str,
    pub passcode_hint: &'static str,
    pub submit_rsvp: &'static str,
    pub submitting: &'static str,
    pub verified: &'static str,
    pub pending: &'static str,

    // No events
    pub no_events_title: &'static str,
    pub no_events_subtitle: &'static str,

    // Submitted
    pub submitted_title: &'static str,
    pub submitted_subtitle: &'static str,
    pub event_label: &'static str,
    pub registered_as: &'static str,
    pub see_you: &'static str,

    // Form
    pub optional_label: &'static str,
    pub choose_option: &'static str,
}

pub fn get_locale(lang: Language) -> Locale {
    match lang {
        Language::Thai => Locale {
            app_name: "4ever & Beyond",
            connecting: "กำลังเชื่อมต่อ...",
            footer_text: "สร้างด้วย Rust · SpacetimeDB · Dioxus 0.7",

            loading_title: "กำลังเชื่อมต่อกับชุมชน...",
            loading_subtitle: "กำลังยืนยันตัวตนกับ SpacetimeDB",

            onboard_title: "ลงทะเบียนด่วน",
            onboard_subtitle: "ข้อมูลพื้นฐาน — แก้ไขโปรไฟล์ได้ภายหลัง!",
            nickname_label: "ชื่อเล่น",
            nickname_placeholder: "เราควรเรียกคุณว่าอะไร?",
            year_label: "ชั้นปี / ศิษย์เก่า",
            year_placeholder: "เลือกชั้นปี...",
            student_id_label: "รหัสนักศึกษา",
            student_id_hint: "ไม่จำเป็นสำหรับศิษย์เก่า",
            student_id_placeholder: "เช่น 6610xxxxx",
            continue_button: "ดำเนินการต่อ →",
            creating_profile: "กำลังสร้างโปรไฟล์...",

            err_nickname_required: "กรุณาใส่ชื่อเล่น",
            err_year_required: "กรุณาเลือกชั้นปีหรือสถานะศิษย์เก่า",
            err_passcode_required: "กรุณาใส่รหัสผ่านกิจกรรม",
            err_passcode_invalid: "รหัสผ่านไม่ถูกต้อง ตรวจสอบในกลุ่มแชท",
            err_answer_prefix: "กรุณาตอบ: ",

            welcome_back: "ยินดีต้อนรับกลับ,",
            active_badge: "● กำลังดำเนินการ",
            rsvp_title: "คำถามลงทะเบียน",
            passcode_label: "รหัสผ่านกิจกรรม",
            passcode_placeholder: "ใส่รหัสที่แชร์ในกลุ่ม",
            passcode_hint: "🔒 รหัสนี้ป้องกันการลงทะเบียนไม่ได้รับอนุญาต ถามผู้จัดถ้าไม่มีรหัส",
            submit_rsvp: "ยืนยันลงทะเบียน 🎉",
            submitting: "กำลังส่งข้อมูล...",
            verified: "✓ ยืนยันแล้ว",
            pending: "⏳ รอตรวจสอบ",

            no_events_title: "ไม่มีกิจกรรมในขณะนี้",
            no_events_subtitle: "กลับมาตรวจสอบอีกครั้ง — มีอะไรดีๆ กำลังจะมา!",

            submitted_title: "ลงทะเบียนสำเร็จ!",
            submitted_subtitle: "การลงทะเบียนของคุณได้รับการยืนยันแล้ว",
            event_label: "กิจกรรม",
            registered_as: "ลงทะเบียนในชื่อ: ",
            see_you: "แล้วเจอกันใหม่! 🚀",

            optional_label: "(ไม่จำเป็น)",
            choose_option: "เลือกตัวเลือก...",
        },
        Language::English => Locale {
            app_name: "4ever & Beyond",
            connecting: "Connecting...",
            footer_text: "Built with Rust · SpacetimeDB · Dioxus 0.7",

            loading_title: "Connecting to the community...",
            loading_subtitle: "Verifying your identity with SpacetimeDB",

            onboard_title: "Quick Sign-Up",
            onboard_subtitle: "Just the basics — you can complete your profile later!",
            nickname_label: "Nickname",
            nickname_placeholder: "What should we call you?",
            year_label: "Year / Alumni",
            year_placeholder: "Select your year...",
            student_id_label: "Student ID",
            student_id_hint: "Not required for Alumni",
            student_id_placeholder: "e.g. 6610xxxxx",
            continue_button: "Continue →",
            creating_profile: "Creating Profile...",

            err_nickname_required: "Nickname is required.",
            err_year_required: "Please select your Year or Alumni status.",
            err_passcode_required: "Please enter the event passcode.",
            err_passcode_invalid: "Invalid passcode. Check the group chat for the correct code.",
            err_answer_prefix: "Please answer: ",

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

            no_events_title: "No active events right now",
            no_events_subtitle: "Check back soon — something awesome is coming!",

            submitted_title: "You're In!",
            submitted_subtitle: "Your RSVP has been confirmed.",
            event_label: "Event",
            registered_as: "Registered as: ",
            see_you: "See you there! 🚀",

            optional_label: "(optional)",
            choose_option: "Choose an option...",
        },
    }
}
