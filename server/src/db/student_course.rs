use super::DbPool;
use chrono::NaiveDate;
use sqlx::FromRow;

/// 学员课程记录（精简字段，仅用于前端展示）
#[derive(Debug, Clone, FromRow)]
pub struct StudentCourseRow {
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
    pub consumed_amount: Option<String>,
    pub remaining_amount: Option<String>,
    pub expire_date: Option<NaiveDate>,
}

/// 按手机号查询学员课程记录（含备用手机号匹配）
pub async fn find_by_phone(
    pool: &DbPool,
    phone: &str,
) -> Result<Vec<StudentCourseRow>, sqlx::Error> {
    sqlx::query_as::<_, StudentCourseRow>(
        r#"
        SELECT
            student_name,
            primary_phone,
            class_name,
            course_name,
            course_type,
            purchase_quantity,
            gifted_quantity,
            consumed_quantity,
            refund_transfer_quantity,
            remaining_quantity,
            over_attend_quantity,
            CAST(consumed_amount AS CHAR) AS consumed_amount,
            CAST(remaining_amount AS CHAR) AS remaining_amount,
            expire_date
        FROM student_course_record
        WHERE primary_phone = ? OR alt_phone = ?
        ORDER BY
            expire_date IS NULL,
            expire_date ASC,
            course_name ASC
        "#,
    )
    .bind(phone)
    .bind(phone)
    .fetch_all(pool)
    .await
}
