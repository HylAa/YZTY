use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime};
use sqlx::FromRow;

use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct CourtStatusRecord {
    pub id: u64,
    pub venue_type: String,
    pub court_number: i32,
    pub target_date: NaiveDate,
    pub is_available: i8,
    pub note: Option<String>,
    pub updated_by: Option<u64>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CourtStatusInput<'a> {
    pub venue_type: &'a str,
    pub court_number: i32,
    pub is_available: bool,
    pub note: Option<&'a str>,
    pub updated_by: Option<u64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TimeSlotStatusRecord {
    pub id: u64,
    pub slot_key: String,
    pub start_time: String,
    pub end_time: String,
    pub target_date: NaiveDate,
    pub is_available: i8,
    pub note: Option<String>,
    pub updated_by: Option<u64>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct TimeSlotStatusInput<'a> {
    pub slot_key: &'a str,
    pub start_time: &'a str,
    pub end_time: &'a str,
    pub is_available: bool,
    pub note: Option<&'a str>,
    pub updated_by: Option<u64>,
}

pub async fn ensure_venue_tables(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS venue_court_statuses (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            venue_type VARCHAR(32) NOT NULL,
            court_number INT NOT NULL,
            target_date DATE NOT NULL,
            is_available TINYINT(1) NOT NULL DEFAULT 1,
            note VARCHAR(255) NULL,
            updated_by BIGINT UNSIGNED NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE KEY uniq_court_date (venue_type, court_number, target_date)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS venue_time_slot_statuses (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            slot_key VARCHAR(32) NOT NULL,
            start_time CHAR(5) NOT NULL,
            end_time CHAR(5) NOT NULL,
            target_date DATE NOT NULL,
            is_available TINYINT(1) NOT NULL DEFAULT 1,
            note VARCHAR(255) NULL,
            updated_by BIGINT UNSIGNED NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE KEY uniq_slot_date (slot_key, target_date)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn fetch_court_status_map(
    pool: &DbPool,
    target_date: NaiveDate,
) -> Result<HashMap<(String, i32), CourtStatusRecord>, sqlx::Error> {
    let records = sqlx::query_as::<_, CourtStatusRecord>(
        "SELECT id, venue_type, court_number, target_date, is_available, note, updated_by, updated_at
         FROM venue_court_statuses WHERE target_date = ?",
    )
    .bind(target_date)
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|record| ((record.venue_type.clone(), record.court_number), record))
        .collect())
}

pub async fn fetch_time_slot_status_map(
    pool: &DbPool,
    target_date: NaiveDate,
) -> Result<HashMap<String, TimeSlotStatusRecord>, sqlx::Error> {
    let records = sqlx::query_as::<_, TimeSlotStatusRecord>(
        "SELECT id, slot_key, start_time, end_time, target_date, is_available, note, updated_by, updated_at
         FROM venue_time_slot_statuses WHERE target_date = ?",
    )
    .bind(target_date)
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|record| (record.slot_key.clone(), record))
        .collect())
}

pub async fn upsert_court_statuses(
    pool: &DbPool,
    target_date: NaiveDate,
    payloads: &[CourtStatusInput<'_>],
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();
    for payload in payloads {
        sqlx::query(
            "INSERT INTO venue_court_statuses (venue_type, court_number, target_date, is_available, note, updated_by, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE is_available = VALUES(is_available), note = VALUES(note), updated_by = VALUES(updated_by), updated_at = VALUES(updated_at)",
        )
        .bind(payload.venue_type)
        .bind(payload.court_number)
        .bind(target_date)
        .bind(if payload.is_available { 1 } else { 0 })
        .bind(payload.note)
        .bind(payload.updated_by)
        .bind(now)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn upsert_time_slot_statuses(
    pool: &DbPool,
    target_date: NaiveDate,
    payloads: &[TimeSlotStatusInput<'_>],
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();
    for payload in payloads {
        sqlx::query(
            "INSERT INTO venue_time_slot_statuses (slot_key, start_time, end_time, target_date, is_available, note, updated_by, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE is_available = VALUES(is_available), note = VALUES(note), updated_by = VALUES(updated_by), updated_at = VALUES(updated_at)",
        )
        .bind(payload.slot_key)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(target_date)
        .bind(if payload.is_available { 1 } else { 0 })
        .bind(payload.note)
        .bind(payload.updated_by)
        .bind(now)
        .execute(pool)
        .await?;
    }
    Ok(())
}
