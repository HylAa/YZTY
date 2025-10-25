use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    db::venue::{self, CourtStatusInput, CourtTimeSlotInput, TimeSlotStatusInput},
    handlers::auth::extract_claims_from_headers,
    ApiResponse, AppState,
};

#[derive(Clone, Copy)]
struct CourtMeta {
    venue_type: &'static str,
    title: &'static str,
    total: u32,
    slot_minutes: i32,
    open_start: i32,
    open_end: i32,
    weekend_full_hour: bool,
    court_labels: &'static [&'static str],
}

const COURT_METAS: [CourtMeta; 3] = [
    CourtMeta {
        venue_type: "badminton",
        title: "羽毛球场地",
        total: 18,
        slot_minutes: 30,
        open_start: 9 * 60,
        open_end: 21 * 60,
        weekend_full_hour: true,
        court_labels: &[],
    },
    CourtMeta {
        venue_type: "basketball",
        title: "篮球场地",
        total: 4,
        slot_minutes: 30,
        open_start: 9 * 60,
        open_end: 21 * 60,
        weekend_full_hour: false,
        court_labels: &["1A", "1B", "2A", "2B"],
    },
    CourtMeta {
        venue_type: "football",
        title: "足球场地",
        total: 2,
        slot_minutes: 30,
        open_start: 8 * 60,
        open_end: 22 * 60 + 30,
        weekend_full_hour: false,
        court_labels: &["南场", "北场"],
    },
];

struct SlotTemplate {
    key: String,
    start_time: String,
    end_time: String,
}

fn minutes_to_label(minutes: i32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    format!("{:02}:{:02}", hours, mins)
}

fn build_slot_key(venue_type: &str, start_time: &str, end_time: &str) -> String {
    format!("{}|{}-{}", venue_type, start_time, end_time)
}

fn parse_slot_key(slot_key: &str) -> (&str, &str) {
    slot_key
        .split_once('|')
        .map(|(venue_type, range)| (venue_type, range))
        .unwrap_or(("ALL", slot_key))
}

fn build_slot_templates(meta: &CourtMeta) -> Vec<SlotTemplate> {
    let mut slots = Vec::new();
    let mut current = meta.open_start;
    while current + meta.slot_minutes <= meta.open_end {
        let end = current + meta.slot_minutes;
        let start_label = minutes_to_label(current);
        let end_label = minutes_to_label(end);
        slots.push(SlotTemplate {
            key: build_slot_key(meta.venue_type, &start_label, &end_label),
            start_time: start_label,
            end_time: end_label,
        });
        current += meta.slot_minutes;
    }
    slots
}

fn build_global_slot_templates() -> Vec<SlotTemplate> {
    let global_start = global_open_start();
    let global_end = global_open_end();
    let slot_minutes = 30;
    let mut slots = Vec::new();
    let mut current = global_start;
    while current + slot_minutes <= global_end {
        let end = current + slot_minutes;
        let start_label = minutes_to_label(current);
        let end_label = minutes_to_label(end);
        slots.push(SlotTemplate {
            key: build_slot_key("ALL", &start_label, &end_label),
            start_time: start_label,
            end_time: end_label,
        });
        current += slot_minutes;
    }
    slots
}

fn global_open_start() -> i32 {
    COURT_METAS
        .iter()
        .map(|meta| meta.open_start)
        .min()
        .unwrap_or(0)
}

fn global_open_end() -> i32 {
    COURT_METAS
        .iter()
        .map(|meta| meta.open_end)
        .max()
        .unwrap_or(24 * 60)
}

fn find_slot_record(
    map: &HashMap<String, venue::TimeSlotStatusRecord>,
    venue_type: &str,
    start_time: &str,
    end_time: &str,
) -> Option<venue::TimeSlotStatusRecord> {
    let mut keys = vec![build_slot_key(venue_type, start_time, end_time)];
    if venue_type != "ALL" {
        keys.push(build_slot_key("ALL", start_time, end_time));
    }
    keys.push(format!("{}-{}", start_time, end_time));
    for key in keys {
        if let Some(record) = map.get(&key) {
            return Some(record.clone());
        }
    }
    None
}

