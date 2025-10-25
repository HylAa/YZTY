use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{db::student_course, ApiResponse, AppState};

#[derive(Debug, Deserialize)]
pub struct StudentCourseQuery {
    pub phone: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CourseItem {
    pub student_name: String,
    pub primary_phone: Option<String>,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    pub course_type: Option<String>,
    pub purchase_quantity: Option<String>,
    pub gifted_quantity: Option<String>,
    pub consumed_quantity: Option<String>,
    pub refund_transfer_quantity: Option<String>,
    pub remaining_quantity: Option<String>,
    pub over_attend_quantity: Option<String>,
    pub consumed_amount: Option<f64>,
    pub remaining_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_date: Option<NaiveDate>,
}

impl From<student_course::StudentCourseRow> for CourseItem {
    fn from(row: student_course::StudentCourseRow) -> Self {
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
            student_name: row.student_name,
            primary_phone: row.primary_phone,
            class_name: row.class_name,
            course_name: row.course_name,
            course_type: row.course_type,
            purchase_quantity: row.purchase_quantity,
            gifted_quantity: row.gifted_quantity,
            consumed_quantity: row.consumed_quantity,
            refund_transfer_quantity: row.refund_transfer_quantity,
            remaining_quantity: row.remaining_quantity,
            over_attend_quantity: row.over_attend_quantity,
            consumed_amount: parse_amount(row.consumed_amount),
            remaining_amount: parse_amount(row.remaining_amount),
            expire_date: row.expire_date,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StudentCourseResponse {
    pub phone: String,
    pub student_names: Vec<String>,
    pub total_courses: usize,
    pub total_consumed_amount: f64,
    pub total_remaining_amount: f64,
    pub records: Vec<CourseItem>,
}

pub async fn api_student_courses_by_phone(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StudentCourseQuery>,
) -> impl axum::response::IntoResponse {
    let phone = query.phone.trim();
    if phone.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<StudentCourseResponse>::msg(
                400,
                "手机号不能为空",
            )),
        );
    }

    match student_course::find_by_phone(&state.db_pool, phone).await {
        Ok(rows) => {
            let records: Vec<CourseItem> = rows.into_iter().map(CourseItem::from).collect();
            let mut student_names: Vec<String> = records
                .iter()
                .map(|item| item.student_name.clone())
                .collect();
            student_names.sort();
            student_names.dedup();

            let total_consumed_amount = records
                .iter()
                .fold(0.0, |acc, item| acc + item.consumed_amount.unwrap_or(0.0));
            let total_remaining_amount = records
                .iter()
                .fold(0.0, |acc, item| acc + item.remaining_amount.unwrap_or(0.0));

            let response = StudentCourseResponse {
                phone: phone.to_string(),
                student_names,
                total_courses: records.len(),
                total_consumed_amount,
                total_remaining_amount,
                records,
            };

            (StatusCode::OK, Json(ApiResponse::ok(response)))
        }
        Err(err) => {
            eprintln!("[student_courses] 数据库查询失败: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<StudentCourseResponse>::msg(
                    500,
                    "查询学员课程失败，请稍后再试",
                )),
            )
        }
    }
}
