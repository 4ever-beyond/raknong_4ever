//!
//! Backend module for 4ever & Beyond — Community Platform
//!
//! PostgreSQL via sqlx + Dioxus 0.7 server functions.
//! Shared types are available on both client and server.
//! All sqlx / DB code lives behind `#[cfg(feature = "server")]`.
//! Server functions use `#[server]` macro which auto-generates
//! HTTP client stubs on the web build.
//!

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use sqlx::Row;

// =============================================================================
// SHARED TYPES — available on client AND server
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    pub id: i32,
    pub session_id: String,
    pub nickname: String,
    pub entry_year: String,
    pub phone: String,
    pub instagram: String,
    pub line_id: String,
    pub is_verified: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventData {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub event_date: String,
    pub priority: i32,
    pub is_active: bool,
    pub passcode: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventQuestion {
    pub id: i32,
    pub event_id: i32,
    pub label: String,
    pub field_type: String,
    pub options: Option<String>,
    pub is_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventResponse {
    pub id: i32,
    pub event_id: i32,
    pub session_id: String,
    pub answers: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventWithQuestions {
    pub event: EventData,
    pub questions: Vec<EventQuestion>,
}

/// A menu item managed by the restaurant admin.
///
/// Stored in the `menu_item` table. For `menu_select` questions the
/// `get_events` server function populates `options` from active rows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MenuItem {
    pub id: i32,
    pub name: String,
    pub price: i64,
    pub is_active: bool,
    pub sort_order: i32,
}

// =============================================================================
// SERVER-ONLY DATABASE MODULE
// =============================================================================
// The #[server] macro completely replaces function bodies on the client,
// so none of the sqlx code below is ever compiled for the web target.

#[cfg(feature = "server")]
mod server_only {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::{Executor, PgPool};
    use std::cell::RefCell;

    // ── Thread-local connection pool ──────────────────────────────────

    thread_local! {
        static POOL: RefCell<Option<PgPool>> = const { RefCell::new(None) };
    }

    /// Convert a sqlx error into a ServerFnError.
    pub(super) fn db_err(e: sqlx::Error) -> ServerFnError {
        ServerFnError::new(format!("DB error: {e}"))
    }

    /// Lazily create (or return the existing) PgPool.
    pub(super) async fn get_pool() -> Result<PgPool, ServerFnError> {
        // Fast path — already initialised
        let exists = POOL.with(|cell| cell.borrow().is_some());
        if exists {
            return POOL.with(|cell| {
                cell.borrow()
                    .clone()
                    .ok_or_else(|| ServerFnError::new("Pool missing"))
            });
        }

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/forever".into());

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to connect to PostgreSQL: {e}")))?;

        POOL.with(|cell| *cell.borrow_mut() = Some(pool.clone()));

        log::info!("✅ PostgreSQL pool created ({database_url})");
        Ok(pool)
    }

    // ── Database initialisation & seed ────────────────────────────────

    pub async fn init_db() -> Result<(), ServerFnError> {
        let pool = get_pool().await?;

        // ── Create tables ─────────────────────────────────────────
        pool.execute(
            r#"
            CREATE TABLE IF NOT EXISTS user_profile (
                id            SERIAL PRIMARY KEY,
                session_id    TEXT UNIQUE NOT NULL,
                nickname      TEXT NOT NULL,
                entry_year    TEXT NOT NULL,
                phone         TEXT NOT NULL,
                instagram     TEXT NOT NULL,
                line_id       TEXT NOT NULL,
                is_verified   BOOLEAN NOT NULL DEFAULT FALSE,
                created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .await
        .map_err(db_err)?;

        pool.execute(
            r#"
            CREATE TABLE IF NOT EXISTS event (
                id            SERIAL PRIMARY KEY,
                title         TEXT NOT NULL,
                description   TEXT NOT NULL DEFAULT '',
                event_date    TEXT NOT NULL DEFAULT '',
                priority      INTEGER NOT NULL DEFAULT 0,
                is_active     BOOLEAN NOT NULL DEFAULT TRUE,
                passcode      TEXT NOT NULL,
                created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .await
        .map_err(db_err)?;

        pool.execute(
            r#"
            CREATE TABLE IF NOT EXISTS event_question (
                id            SERIAL PRIMARY KEY,
                event_id      INTEGER NOT NULL REFERENCES event(id),
                label         TEXT NOT NULL,
                field_type    TEXT NOT NULL DEFAULT 'text',
                options       TEXT,
                is_required   BOOLEAN NOT NULL DEFAULT TRUE
            )
            "#,
        )
        .await
        .map_err(db_err)?;

        pool.execute(
            r#"
            CREATE TABLE IF NOT EXISTS event_response (
                id            SERIAL PRIMARY KEY,
                event_id      INTEGER NOT NULL REFERENCES event(id),
                session_id    TEXT NOT NULL,
                answers       TEXT NOT NULL,
                submitted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .await
        .map_err(db_err)?;

        pool.execute(
            r#"
            CREATE TABLE IF NOT EXISTS menu_item (
                id          SERIAL PRIMARY KEY,
                name        TEXT NOT NULL,
                price       INTEGER NOT NULL DEFAULT 0,
                is_active   BOOLEAN NOT NULL DEFAULT TRUE,
                sort_order  INTEGER NOT NULL DEFAULT 0,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .await
        .map_err(db_err)?;

        log::info!("✅ Tables verified / created");

        // ── Seed data (only if empty) ─────────────────────────────
        let event_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM event")
            .fetch_one(&pool)
            .await
            .map_err(db_err)?;

        if event_count == 0 {
            sqlx::query(
                r#"
                INSERT INTO event (title, description, event_date, priority, passcode)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind("4EVER รวมตัวกินสเต็กเด็กอ้วน")
            .bind("รวมตัวกินสเต็กเด็กอ้วน ศาลายา ซอย 11\n\n🚗 มีที่จอดรถ ร้านอยู่ท้ายซอย\n📍 https://linktr.ee/steakdekuanwattana\n\nข้อมูลปลอดภัยแน่นอนจ้ะ พี่โอโซนรับประกัน")
            .bind("08-04-2569")
            .bind(10i32)
            .bind("4ever2026")
            .execute(&pool)
            .await
            .map_err(db_err)?;

            // Menu question — uses custom `menu_select` field type
            // Options are populated dynamically from the `menu_item` table
            sqlx::query(
                r#"
                INSERT INTO event_question (event_id, label, field_type, options, is_required)
                VALUES (
                    (SELECT id FROM event WHERE title = '4EVER รวมตัวกินสเต็กเด็กอ้วน'),
                    $1, $2, $3, $4
                )
                "#,
            )
            .bind("เมนูที่จะกินค่าาา")
            .bind("menu_select")
            .bind("[]")
            .bind(true)
            .execute(&pool)
            .await
            .map_err(db_err)?;

            // Seed default menu items
            let seed_menu = vec![
                ("สเต็กหมู ขนาด S", 109i32, 1),
                ("สเต็กหมู ขนาด M", 139, 2),
                ("สเต็กหมู ขนาด L", 169, 3),
                ("สเต็กไก่ ขนาด S", 99, 4),
                ("สเต็กไก่ ขนาด M", 129, 5),
                ("สเต็กไก่ ขนาด L", 159, 6),
                ("สเต็กปลาแซลมอน", 229, 7),
                ("เมนูอื่นๆ (ระบุในช่องอื่น)", 0, 8),
            ];
            for (name, price, sort) in &seed_menu {
                sqlx::query("INSERT INTO menu_item (name, price, sort_order) VALUES ($1, $2, $3)")
                    .bind(name)
                    .bind(price)
                    .bind(*sort)
                    .execute(&pool)
                    .await
                    .map_err(db_err)?;
            }

            log::info!(
                "✅ Seed data inserted (default event + menu question + {} menu items)",
                seed_menu.len()
            );
        } else {
            log::info!("Seed data skipped — {event_count} event(s) already exist");

            // ── Migrate existing data ──────────────────────────────
            // Remove the old "เห็นข่าว" question if it still exists
            let removed = sqlx::query(
                "DELETE FROM event_question WHERE label = 'เห็นข่าวการเรียกรวมตัวจากที่ไหนเอ่ย'",
            )
            .execute(&pool)
            .await
            .map_err(db_err)?;
            if removed.rows_affected() > 0 {
                log::info!("✅ Removed old 'เห็นข่าว' question");
            }

            // Migrate old menu question to menu_select (options now come from menu_item table)
            let migrated = sqlx::query(
                r#"
                UPDATE event_question
                SET field_type = 'menu_select', options = '[]'
                WHERE label = 'เมนูที่จะกินค่าาา' AND field_type = 'select'
                "#,
            )
            .execute(&pool)
            .await
            .map_err(db_err)?;
            if migrated.rows_affected() > 0 {
                log::info!("✅ Migrated menu question to menu_select (from menu_item table)");
            }

            // Seed menu items if the table is empty
            let menu_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM menu_item")
                .fetch_one(&pool)
                .await
                .map_err(db_err)?;
            if menu_count == 0 {
                let seed_menu = vec![
                    ("สเต็กหมู ขนาด S", 109i32, 1),
                    ("สเต็กหมู ขนาด M", 139, 2),
                    ("สเต็กหมู ขนาด L", 169, 3),
                    ("สเต็กไก่ ขนาด S", 99, 4),
                    ("สเต็กไก่ ขนาด M", 129, 5),
                    ("สเต็กไก่ ขนาด L", 159, 6),
                    ("สเต็กปลาแซลมอน", 229, 7),
                    ("เมนูอื่นๆ (ระบุในช่องอื่น)", 0, 8),
                ];
                for (name, price, sort) in &seed_menu {
                    sqlx::query(
                        "INSERT INTO menu_item (name, price, sort_order) VALUES ($1, $2, $3)",
                    )
                    .bind(name)
                    .bind(price)
                    .bind(*sort)
                    .execute(&pool)
                    .await
                    .map_err(db_err)?;
                }
                log::info!("✅ Seeded {} default menu items", seed_menu.len());
            }
        }

        Ok(())
    }
}

/// Re-export `init_db` so that the server entry-point in `main.rs` can call it.
#[cfg(feature = "server")]
pub use server_only::init_db;

// =============================================================================
// SERVER FUNCTIONS
// =============================================================================
// Each function has a full body that is compiled on the server.
// On the client the `#[server]` macro replaces the body with an HTTP POST
// stub, so sqlx / server_only references are never seen by the client
// compiler.
// =============================================================================

#[server(endpoint = "register_profile")]
pub async fn register_profile(
    session_id: String,
    nickname: String,
    entry_year: String,
    phone: String,
    instagram: String,
    line_id: String,
) -> Result<String, ServerFnError> {
    // ── Validate ──────────────────────────────────────────────────────
    if session_id.trim().is_empty()
        || nickname.trim().is_empty()
        || entry_year.trim().is_empty()
        || phone.trim().is_empty()
        || instagram.trim().is_empty()
        || line_id.trim().is_empty()
    {
        return Err(ServerFnError::new("All fields are required"));
    }

    let pool = server_only::get_pool().await?;

    // ── Duplicate check ───────────────────────────────────────────────
    let existing = sqlx::query("SELECT id FROM user_profile WHERE session_id = $1")
        .bind(&session_id)
        .fetch_optional(&pool)
        .await
        .map_err(server_only::db_err)?;

    if existing.is_some() {
        return Err(ServerFnError::new(
            "A profile with this session already exists",
        ));
    }

    // ── Insert ────────────────────────────────────────────────────────
    sqlx::query(
        r#"
        INSERT INTO user_profile (session_id, nickname, entry_year, phone, instagram, line_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(session_id.trim())
    .bind(nickname.trim())
    .bind(entry_year.trim())
    .bind(phone.trim())
    .bind(instagram.trim())
    .bind(line_id.trim())
    .execute(&pool)
    .await
    .map_err(server_only::db_err)?;

    log::info!(
        "[register_profile] Profile created — nickname: '{}'",
        nickname.trim()
    );
    Ok(session_id)
}

#[server(endpoint = "submit_response")]
pub async fn submit_response(
    event_id: i64,
    passcode: String,
    session_id: String,
    answers: String,
) -> Result<(), ServerFnError> {
    let pool = server_only::get_pool().await?;

    // ── Validate user profile ─────────────────────────────────────────
    let user_row = sqlx::query("SELECT id FROM user_profile WHERE session_id = $1")
        .bind(&session_id)
        .fetch_optional(&pool)
        .await
        .map_err(server_only::db_err)?;

    if user_row.is_none() {
        return Err(ServerFnError::new(
            "User profile not found. Register first.",
        ));
    }

    // ── Validate event ────────────────────────────────────────────────
    let event_row = sqlx::query("SELECT id, is_active, passcode FROM event WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&pool)
        .await
        .map_err(server_only::db_err)?;

    let event_row = event_row.ok_or_else(|| ServerFnError::new("Event not found"))?;

    let is_active: bool = event_row
        .try_get("is_active")
        .map_err(server_only::db_err)?;
    let stored_passcode: String = event_row.try_get("passcode").map_err(server_only::db_err)?;

    if !is_active {
        return Err(ServerFnError::new("Event is no longer active"));
    }

    // ── Passcode gate ─────────────────────────────────────────────────
    if passcode.trim() != stored_passcode.trim() {
        return Err(ServerFnError::new("Invalid passcode"));
    }

    // ── Duplicate RSVP check ──────────────────────────────────────────
    let existing =
        sqlx::query("SELECT id FROM event_response WHERE event_id = $1 AND session_id = $2")
            .bind(event_id)
            .bind(&session_id)
            .fetch_optional(&pool)
            .await
            .map_err(server_only::db_err)?;

    if existing.is_some() {
        return Err(ServerFnError::new("You have already RSVP'd to this event"));
    }

    // ── Validate answers ──────────────────────────────────────────────
    if answers.trim().is_empty() {
        return Err(ServerFnError::new("Answers cannot be empty"));
    }

    // ── Persist ───────────────────────────────────────────────────────
    sqlx::query(
        r#"
        INSERT INTO event_response (event_id, session_id, answers)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(event_id)
    .bind(&session_id)
    .bind(answers.trim())
    .execute(&pool)
    .await
    .map_err(server_only::db_err)?;

    log::info!(
        "[submit_response] RSVP confirmed — session: {:?}, event: {}",
        session_id,
        event_id
    );
    Ok(())
}

#[server(endpoint = "get_events")]
pub async fn get_events() -> Result<Vec<EventWithQuestions>, ServerFnError> {
    let pool = server_only::get_pool().await?;

    let event_rows = sqlx::query(
        "SELECT id, title, description, event_date, priority, is_active, passcode, created_at \
         FROM event \
         WHERE is_active = true \
         ORDER BY priority DESC, created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(server_only::db_err)?;

    let mut result = Vec::new();

    for row in event_rows {
        let event_id: i32 = row.try_get("id").map_err(server_only::db_err)?;
        let created_at: chrono::DateTime<chrono::Utc> =
            row.try_get("created_at").map_err(server_only::db_err)?;

        let event_data = EventData {
            id: event_id,
            title: row.try_get("title").map_err(server_only::db_err)?,
            description: row.try_get("description").map_err(server_only::db_err)?,
            event_date: row.try_get("event_date").map_err(server_only::db_err)?,
            priority: row.try_get("priority").map_err(server_only::db_err)?,
            is_active: row.try_get("is_active").map_err(server_only::db_err)?,
            passcode: row.try_get("passcode").map_err(server_only::db_err)?,
            created_at: created_at.to_rfc3339(),
        };

        // Fetch questions for this event
        let q_rows = sqlx::query(
            "SELECT id, event_id, label, field_type, options, is_required \
             FROM event_question \
             WHERE event_id = $1 \
             ORDER BY id",
        )
        .bind(event_id)
        .fetch_all(&pool)
        .await
        .map_err(server_only::db_err)?;

        // Pre-fetch active menu items once (used by menu_select questions)
        let menu_rows = sqlx::query(
            "SELECT name, price FROM menu_item WHERE is_active = true ORDER BY sort_order, id",
        )
        .fetch_all(&pool)
        .await
        .map_err(server_only::db_err)?;

        let menu_options_json: String = {
            let items: Vec<serde_json::Value> = menu_rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.try_get::<String, _>("name").unwrap_or_default(),
                        "price": r.try_get::<i32, _>("price").unwrap_or(0),
                    })
                })
                .collect();
            serde_json::to_string(&items).unwrap_or_default()
        };

        let questions: Vec<EventQuestion> = q_rows
            .into_iter()
            .map(|qr| -> Result<EventQuestion, ServerFnError> {
                let field_type: String = qr.try_get("field_type").map_err(server_only::db_err)?;
                let mut options: Option<String> = qr.try_get("options").ok();

                // For menu_select questions, populate options from the menu_item table
                if field_type == "menu_select" {
                    options = Some(menu_options_json.clone());
                }

                Ok(EventQuestion {
                    id: qr.try_get("id").map_err(server_only::db_err)?,
                    event_id: qr.try_get("event_id").map_err(server_only::db_err)?,
                    label: qr.try_get("label").map_err(server_only::db_err)?,
                    field_type,
                    options,
                    is_required: qr.try_get("is_required").map_err(server_only::db_err)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        result.push(EventWithQuestions {
            event: event_data,
            questions,
        });
    }

    Ok(result)
}

#[server(endpoint = "get_user_profile")]
pub async fn get_user_profile(session_id: String) -> Result<Option<UserProfile>, ServerFnError> {
    let pool = server_only::get_pool().await?;

    let row = sqlx::query(
        "SELECT id, session_id, nickname, entry_year, phone, instagram, line_id, \
                is_verified, created_at \
         FROM user_profile \
         WHERE session_id = $1",
    )
    .bind(&session_id)
    .fetch_optional(&pool)
    .await
    .map_err(server_only::db_err)?;

    match row {
        Some(r) => {
            let created_at: chrono::DateTime<chrono::Utc> =
                r.try_get("created_at").map_err(server_only::db_err)?;
            Ok(Some(UserProfile {
                id: r.try_get("id").map_err(server_only::db_err)?,
                session_id: r.try_get("session_id").map_err(server_only::db_err)?,
                nickname: r.try_get("nickname").map_err(server_only::db_err)?,
                entry_year: r.try_get("entry_year").map_err(server_only::db_err)?,
                phone: r.try_get("phone").map_err(server_only::db_err)?,
                instagram: r.try_get("instagram").map_err(server_only::db_err)?,
                line_id: r.try_get("line_id").map_err(server_only::db_err)?,
                is_verified: r.try_get("is_verified").map_err(server_only::db_err)?,
                created_at: created_at.to_rfc3339(),
            }))
        }
        None => Ok(None),
    }
}

#[server(endpoint = "get_all_users")]
pub async fn get_all_users() -> Result<Vec<UserProfile>, ServerFnError> {
    let pool = server_only::get_pool().await?;

    let rows = sqlx::query(
        "SELECT id, session_id, nickname, entry_year, phone, instagram, line_id, \
                is_verified, created_at \
         FROM user_profile \
         ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(server_only::db_err)?;

    rows.into_iter()
        .map(|r| {
            let created_at: chrono::DateTime<chrono::Utc> =
                r.try_get("created_at").map_err(server_only::db_err)?;
            Ok(UserProfile {
                id: r.try_get("id").map_err(server_only::db_err)?,
                session_id: r.try_get("session_id").map_err(server_only::db_err)?,
                nickname: r.try_get("nickname").map_err(server_only::db_err)?,
                entry_year: r.try_get("entry_year").map_err(server_only::db_err)?,
                phone: r.try_get("phone").map_err(server_only::db_err)?,
                instagram: r.try_get("instagram").map_err(server_only::db_err)?,
                line_id: r.try_get("line_id").map_err(server_only::db_err)?,
                is_verified: r.try_get("is_verified").map_err(server_only::db_err)?,
                created_at: created_at.to_rfc3339(),
            })
        })
        .collect()
}

#[server(endpoint = "get_all_responses")]
pub async fn get_all_responses() -> Result<Vec<EventResponse>, ServerFnError> {
    let pool = server_only::get_pool().await?;

    let rows = sqlx::query(
        "SELECT id, event_id, session_id, answers, submitted_at \
         FROM event_response \
         ORDER BY submitted_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(server_only::db_err)?;

    rows.into_iter()
        .map(|r| {
            let submitted_at: chrono::DateTime<chrono::Utc> =
                r.try_get("submitted_at").map_err(server_only::db_err)?;
            Ok(EventResponse {
                id: r.try_get("id").map_err(server_only::db_err)?,
                event_id: r.try_get("event_id").map_err(server_only::db_err)?,
                session_id: r.try_get("session_id").map_err(server_only::db_err)?,
                answers: r.try_get("answers").map_err(server_only::db_err)?,
                submitted_at: submitted_at.to_rfc3339(),
            })
        })
        .collect()
}

#[server(endpoint = "toggle_verification")]
pub async fn toggle_verification(session_id: String) -> Result<(), ServerFnError> {
    let pool = server_only::get_pool().await?;

    let row = sqlx::query("SELECT is_verified FROM user_profile WHERE session_id = $1")
        .bind(&session_id)
        .fetch_optional(&pool)
        .await
        .map_err(server_only::db_err)?;

    let row = row.ok_or_else(|| ServerFnError::new("User not found"))?;

    let current: bool = row.try_get("is_verified").map_err(server_only::db_err)?;

    sqlx::query("UPDATE user_profile SET is_verified = $1 WHERE session_id = $2")
        .bind(!current)
        .bind(&session_id)
        .execute(&pool)
        .await
        .map_err(server_only::db_err)?;

    log::info!(
        "[toggle_verification] {session_id}: {} → {}",
        current,
        !current
    );
    Ok(())
}

#[server(endpoint = "delete_event_response")]
pub async fn delete_event_response(response_id: i64) -> Result<(), ServerFnError> {
    let pool = server_only::get_pool().await?;

    sqlx::query("DELETE FROM event_response WHERE id = $1")
        .bind(response_id)
        .execute(&pool)
        .await
        .map_err(server_only::db_err)?;

    log::info!("[delete_event_response] id={response_id} deleted");
    Ok(())
}

#[server(endpoint = "get_menu_items")]
pub async fn get_menu_items() -> Result<Vec<MenuItem>, ServerFnError> {
    let pool = server_only::get_pool().await?;

    let rows = sqlx::query(
        "SELECT id, name, price, is_active, sort_order \
         FROM menu_item \
         ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await
    .map_err(server_only::db_err)?;

    Ok(rows
        .into_iter()
        .map(|r| MenuItem {
            id: r.try_get("id").unwrap_or(0),
            name: r.try_get("name").unwrap_or_default(),
            price: r.try_get("price").unwrap_or(0),
            is_active: r.try_get("is_active").unwrap_or(true),
            sort_order: r.try_get("sort_order").unwrap_or(0),
        })
        .collect())
}

#[server(endpoint = "add_menu_item")]
pub async fn add_menu_item(name: String, price: i64) -> Result<(), ServerFnError> {
    let pool = server_only::get_pool().await?;

    let max_sort: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(sort_order), 0) FROM menu_item")
        .fetch_one(&pool)
        .await
        .map_err(server_only::db_err)?;

    sqlx::query("INSERT INTO menu_item (name, price, sort_order) VALUES ($1, $2, $3)")
        .bind(&name)
        .bind(price as i32)
        .bind(max_sort + 1)
        .execute(&pool)
        .await
        .map_err(server_only::db_err)?;

    log::info!("[add_menu_item] Added: {name} (฿{price})");
    Ok(())
}

#[server(endpoint = "update_menu_item")]
pub async fn update_menu_item(
    id: i64,
    name: String,
    price: i64,
    is_active: bool,
) -> Result<(), ServerFnError> {
    let pool = server_only::get_pool().await?;

    sqlx::query("UPDATE menu_item SET name = $1, price = $2, is_active = $3 WHERE id = $4")
        .bind(&name)
        .bind(price as i32)
        .bind(is_active)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(server_only::db_err)?;

    log::info!("[update_menu_item] Updated id={id}: {name} (฿{price}) active={is_active}");
    Ok(())
}

#[server(endpoint = "delete_menu_item")]
pub async fn delete_menu_item(id: i64) -> Result<(), ServerFnError> {
    let pool = server_only::get_pool().await?;

    sqlx::query("DELETE FROM menu_item WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(server_only::db_err)?;

    log::info!("[delete_menu_item] Deleted id={id}");
    Ok(())
}

#[server(endpoint = "check_existing_response")]
pub async fn check_existing_response(
    event_id: i64,
    session_id: String,
) -> Result<bool, ServerFnError> {
    let pool = server_only::get_pool().await?;

    let row = sqlx::query("SELECT id FROM event_response WHERE event_id = $1 AND session_id = $2")
        .bind(event_id)
        .bind(&session_id)
        .fetch_optional(&pool)
        .await
        .map_err(server_only::db_err)?;

    Ok(row.is_some())
}

/// Verify the admin dashboard passcode.
///
/// Reads the `ADMIN_PASSCODE` environment variable (falls back to
/// `"4ever-admin-2026"` if unset) and compares it with the supplied value.
/// Returns `Ok(true)` on match, `Ok(false)` on mismatch.
#[server(endpoint = "verify_admin_passcode")]
pub async fn verify_admin_passcode(passcode: String) -> Result<bool, ServerFnError> {
    let expected =
        std::env::var("ADMIN_PASSCODE").unwrap_or_else(|_| "4ever-admin-2026".to_string());
    Ok(passcode.trim() == expected.trim())
}