#[derive(Debug, Deserialize)]
pub struct VenueOverviewQuery {
    pub date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VenueOverviewResponse {
    pub date: String,
    pub categories: Vec<VenueCategory>,
    #[serde(rename = "timeSlots")]
    pub time_slots: Vec<TimeSlotStatus>,
    #[serde(rename = "timeSlotsBySport")]
    pub time_slots_by_sport: HashMap<String, Vec<TimeSlotStatus>>,
}

#[derive(Debug, Serialize)]
pub struct VenueCategory {
    #[serde(rename = "type")]
    pub venue_type: String,
    pub title: String,
    pub total: u32,
    pub courts: Vec<CourtStatus>,
}

#[derive(Debug, Serialize)]
pub struct CourtStatus {
    pub number: i32,
    pub label: String,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TimeSlotCourtStatus {
    pub number: i32,
    pub label: String,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TimeSlotStatus {
    #[serde(rename = "type")]
    pub venue_type: String,
    #[serde(rename = "slotKey")]
    pub slot_key: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub courts: Vec<TimeSlotCourtStatus>,
}

pub async fn api_get_venue_overview(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VenueOverviewQuery>,
) -> (StatusCode, Json<ApiResponse<VenueOverviewResponse>>) {
    let target_date = match parse_target_date(query.date.as_deref()) {
        Ok(date) => date,
        Err(resp) => return (StatusCode::BAD_REQUEST, Json(resp)),
    };

    let court_status_map = match venue::fetch_court_status_map(&state.db_pool, target_date).await {
        Ok(map) => map,
        Err(err) => {
            eprintln!("查询场地状态失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "获取场地状态失败")),
            );
        }
    };

    let time_slot_map = match venue::fetch_time_slot_status_map(&state.db_pool, target_date).await {
        Ok(map) => map,
        Err(err) => {
            eprintln!("查询时段状态失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "获取时段状态失败")),
            );
        }
    };

    let court_time_slot_map =
        match venue::fetch_court_time_slot_map(&state.db_pool, target_date).await {
            Ok(map) => map,
            Err(err) => {
                eprintln!("查询场地时段状态失败: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::msg(500, "获取场地时段状态失败")),
                );
            }
        };

    let categories = COURT_METAS
        .iter()
        .map(|meta| build_category(meta, &court_status_map))
        .collect::<Vec<_>>();

    let mut time_slots_by_sport: HashMap<String, Vec<TimeSlotStatus>> = HashMap::new();
    for meta in COURT_METAS.iter() {
        let slots = build_time_slots_for_meta(meta, &time_slot_map, &court_time_slot_map);
        time_slots_by_sport.insert(meta.venue_type.to_string(), slots);
    }

    let global_time_slots = build_global_time_slots(&time_slot_map, &court_time_slot_map);

    let response = VenueOverviewResponse {
        date: target_date.to_string(),
        categories,
        time_slots: global_time_slots,
        time_slots_by_sport,
    };

    (StatusCode::OK, Json(ApiResponse::ok(response)))
}

fn build_category(
    meta: &CourtMeta,
    map: &HashMap<(String, i32), venue::CourtStatusRecord>,
) -> VenueCategory {
    let mut courts = Vec::with_capacity(meta.total as usize);
    for number in 1..=meta.total as i32 {
        let key = (meta.venue_type.to_string(), number);
        let label = if !meta.court_labels.is_empty() {
            meta.court_labels
                .get((number - 1) as usize)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}{}号场", meta.title, number))
        } else {
            format!("{}{}号场", meta.title, number)
        };

        if let Some(record) = map.get(&key) {
            courts.push(CourtStatus {
                number,
                label: label.clone(),
                is_available: record.is_available != 0,
                note: record.note.clone(),
                updated_at: Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            });
        } else {
            courts.push(CourtStatus {
                number,
                label: label.clone(),
                is_available: true,
                note: None,
                updated_at: None,
            });
        }
    }

    VenueCategory {
        venue_type: meta.venue_type.to_string(),
        title: meta.title.to_string(),
        total: meta.total,
        courts,
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateVenueStatusPayload {
    pub date: String,
    pub courts: Option<Vec<UpdateCourtStatus>>,
    #[serde(rename = "timeSlots")]
    pub time_slots: Option<Vec<UpdateTimeSlotStatus>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCourtStatus {
    #[serde(rename = "type")]
    pub venue_type: String,
    pub number: i32,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTimeSlotStatus {
    #[serde(rename = "type")]
    pub venue_type: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
    pub courts: Option<Vec<UpdateCourtTimeSlotStatus>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCourtTimeSlotStatus {
    pub number: i32,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
}

pub async fn api_admin_update_venue_status(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<UpdateVenueStatusPayload>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let claims = match extract_claims_from_headers(&headers) {
        Ok(claims) => claims,
        Err(resp) => return (StatusCode::UNAUTHORIZED, Json(resp)),
    };

    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::msg(403, "无权进行该操作")),
        );
    }

    let target_date = match NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::msg(400, "日期格式需为 YYYY-MM-DD")),
            );
        }
    };

