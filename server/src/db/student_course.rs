use super::DbPool;
use chrono::NaiveDate;
use sqlx::{mysql::MySqlArguments, Arguments, Executor, FromRow};
use std::collections::HashSet;

const STUDENT_EXPECTED_COLUMNS: [&str; 34] = [
    "student_name",
    "phone_owner_type",
    "primary_phone",
    "class_name",
    "course_name",
    "course_type",
    "purchase_quantity",
    "gifted_quantity",
    "consumed_quantity",
    "refund_transfer_quantity",
    "remaining_quantity",
    "over_attend_quantity",
    "consumed_amount",
    "remaining_amount",
    "absence_count",
    "follower",
    "advisor",
    "expire_date",
    "gender",
    "wechat_bind_status",
    "card_bind_status",
    "face_capture_status",
    "alt_phone_owner_type",
    "alt_phone",
    "source",
    "birth_date",
    "age",
    "grade",
    "student_number",
    "school",
    "address",
    "tags",
    "remark",
    "created_by",
];

#[derive(Debug, Clone)]
pub struct StudentCourseImportRow {
    pub student_name: String,
    pub phone_owner_type: Option<String>,
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
    pub absence_count: Option<i32>,
    pub follower: Option<String>,
    pub advisor: Option<String>,
    pub expire_date: Option<NaiveDate>,
    pub gender: Option<String>,
    pub wechat_bind_status: Option<String>,
    pub card_bind_status: Option<String>,
    pub face_capture_status: Option<String>,
    pub alt_phone_owner_type: Option<String>,
    pub alt_phone: Option<String>,
    pub source: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub age: Option<i32>,
    pub grade: Option<String>,
    pub student_number: Option<String>,
    pub school: Option<String>,
    pub address: Option<String>,
    pub tags: Option<String>,
    pub remark: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StudentCourseImportResult {
    pub affected_rows: u64,
    pub missing_columns: Vec<String>,
}

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

pub async fn overwrite_all_courses(
    pool: &DbPool,
    rows: &[StudentCourseImportRow],
) -> Result<StudentCourseImportResult, sqlx::Error> {
    let existing_columns: HashSet<String> = sqlx::query_scalar::<_, String>(
        r#"
        SELECT COLUMN_NAME
        FROM INFORMATION_SCHEMA.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'student_course_record'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .collect();

    let missing_columns: Vec<String> = STUDENT_EXPECTED_COLUMNS
        .iter()
        .filter(|col| !existing_columns.contains(**col))
        .map(|col| col.to_string())
        .collect();

    if !missing_columns.is_empty() {
        eprintln!(
            "[student_course_import] 当前数据库缺少列: {}",
            missing_columns.join(", ")
        );
    }

    let has_column = |name: &str| existing_columns.contains(name);

    let mut tx = pool.begin().await?;

    tx.execute(sqlx::query("DELETE FROM student_course_record"))
        .await?;

    for row in rows {
        let mut columns: Vec<&'static str> = Vec::new();
        let mut args = MySqlArguments::default();

        macro_rules! push_value {
            ($name:expr, $value:expr) => {
                if has_column($name) {
                    columns.push($name);
                    args.add($value);
                }
            };
        }

        push_value!("student_name", row.student_name.clone());
        push_value!("phone_owner_type", row.phone_owner_type.clone());
        push_value!("primary_phone", row.primary_phone.clone());
        push_value!("class_name", row.class_name.clone());
        push_value!("course_name", row.course_name.clone());
        push_value!("course_type", row.course_type.clone());
        push_value!("purchase_quantity", row.purchase_quantity.clone());
        push_value!("gifted_quantity", row.gifted_quantity.clone());
        push_value!("consumed_quantity", row.consumed_quantity.clone());
        push_value!(
            "refund_transfer_quantity",
            row.refund_transfer_quantity.clone()
        );
        push_value!("remaining_quantity", row.remaining_quantity.clone());
        push_value!("over_attend_quantity", row.over_attend_quantity.clone());
        push_value!("consumed_amount", row.consumed_amount);
        push_value!("remaining_amount", row.remaining_amount);
        push_value!("absence_count", row.absence_count);
        push_value!("follower", row.follower.clone());
        push_value!("advisor", row.advisor.clone());
        push_value!("expire_date", row.expire_date);
        push_value!("gender", row.gender.clone());
        push_value!("wechat_bind_status", row.wechat_bind_status.clone());
        push_value!("card_bind_status", row.card_bind_status.clone());
        push_value!("face_capture_status", row.face_capture_status.clone());
        push_value!("alt_phone_owner_type", row.alt_phone_owner_type.clone());
        push_value!("alt_phone", row.alt_phone.clone());
        push_value!("source", row.source.clone());
        push_value!("birth_date", row.birth_date);
        push_value!("age", row.age);
        push_value!("grade", row.grade.clone());
        push_value!("student_number", row.student_number.clone());
        push_value!("school", row.school.clone());
        push_value!("address", row.address.clone());
        push_value!("tags", row.tags.clone());
        push_value!("remark", row.remark.clone());
        push_value!("created_by", row.created_by.clone());

        if columns.is_empty() {
            continue;
        }

        let placeholders = vec!["?"; columns.len()].join(", ");
        let sql = format!(
            "INSERT INTO student_course_record ({}) VALUES ({})",
            columns.join(", "),
            placeholders
        );

        tx.execute(sqlx::query_with(&sql, args)).await?;
    }

    tx.commit().await?;
    Ok(StudentCourseImportResult {
        affected_rows: rows.len() as u64,
        missing_columns,
    })
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
