use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    db::venue::{self, CourtStatusInput, TimeSlotStatusInput},
    handlers::auth::extract_claims_from_headers,
    ApiResponse, AppState,
};

#[derive(Clone, Copy)]
struct CourtMeta {
    venue_type: &'static str,
    title: &'static str,
    total: u32,
}

const COURT_METAS: [CourtMeta; 3] = [
    CourtMeta { venue_type: "badminton", title: "羽毛球场地", total: 18 },
    CourtMeta { venue_type: "basketball", title: "篮球场地", total: 2 },
    CourtMeta { venue_type: "football", title: "足球场地", total: 1 },
];

const TIME_SLOTS: &[(&str, &str, &str)] = &[
    ("08:00-09:00", "08:00", "09:00"),
    ("09:00-10:00", "09:00", "10:00"),
    ("10:00-11:00", "10:00", "11:00"),
    ("11:00-12:00", "11:00", "12:00"),
    ("12:00-13:00", "12:00", "13:00"),
    ("13:00-14:00", "13:00", "14:00"),
    ("14:00-15:00", "14:00", "15:00"),
    ("15:00-16:00", "15:00", "16:00"),
    ("16:00-17:00", "16:00", "17:00"),
    ("17:00-18:00", "17:00", "18:00"),
    ("18:00-19:00", "18:00", "19:00"),
    ("19:00-20:00", "19:00", "20:00"),
    ("20:00-21:00", "20:00", "21:00"),
    ("21:00-22:00", "21:00", "22:00"),
];

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
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TimeSlotStatus {
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

    let categories = COURT_METAS
        .iter()
        .map(|meta| build_category(meta, &court_status_map))
        .collect::<Vec<_>>();

    let time_slots = TIME_SLOTS
        .iter()
        .map(|(label, start, end)| {
            if let Some(record) = time_slot_map.get(*label) {
                TimeSlotStatus {
                    slot_key: label.to_string(),
                    start_time: start.to_string(),
                    end_time: end.to_string(),
                    is_available: record.is_available != 0,
                    note: record.note.clone(),
                    updated_at: Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                }
            } else {
                TimeSlotStatus {
                    slot_key: label.to_string(),
                    start_time: start.to_string(),
                    end_time: end.to_string(),
                    is_available: true,
                    note: None,
                    updated_at: None,
                }
            }
        })
        .collect::<Vec<_>>();

    let response = VenueOverviewResponse {
        date: target_date.to_string(),
        categories,
        time_slots,
    };

    (StatusCode::OK, Json(ApiResponse::ok(response)))
}

fn build_category(meta: &CourtMeta, map: &HashMap<(String, i32), venue::CourtStatusRecord>) -> VenueCategory {
    let mut courts = Vec::with_capacity(meta.total as usize);
    for number in 1..=meta.total as i32 {
        let key = (meta.venue_type.to_string(), number);
        if let Some(record) = map.get(&key) {
            courts.push(CourtStatus {
                number,
                is_available: record.is_available != 0,
                note: record.note.clone(),
                updated_at: Some(record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            });
        } else {
            courts.push(CourtStatus {
                number,
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
    #[serde(rename = "slotKey")]
    pub slot_key: String,
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

    let mut time_slot_inputs = Vec::new();
    if let Some(time_slots) = payload.time_slots.as_ref() {
        for slot in time_slots {
            if let Some((_, start, end)) = TIME_SLOTS.iter().find(|(key, _, _)| key == &slot.slot_key) {
                time_slot_inputs.push(TimeSlotStatusInput {
                    slot_key: &slot.slot_key,
                    start_time: start,
                    end_time: end,
                    is_available: slot.is_available,
                    note: slot.note.as_deref(),
                    updated_by: Some(claims.uid),
                });
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::msg(400, "未知的时段标识")),
                );
            }
        }
    }

    if let Err(err) = venue::upsert_court_statuses(&state.db_pool, target_date, &court_inputs).await {
        eprintln!("更新场地状态失败: {}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::msg(500, "更新场地状态失败")),
        );
    }

    if let Err(err) = venue::upsert_time_slot_statuses(&state.db_pool, target_date, &time_slot_inputs).await {
        eprintln!("更新时段状态失败: {}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::msg(500, "更新时段状态失败")),
        );
    }

    (StatusCode::OK, Json(ApiResponse::msg(0, "更新成功")))
}

fn parse_target_date(date_str: Option<&str>) -> Result<NaiveDate, ApiResponse<VenueOverviewResponse>> {
    if let Some(date_str) = date_str {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| ApiResponse::msg(400, "日期格式需为 YYYY-MM-DD"))
    } else {
        Ok(Utc::now().date_naive())
    }
}