    let mut court_inputs = Vec::new();
    if let Some(courts) = payload.courts.as_ref() {
        for court in courts {
            if !COURT_METAS
                .iter()
                .any(|meta| meta.venue_type == court.venue_type.as_str())
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::msg(400, "未知的场地类型")),
                );
            }
            court_inputs.push(CourtStatusInput {
                venue_type: &court.venue_type,
                court_number: court.number,
                is_available: court.is_available,
                note: court.note.as_deref(),
                updated_by: Some(claims.uid),
            });
        }
    }

    let mut time_slot_inputs: Vec<TimeSlotStatusInput> = Vec::new();
    let mut court_time_slot_inputs: Vec<CourtTimeSlotInput> = Vec::new();
    if let Some(time_slots) = payload.time_slots.as_ref() {
        for slot in time_slots {
            let (slot_minutes, open_start, open_end, meta_option) = if slot.venue_type == "ALL" {
                (30, global_open_start(), global_open_end(), None)
            } else if let Some(meta) = COURT_METAS
                .iter()
                .find(|meta| meta.venue_type == slot.venue_type)
            {
                (
                    meta.slot_minutes,
                    meta.open_start,
                    meta.open_end,
                    Some(meta),
                )
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::msg(400, "未知的场地类型")),
                );
            };

            let start_minutes = match parse_hhmm(&slot.start_time) {
                Some(value) => value,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::msg(400, "开始时间格式不正确")),
                    );
                }
            };
            let end_minutes = match parse_hhmm(&slot.end_time) {
                Some(value) if value > start_minutes => value,
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::msg(400, "结束时间需晚于开始时间")),
                    );
                }
            };

            if start_minutes < open_start || end_minutes > open_end {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::msg(400, "时间段不在营业时间范围内")),
                );
            }

            if (end_minutes - start_minutes) % slot_minutes != 0 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::msg(400, "时间段需为基础单位的倍数")),
                );
            }

            let slot_key = build_slot_key(&slot.venue_type, &slot.start_time, &slot.end_time);

            let court_numbers: Vec<i32> = if let Some(ref courts) = slot.courts {
                if courts.is_empty() {
                    if let Some(meta) = meta_option {
                        (1..=meta.total as i32).collect()
                    } else {
                        COURT_METAS
                            .iter()
                            .filter(|meta| {
                                slot.venue_type == "ALL" || meta.venue_type == slot.venue_type
                            })
                            .flat_map(|meta| (1..=meta.total as i32))
                            .collect()
                    }
                } else {
                    courts.iter().map(|c| c.number).collect()
                }
            } else if let Some(meta) = meta_option {
                (1..=meta.total as i32).collect()
            } else {
                COURT_METAS
                    .iter()
                    .filter(|meta| slot.venue_type == "ALL" || meta.venue_type == slot.venue_type)
                    .flat_map(|meta| (1..=meta.total as i32))
                    .collect()
            };

            time_slot_inputs.push(TimeSlotStatusInput {
                slot_key: slot_key.clone(),
                start_time: slot.start_time.clone(),
                end_time: slot.end_time.clone(),
                is_available: slot.is_available,
                note: slot.note.clone(),
                updated_by: Some(claims.uid),
            });

            for number in court_numbers {
                let override_payload = slot
                    .courts
                    .as_ref()
                    .and_then(|courts| courts.iter().find(|c| c.number == number));
                let court_is_available = override_payload
                    .map(|c| c.is_available)
                    .unwrap_or(slot.is_available);
                let court_note = override_payload
                    .and_then(|c| c.note.clone())
                    .or_else(|| slot.note.clone());

                court_time_slot_inputs.push(CourtTimeSlotInput {
                    venue_type: slot.venue_type.clone(),
                    court_number: number,
                    start_time: slot.start_time.clone(),
                    end_time: slot.end_time.clone(),
                    target_date,
                    is_available: court_is_available,
                    note: court_note,
                    updated_by: Some(claims.uid),
                });
            }
        }
    }

    if let Err(err) = venue::upsert_court_statuses(&state.db_pool, target_date, &court_inputs).await
    {
        eprintln!("更新场地状态失败: {}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::msg(500, "更新场地状态失败")),
        );
    }

    if let Err(err) =
        venue::upsert_time_slot_statuses(&state.db_pool, target_date, &time_slot_inputs).await
    {
        eprintln!("更新时段状态失败: {}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::msg(500, "更新时段状态失败")),
        );
    }

    if !court_time_slot_inputs.is_empty() {
        if let Err(err) =
            venue::upsert_court_time_slots(&state.db_pool, &court_time_slot_inputs).await
        {
            eprintln!("更新场地时段状态失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "更新场地时段状态失败")),
            );
        }
    }

    (StatusCode::OK, Json(ApiResponse::msg(0, "更新成功")))
}

