use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::{db::swim_course, ApiResponse, AppState};

#[derive(Debug, Deserialize)]
pub struct SwimCourseQuery {
    pub phone: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SwimCourseItem {
    pub store_name: String,
    pub customer_name: String,
    pub mobile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_follow_staff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coach_follow_staff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub membership_start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub membership_end_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_follow_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_expire_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_private_sessions: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coach_follow_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checkin_time: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_checkins: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_spent_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchased_card_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

impl From<swim_course::SwimCourseRow> for SwimCourseItem {
    fn from(row: swim_course::SwimCourseRow) -> Self {
        fn parse_amount(raw: Option<String>) -> Option<f64> {
            raw.and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return None;
                }
                trimmed.parse::<f64>().ok()
            })
        }

        Self {
            store_name: row.store_name,
            customer_name: row.customer_name,
            mobile: row.mobile,
            gender: row.gender,
            birth_date: row.birth_date,
            age: row.age,
            acquisition_channel: row.acquisition_channel,
            sales_follow_staff: row.sales_follow_staff,
            coach_follow_staff: row.coach_follow_staff,
            follow_level: row.follow_level,
            membership_start_date: row.membership_start_date,
            membership_end_date: row.membership_end_date,
            sales_follow_status: row.sales_follow_status,
            course_expire_date: row.course_expire_date,
            remaining_private_sessions: row.remaining_private_sessions,
            coach_follow_status: row.coach_follow_status,
            last_checkin_time: row.last_checkin_time,
            total_checkins: row.total_checkins,
            total_spent_amount: parse_amount(row.total_spent_amount),
            purchased_card_type: row.purchased_card_type,
            current_identity: row.current_identity,
            created_at: row.created_at,
            created_by: row.created_by,
            tags: row.tags,
            remark: row.remark,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SwimCourseResponse {
    pub phone: String,
    pub total_records: usize,
    pub total_remaining_private_sessions: i32,
    pub total_spent_amount: f64,
    pub total_checkins: i32,
    pub records: Vec<SwimCourseItem>,
}

pub async fn api_swim_courses_by_phone(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SwimCourseQuery>,
) -> impl axum::response::IntoResponse {
    let phone = query.phone.trim();
    if phone.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<SwimCourseResponse>::msg(
                400,
                "手机号不能为空",
            )),
        );
    }

    match swim_course::find_by_phone(&state.db_pool, phone).await {
        Ok(rows) => {
            let records: Vec<SwimCourseItem> = rows.into_iter().map(SwimCourseItem::from).collect();
            let total_records = records.len();
            let total_remaining_private_sessions = records.iter().fold(0, |acc, item| {
                acc + item.remaining_private_sessions.unwrap_or(0)
            });
            let total_spent_amount = records.iter().fold(0.0, |acc, item| {
                acc + item.total_spent_amount.unwrap_or(0.0)
            });
            let total_checkins = records
                .iter()
                .fold(0, |acc, item| acc + item.total_checkins.unwrap_or(0));

            let response = SwimCourseResponse {
                phone: phone.to_string(),
                total_records,
                total_remaining_private_sessions,
                total_spent_amount,
                total_checkins,
                records,
            };

            (StatusCode::OK, Json(ApiResponse::ok(response)))
        }
        Err(err) => {
            eprintln!("[swim_courses] 数据库查询失败: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SwimCourseResponse>::msg(
                    500,
                    "查询游泳课程失败，请稍后再试",
                )),
            )
        }
    }
}
