use super::DbPool;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::{Executor, FromRow};

#[derive(Debug, Clone)]
pub struct SwimCustomerImportRow {
    pub store_name: String,
    pub customer_name: String,
    pub mobile: String,
    pub gender: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub id_type: Option<String>,
    pub id_number: Option<String>,
    pub age: Option<i32>,
    pub acquisition_channel: Option<String>,
    pub sales_follow_staff: Option<String>,
    pub coach_follow_staff: Option<String>,
    pub follow_level: Option<String>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub sales_follow_status: Option<String>,
    pub last_sales_follow_date: Option<NaiveDate>,
    pub course_expire_date: Option<NaiveDate>,
    pub remaining_private_sessions: Option<i32>,
    pub coach_follow_status: Option<String>,
    pub last_coach_follow_date: Option<NaiveDate>,
    pub last_checkin_time: Option<NaiveDateTime>,
    pub total_checkins: Option<i32>,
    pub total_spent_amount: Option<f64>,
    pub purchased_card_type: Option<String>,
    pub current_identity: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub tags: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SwimCourseRow {
    pub store_name: String,
    pub customer_name: String,
    pub mobile: String,
    pub gender: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub id_type: Option<String>,
    pub id_number: Option<String>,
    pub age: Option<i32>,
    pub acquisition_channel: Option<String>,
    pub sales_follow_staff: Option<String>,
    pub coach_follow_staff: Option<String>,
    pub follow_level: Option<String>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub sales_follow_status: Option<String>,
    pub last_sales_follow_date: Option<NaiveDate>,
    pub course_expire_date: Option<NaiveDate>,
    pub remaining_private_sessions: Option<i32>,
    pub coach_follow_status: Option<String>,
    pub last_coach_follow_date: Option<NaiveDate>,
    pub last_checkin_time: Option<NaiveDateTime>,
    pub total_checkins: Option<i32>,
    pub total_spent_amount: Option<String>,
    pub purchased_card_type: Option<String>,
    pub current_identity: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub tags: Option<String>,
    pub remark: Option<String>,
}

pub async fn overwrite_all_customers(
    pool: &DbPool,
    rows: &[SwimCustomerImportRow],
) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    tx.execute(sqlx::query("DELETE FROM swim_customer_record"))
        .await?;

    for row in rows {
        tx.execute(
            sqlx::query(
                r#"
                INSERT INTO swim_customer_record (
                    store_name,
                    customer_name,
                    mobile,
                    gender,
                    birth_date,
                    id_type,
                    id_number,
                    age,
                    acquisition_channel,
                    sales_follow_staff,
                    coach_follow_staff,
                    follow_level,
                    membership_start_date,
                    membership_end_date,
                    sales_follow_status,
                    last_sales_follow_date,
                    course_expire_date,
                    remaining_private_sessions,
                    coach_follow_status,
                    last_coach_follow_date,
                    last_checkin_time,
                    total_checkins,
                    total_spent_amount,
                    purchased_card_type,
                    current_identity,
                    created_at,
                    created_by,
                    tags,
                    remark
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&row.store_name)
            .bind(&row.customer_name)
            .bind(&row.mobile)
            .bind(row.gender.as_deref())
            .bind(row.birth_date)
            .bind(row.id_type.as_deref())
            .bind(row.id_number.as_deref())
            .bind(row.age)
            .bind(row.acquisition_channel.as_deref())
            .bind(row.sales_follow_staff.as_deref())
            .bind(row.coach_follow_staff.as_deref())
            .bind(row.follow_level.as_deref())
            .bind(row.membership_start_date)
            .bind(row.membership_end_date)
            .bind(row.sales_follow_status.as_deref())
            .bind(row.last_sales_follow_date)
            .bind(row.course_expire_date)
            .bind(row.remaining_private_sessions)
            .bind(row.coach_follow_status.as_deref())
            .bind(row.last_coach_follow_date)
            .bind(row.last_checkin_time)
            .bind(row.total_checkins)
            .bind(row.total_spent_amount)
            .bind(row.purchased_card_type.as_deref())
            .bind(row.current_identity.as_deref())
            .bind(row.created_at)
            .bind(row.created_by.as_deref())
            .bind(row.tags.as_deref())
            .bind(row.remark.as_deref()),
        )
        .await?;
    }

    tx.commit().await?;
    Ok(rows.len() as u64)
}

pub async fn find_by_phone(pool: &DbPool, phone: &str) -> Result<Vec<SwimCourseRow>, sqlx::Error> {
    sqlx::query_as::<_, SwimCourseRow>(
        r#"
        SELECT
            store_name,
            customer_name,
            mobile,
            gender,
            birth_date,
            id_type,
            id_number,
            age,
            acquisition_channel,
            sales_follow_staff,
            coach_follow_staff,
            follow_level,
            membership_start_date,
            membership_end_date,
            sales_follow_status,
            last_sales_follow_date,
            course_expire_date,
            remaining_private_sessions,
            coach_follow_status,
            last_coach_follow_date,
            last_checkin_time,
            total_checkins,
            CAST(total_spent_amount AS CHAR) AS total_spent_amount,
            purchased_card_type,
            current_identity,
            created_at,
            created_by,
            tags,
            remark
        FROM swim_customer_record
        WHERE mobile = ?
        ORDER BY membership_end_date DESC, membership_start_date DESC
        "#,
    )
    .bind(phone)
    .fetch_all(pool)
    .await
}