fn parse_target_date(
    date_str: Option<&str>,
) -> Result<NaiveDate, ApiResponse<VenueOverviewResponse>> {
    if let Some(date_str) = date_str {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| ApiResponse::msg(400, "日期格式需为 YYYY-MM-DD"))
    } else {
        Ok(Utc::now().date_naive())
    }
}

fn parse_hhmm(value: &str) -> Option<i32> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hour = parts[0].parse::<i32>().ok()?;
    let minute = parts[1].parse::<i32>().ok()?;
    if hour < 0 || hour >= 24 || minute < 0 || minute >= 60 {
        return None;
    }
    Some(hour * 60 + minute)
}

fn ranges_overlap(start_a: i32, end_a: i32, start_b: i32, end_b: i32) -> bool {
    start_a < end_b && start_b < end_a
}

#[derive(Debug, Deserialize)]
pub struct AvailabilityQuery {
    pub sport: String,
    pub date: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityCourt {
    pub id: String,
    pub label: String,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityResponse {
    pub sport: String,
    pub date: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    pub courts: Vec<AvailabilityCourt>,
}

pub async fn api_get_venue_availability(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AvailabilityQuery>,
) -> (StatusCode, Json<ApiResponse<AvailabilityResponse>>) {
    let sport_meta = match COURT_METAS
        .iter()
        .find(|meta| meta.venue_type == query.sport)
    {
        Some(meta) => meta,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::msg(400, "未知的场地类型")),
            );
        }
    };

    let target_date = match NaiveDate::parse_from_str(&query.date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::msg(400, "日期格式需为 YYYY-MM-DD")),
            );
        }
    };

    let start_minutes = match parse_hhmm(&query.start_time) {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::msg(400, "开始时间格式不正确")),
            );
        }
    };

    let end_minutes = match parse_hhmm(&query.end_time) {
        Some(value) if value > start_minutes => value,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::msg(400, "结束时间需晚于开始时间")),
            );
        }
    };

    // 获取基础数据
    let court_status_map = match venue::fetch_court_status_map(&state.db_pool, target_date).await {
        Ok(map) => map,
        Err(err) => {
            eprintln!("查询场地状态失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "获取场地状态失败")),
            );
        }
    };

    let time_slot_map = match venue::fetch_time_slot_status_map(&state.db_pool, target_date).await {
        Ok(map) => map,
        Err(err) => {
            eprintln!("查询时段状态失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "获取时段状态失败")),
            );
        }
    };

    let court_time_slot_map =
        match venue::fetch_court_time_slot_map(&state.db_pool, target_date).await {
            Ok(map) => map,
            Err(err) => {
                eprintln!("查询场地时段状态失败: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::msg(500, "获取场地时段状态失败")),
                );
            }
        };

    // 预处理时段状态，判断是否存在不可用的时间段
    let mut has_blocked_slot = false;
    let mut blocked_note: Option<String> = None;
    let mut blocked_updated: Option<String> = None;
    for record in time_slot_map.values() {
        if record.is_available != 0 {
            continue;
        }
        let (record_type, _) = parse_slot_key(&record.slot_key);
        if record_type != "ALL" && record_type != sport_meta.venue_type {
            continue;
        }
        if let (Some(start), Some(end)) =
            (parse_hhmm(&record.start_time), parse_hhmm(&record.end_time))
        {
            if ranges_overlap(start_minutes, end_minutes, start, end) {
                has_blocked_slot = true;
                blocked_note = record.note.clone();
                blocked_updated = Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
                break;
            }
        }
    }

    let mut courts = Vec::with_capacity(sport_meta.total as usize);
    for number in 1..=sport_meta.total as i32 {
        let mut available = true;
        let key = (sport_meta.venue_type.to_string(), number);
        let mut note = None;
        let mut updated_at = None;
        if let Some(record) = court_status_map.get(&key) {
            if record.is_available == 0 {
                available = false;
                note = record.note.clone();
                updated_at = Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }

        if let Some(records) = court_time_slot_map.get(&(sport_meta.venue_type.to_string(), number))
        {
            for record in records {
                if record.start_time == query.start_time && record.end_time == query.end_time {
                    available = record.is_available != 0;
                    note = record.note.clone();
                    updated_at = Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
                    break;
                }
            }
        }

        if available && has_blocked_slot {
            available = false;
            let fallback_note = blocked_note
                .clone()
                .unwrap_or_else(|| "该时段不可用".to_string());
            note = Some(note.unwrap_or(fallback_note));
            updated_at = updated_at.or(blocked_updated.clone());
        }

        let label = if !sport_meta.court_labels.is_empty() {
            sport_meta
                .court_labels
                .get((number - 1) as usize)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{} {}号场", sport_meta.title, number))
        } else {
            format!("{} {}号场", sport_meta.title, number)
        };

        courts.push(AvailabilityCourt {
            id: format!("{}-{}", sport_meta.venue_type, number),
            label,
            is_available: available,
            note,
            updated_at,
        });
    }

    let response = AvailabilityResponse {
        sport: sport_meta.venue_type.to_string(),
        date: target_date.to_string(),
        start_time: query.start_time,
        end_time: query.end_time,
        courts,
    };

    (StatusCode::OK, Json(ApiResponse::ok(response)))
}
fn build_time_slots_for_meta(
    meta: &CourtMeta,
    time_slot_map: &HashMap<String, venue::TimeSlotStatusRecord>,
    court_time_slot_map: &HashMap<(String, i32), Vec<venue::CourtTimeSlotRecord>>,
) -> Vec<TimeSlotStatus> {
    let templates = build_slot_templates(meta);
    templates
        .into_iter()
        .map(|template| {
            let record = find_slot_record(
                time_slot_map,
                meta.venue_type,
                &template.start_time,
                &template.end_time,
            );
            let (is_available, note, updated_at) = if let Some(record) = record {
                (
                    record.is_available != 0,
                    record.note.clone(),
                    Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                )
            } else {
                (true, None, None)
            };

            let courts = build_time_slot_courts(meta, court_time_slot_map, &template, is_available);

            TimeSlotStatus {
                venue_type: meta.venue_type.to_string(),
                slot_key: template.key,
                start_time: template.start_time,
                end_time: template.end_time,
                is_available,
                note,
                updated_at,
                courts,
            }
        })
        .collect()
}

