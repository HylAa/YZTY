pub mod auth;
pub mod student_course;
pub mod swim_course;
pub mod venue;
pub mod wechat;

pub use auth::{api_admin_login, api_auth_me, hash_password};
pub use student_course::api_student_courses_by_phone;
pub use swim_course::api_swim_courses_by_phone;
pub use venue::{
    api_admin_update_venue_status, api_get_venue_availability, api_get_venue_overview,
};
pub use wechat::{
    api_wechat_bind_phone, api_wechat_decrypt_phone, api_wechat_jssdk, api_wechat_userinfo,
};