fn build_time_slot_courts(
    meta: &CourtMeta,
    court_time_slot_map: &HashMap<(String, i32), Vec<venue::CourtTimeSlotRecord>>,
    template: &SlotTemplate,
    default_available: bool,
) -> Vec<TimeSlotCourtStatus> {
    let mut courts = Vec::with_capacity(meta.total as usize);
    for number in 1..=meta.total as i32 {
        let mut available = default_available;
        let mut note = None;
        let mut updated_at = None;

        if let Some(records) = court_time_slot_map.get(&(meta.venue_type.to_string(), number)) {
            for record in records {
                if record.start_time == template.start_time && record.end_time == template.end_time
                {
                    available = record.is_available != 0;
                    note = record.note.clone();
                    updated_at = Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
                    break;
                }
            }
        }

        let label = if !meta.court_labels.is_empty() {
            meta.court_labels
                .get((number - 1) as usize)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{} {}号场", meta.title, number))
        } else {
            format!("{} {}号场", meta.title, number)
        };

        courts.push(TimeSlotCourtStatus {
            number,
            label,
            is_available: available,
            note,
            updated_at,
        });
    }
    courts
}

fn build_global_time_slots(
    time_slot_map: &HashMap<String, venue::TimeSlotStatusRecord>,
    court_time_slot_map: &HashMap<(String, i32), Vec<venue::CourtTimeSlotRecord>>,
) -> Vec<TimeSlotStatus> {
    let templates = build_global_slot_templates();
    templates
        .into_iter()
        .map(|template| {
            let record = find_slot_record(
                time_slot_map,
                "ALL",
                &template.start_time,
                &template.end_time,
            );
            let (is_available, note, updated_at) = if let Some(record) = record {
                (
                    record.is_available != 0,
                    record.note.clone(),
                    Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                )
            } else {
                (true, None, None)
            };

            let courts = COURT_METAS
                .iter()
                .flat_map(|meta| {
                    build_time_slot_courts(meta, court_time_slot_map, &template, is_available)
                })
                .collect();

            TimeSlotStatus {
                venue_type: "ALL".to_string(),
                slot_key: template.key,
                start_time: template.start_time,
                end_time: template.end_time,
                is_available,
                note,
                updated_at,
                courts,
            }
        })
        .collect()
}
