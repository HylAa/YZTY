use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::sync::Arc;

use axum::extract::Multipart;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use calamine::{self, DataType, Range, Reader, Sheets};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader as XmlReader;
use serde::Serialize;
use zip::read::ZipArchive;

use crate::db::{
    student_course::{self, StudentCourseImportRow},
    swim_course::{self, SwimCustomerImportRow},
};
use crate::handlers::auth::extract_claims_from_headers;
use crate::{ApiResponse, AppState};

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub total_rows: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_columns: Vec<String>,
}

enum ImportError {
    Unauthorized(i32, String),
    MissingFile,
    InvalidExcel(String),
    Parse(String),
    Multipart(String),
    Workbook(String),
    Database(sqlx::Error),
}

impl From<sqlx::Error> for ImportError {
    fn from(value: sqlx::Error) -> Self {
        ImportError::Database(value)
    }
}

impl ImportError {
    fn status_code(&self) -> StatusCode {
        match self {
            ImportError::Unauthorized(_, _) => StatusCode::UNAUTHORIZED,
            ImportError::MissingFile => StatusCode::BAD_REQUEST,
            ImportError::InvalidExcel(_)
            | ImportError::Parse(_)
            | ImportError::Multipart(_)
            | ImportError::Workbook(_) => StatusCode::BAD_REQUEST,
            ImportError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn api_code(&self) -> i32 {
        match self {
            ImportError::Unauthorized(code, _) => *code,
            ImportError::MissingFile => 400,
            ImportError::InvalidExcel(_) => 400,
            ImportError::Parse(_) => 400,
            ImportError::Multipart(_) => 400,
            ImportError::Workbook(_) => 400,
            ImportError::Database(_) => 500,
        }
    }

    fn message(&self) -> String {
        match self {
            ImportError::Unauthorized(_, msg) => msg.clone(),
            ImportError::MissingFile => "未检测到上传文件，请选择 Excel 文件后重试".into(),
            ImportError::InvalidExcel(msg) => msg.clone(),
            ImportError::Parse(msg) => msg.clone(),
            ImportError::Multipart(msg) => msg.clone(),
            ImportError::Workbook(msg) => msg.clone(),
            ImportError::Database(err) => {
                format!("导入数据写入数据库失败: {}", err)
            }
        }
    }
}

pub async fn api_import_student_course(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<ImportSummary>>) {
    if let Err(resp) = extract_claims_from_headers(&headers) {
        let err = ImportError::Unauthorized(resp.code, resp.message);
        return error_response(err);
    }

    match handle_student_import(&state, &mut multipart).await {
        Ok((total_rows, missing_columns)) => (
            StatusCode::OK,
            Json(ApiResponse::ok(ImportSummary {
                total_rows,
                missing_columns,
            })),
        ),
        Err(err) => emit_error(err),
    }
}

pub async fn api_import_swim_customer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<ImportSummary>>) {
    if let Err(resp) = extract_claims_from_headers(&headers) {
        let err = ImportError::Unauthorized(resp.code, resp.message);
        return error_response(err);
    }

    match handle_swim_import(&state, &mut multipart).await {
        Ok((total_rows, _)) => (
            StatusCode::OK,
            Json(ApiResponse::ok(ImportSummary {
                total_rows,
                missing_columns: Vec::new(),
            })),
        ),
        Err(err) => emit_error(err),
    }
}

fn error_response(err: ImportError) -> (StatusCode, Json<ApiResponse<ImportSummary>>) {
    emit_error(err)
}

fn emit_error(err: ImportError) -> (StatusCode, Json<ApiResponse<ImportSummary>>) {
    let status = err.status_code();
    let code = err.api_code();
    let message = err.message();
    if !matches!(status, StatusCode::BAD_REQUEST) {
        eprintln!("[import] {}", message);
    }
    (status, Json(ApiResponse::msg(code, &message)))
}

async fn handle_student_import(
    state: &Arc<AppState>,
    multipart: &mut Multipart,
) -> Result<(usize, Vec<String>), ImportError> {
    let bytes = read_excel_bytes(multipart).await?;
    let range = load_first_sheet(&bytes)?;
    let records = parse_student_records(&range)?;
    let result = student_course::overwrite_all_courses(&state.db_pool, &records).await?;
    Ok((result.affected_rows as usize, result.missing_columns))
}

async fn handle_swim_import(
    state: &Arc<AppState>,
    multipart: &mut Multipart,
) -> Result<(usize, Vec<String>), ImportError> {
    let bytes = read_excel_bytes(multipart).await?;
    let range = load_first_sheet(&bytes)?;
    let records = parse_swim_records(&range)?;
    let affected = swim_course::overwrite_all_customers(&state.db_pool, &records).await?;
    Ok((affected as usize, Vec::new()))
}

async fn read_excel_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, ImportError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ImportError::Multipart(format!("读取上传文件失败: {}", err)))?
    {
        let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
        if field_name != "file" {
            continue;
        }

        let data = field
            .bytes()
            .await
            .map_err(|err| ImportError::Multipart(format!("接收 Excel 文件失败: {}", err)))?;
        return Ok(data.to_vec());
    }

    Err(ImportError::MissingFile)
}

fn open_workbook_from_bytes(bytes: &[u8]) -> Result<Sheets<Cursor<Vec<u8>>>, ImportError> {
    let cursor = Cursor::new(bytes.to_vec());
    calamine::open_workbook_auto_from_rs(cursor).map_err(map_workbook_error)
}

fn pretty_calamine_error(err: calamine::Error) -> String {
    match &err {
        calamine::Error::Xlsx(inner) => {
            let text = inner.to_string();
            if text.contains("Parse integer error") {
                format!(
                    "{}。可能原因：单元格包含非纯数字字符或空字符串，请检查需要为数字/日期的列是否存在例如 '16岁'、'--' 等内容",
                    text
                )
            } else {
                text
            }
        }
        _ => err.to_string(),
    }
}

fn load_first_sheet(bytes: &[u8]) -> Result<Range<DataType>, ImportError> {
    match open_workbook_from_bytes(bytes) {
        Ok(mut workbook) => {
            let range_result = workbook
                .worksheet_range_at(0)
                .ok_or_else(|| ImportError::InvalidExcel("Excel 文件未包含任何工作表".into()))?;
            match range_result {
                Ok(range) => Ok(range),
                Err(err) => {
                    let err_text = err.to_string();
                    if err_text.contains("Parse integer error") {
                        manual_parse_first_sheet(bytes)
                    } else {
                        Err(ImportError::Workbook(format!(
                            "读取工作表失败: {}",
                            pretty_calamine_error(err)
                        )))
                    }
                }
            }
        }
        Err(ImportError::Workbook(message)) if message.contains("Parse integer error") => {
            manual_parse_first_sheet(bytes)
        }
        Err(err) => Err(err),
    }
}

fn map_workbook_error(err: calamine::Error) -> ImportError {
    let message = match &err {
        calamine::Error::Msg(msg) if *msg == "Cannot detect file format" => {
            "无法识别 Excel 文件类型，请确认文件为 .xlsx、.xls、.xlsb 或 .ods 格式".into()
        }
        calamine::Error::Xlsx(inner) => {
            let text = inner.to_string();
            if text.contains("Zip error: invalid Zip archive") {
                "Excel 文件不是有效的 .xlsx 压缩文件，请另存为 .xlsx 后重新上传".into()
            } else if text.contains("Parse integer error") {
                format!(
                    "无法解析 .xlsx 文件: {}。请检查数值列是否含有如 '—'、'N/A' 等非数字字符",
                    text
                )
            } else {
                format!("无法解析 .xlsx 文件: {}", text)
            }
        }
        calamine::Error::Xls(inner) => format!("无法解析 .xls 文件: {}", inner),
        calamine::Error::Xlsb(inner) => format!("无法解析 .xlsb 文件: {}", inner),
        calamine::Error::Ods(inner) => format!("无法解析 .ods 文件: {}", inner),
        _ => format!("无法读取 Excel: {}", err),
    };
    ImportError::Workbook(message)
}

fn manual_parse_first_sheet(bytes: &[u8]) -> Result<Range<DataType>, ImportError> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor)
        .map_err(|err| ImportError::Workbook(format!("无法解压 Excel 文件: {}", err)))?;

    let shared_strings = read_shared_strings(&mut archive)?;
    let sheet_data = read_first_sheet(&mut archive)?;
    let rows = parse_sheet_rows(&sheet_data, &shared_strings)?;
    range_from_rows(rows)
}

fn read_shared_strings(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
) -> Result<Vec<String>, ImportError> {
    match archive.by_name("xl/sharedStrings.xml") {
        Ok(mut file) => {
            let mut xml = String::new();
            file.read_to_string(&mut xml).map_err(|err| {
                ImportError::Workbook(format!("读取 sharedStrings 失败: {}", err))
            })?;
            parse_shared_strings(&xml)
        }
        Err(_) => Ok(Vec::new()),
    }
}

fn read_first_sheet(archive: &mut ZipArchive<Cursor<Vec<u8>>>) -> Result<Vec<u8>, ImportError> {
    for index in 1..=8 {
        let path = format!("xl/worksheets/sheet{}.xml", index);
        if let Ok(mut file) = archive.by_name(&path) {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|err| ImportError::Workbook(format!("读取工作表失败: {}", err)))?;
            return Ok(data);
        }
    }
    Err(ImportError::InvalidExcel(
        "未找到任何工作表，请确认 Excel 内容是否正确".into(),
    ))
}

fn parse_shared_strings(xml: &str) -> Result<Vec<String>, ImportError> {
    let mut reader = XmlReader::from_str(xml);
    reader.trim_text(false);
    let mut buf = Vec::new();
    let mut strings = Vec::new();
    let mut current = String::new();
    let mut in_si = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"si" => {
                current.clear();
                in_si = true;
            }
            Ok(Event::Text(e)) if in_si => {
                let text = e.unescape().map_err(|err| {
                    ImportError::Workbook(format!("解析 sharedStrings 文本失败: {}", err))
                })?;
                current.push_str(&text);
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"si" => {
                strings.push(current.trim().to_string());
                in_si = false;
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(ImportError::Workbook(format!(
                    "解析 sharedStrings 失败: {}",
                    err
                )))
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(strings)
}

fn parse_sheet_rows(
    sheet_xml: &[u8],
    shared_strings: &[String],
) -> Result<Vec<Vec<String>>, ImportError> {
    let mut reader = XmlReader::from_reader(sheet_xml);
    reader.trim_text(false);

    let mut buf = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Option<Vec<String>> = None;
    let mut col_cursor: usize = 0;

    let mut cell_col: Option<usize> = None;
    let mut cell_type: Option<String> = None;
    let mut cell_value: Option<String> = None;
    let mut in_value = false;
    let mut inline_buffer = String::new();
    let mut in_inline = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"row" => {
                current_row = Some(Vec::new());
                col_cursor = 0;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"row" => {
                if let Some(mut row) = current_row.take() {
                    // 去除行尾空字符串
                    while let Some(true) = row.last().map(|cell| cell.trim().is_empty()) {
                        row.pop();
                    }
                    rows.push(row);
                }
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"c" => {
                let meta = parse_cell_start(e, col_cursor)?;
                cell_col = Some(meta.col_index);
                cell_type = meta.cell_type;
                col_cursor = meta.col_index;
                cell_value = None;
                in_value = false;
                in_inline = false;
                inline_buffer.clear();
            }
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"c" => {
                let meta = parse_cell_start(e, col_cursor)?;
                write_cell(
                    &mut current_row,
                    meta.col_index,
                    resolve_cell_value(None, meta.cell_type.as_deref(), shared_strings),
                );
                col_cursor = meta.col_index + 1;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"v" => {
                in_value = true;
                cell_value = Some(String::new());
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"v" => {
                in_value = false;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"is" => {
                in_inline = true;
                inline_buffer.clear();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"is" => {
                in_inline = false;
                cell_value = Some(inline_buffer.clone());
            }
            Ok(Event::Text(e)) => {
                let text = e
                    .unescape()
                    .map_err(|err| ImportError::Workbook(format!("解析单元格文本失败: {}", err)))?;
                if in_value {
                    if let Some(value) = cell_value.as_mut() {
                        value.push_str(&text);
                    }
                } else if in_inline {
                    inline_buffer.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"c" => {
                let value =
                    resolve_cell_value(cell_value.take(), cell_type.as_deref(), shared_strings);
                if let Some(col_index) = cell_col {
                    write_cell(&mut current_row, col_index, value);
                    col_cursor = col_index + 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(ImportError::Workbook(format!(
                    "解析工作表 XML 失败: {}",
                    err
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(rows)
}

#[derive(Default)]
struct CellMeta {
    col_index: usize,
    cell_type: Option<String>,
}

fn parse_cell_start(event: &BytesStart, default_col: usize) -> Result<CellMeta, ImportError> {
    let mut meta = CellMeta {
        col_index: default_col,
        cell_type: None,
    };

    for attr in event.attributes() {
        let attr =
            attr.map_err(|err| ImportError::Workbook(format!("解析单元格属性失败: {}", err)))?;
        let key = attr.key.as_ref();
        let value_vec = attr.value.into_owned();
        let value_str = std::str::from_utf8(&value_vec)
            .map_err(|err| ImportError::Workbook(format!("解析单元格属性失败: {}", err)))?;
        match key {
            b"r" => {
                if let Some(col) = parse_cell_reference(value_str) {
                    meta.col_index = col;
                }
            }
            b"t" => {
                meta.cell_type = Some(value_str.to_string());
            }
            _ => {}
        }
    }

    Ok(meta)
}

fn parse_cell_reference(reference: &str) -> Option<usize> {
    let mut letters = String::new();
    for ch in reference.chars() {
        if ch.is_ascii_alphabetic() {
            letters.push(ch);
        } else {
            break;
        }
    }
    if letters.is_empty() {
        return None;
    }
    let mut col = 0usize;
    for ch in letters.chars() {
        let v = (ch.to_ascii_uppercase() as u8).saturating_sub(b'A') as usize + 1;
        col = col * 26 + v;
    }
    Some(col.saturating_sub(1))
}

fn resolve_cell_value(
    raw_value: Option<String>,
    cell_type: Option<&str>,
    shared_strings: &[String],
) -> String {
    match cell_type {
        Some("s") => raw_value
            .and_then(|value| value.trim().parse::<usize>().ok())
            .and_then(|idx| shared_strings.get(idx).cloned())
            .unwrap_or_default(),
        Some("b") => match raw_value.unwrap_or_default().trim() {
            "1" => "1".to_string(),
            "0" => "0".to_string(),
            other => other.to_string(),
        },
        Some("inlineStr") => raw_value.unwrap_or_default(),
        _ => raw_value.unwrap_or_default(),
    }
}

fn write_cell(row: &mut Option<Vec<String>>, col_index: usize, value: String) {
    if let Some(ref mut row_vec) = row {
        if row_vec.len() <= col_index {
            row_vec.resize(col_index + 1, String::new());
        }
        row_vec[col_index] = value;
    }
}

fn range_from_rows(rows: Vec<Vec<String>>) -> Result<Range<DataType>, ImportError> {
    if rows.is_empty() {
        return Ok(Range::empty());
    }
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    if width == 0 {
        return Ok(Range::empty());
    }

    let height = rows.len() as u32;
    let width_u32 = width as u32;
    let mut range = Range::new((0, 0), (height - 1, width_u32 - 1));

    for (r_idx, row) in rows.into_iter().enumerate() {
        for (c_idx, value) in row.into_iter().enumerate() {
            if value.trim().is_empty() {
                continue;
            }
            range.set_value((r_idx as u32, c_idx as u32), DataType::String(value));
        }
    }

    Ok(range)
}

fn parse_student_records(
    range: &calamine::Range<DataType>,
) -> Result<Vec<StudentCourseImportRow>, ImportError> {
    let rows: Vec<Vec<DataType>> = range.rows().map(|r| r.to_vec()).collect();
    let header_idx = rows
        .iter()
        .position(|row| {
            row.iter()
                .any(|cell| matches!(cell_to_string(cell).as_deref(), Some("学员姓名")))
        })
        .ok_or_else(|| {
            ImportError::InvalidExcel("未找到“学员姓名”表头，确认模板是否正确".into())
        })?;

    let header_map = build_header_map(&rows[header_idx]);
    if !header_map.contains_key("学员姓名") || !header_map.contains_key("手机号") {
        return Err(ImportError::InvalidExcel(
            "Excel 缺少“学员姓名”或“手机号”列，请使用提供的模板".into(),
        ));
    }

    let mut result = Vec::new();
    for (idx, row) in rows.iter().enumerate().skip(header_idx + 1) {
        if row_is_blank(row) {
            continue;
        }

        let student_name = get_required_string(row, &header_map, "学员姓名", idx)?;
        let primary_phone = get_required_string(row, &header_map, "手机号", idx)?;

        let record = StudentCourseImportRow {
            student_name,
            phone_owner_type: get_optional_string(row, &header_map, "手机号身份"),
            primary_phone: Some(primary_phone),
            class_name: get_optional_string(row, &header_map, "所在班级"),
            course_name: get_optional_string(row, &header_map, "课程名称"),
            course_type: get_optional_string(row, &header_map, "课程类型"),
            purchase_quantity: get_optional_string(row, &header_map, "购买数量"),
            gifted_quantity: get_optional_string(row, &header_map, "赠送数量"),
            consumed_quantity: get_optional_string(row, &header_map, "消耗数量"),
            refund_transfer_quantity: get_optional_string(row, &header_map, "退转数量"),
            remaining_quantity: get_optional_string(row, &header_map, "剩余数量"),
            over_attend_quantity: get_optional_string(row, &header_map, "超上数量"),
            consumed_amount: parse_optional_f64(row, &header_map, "课消金额", idx)?,
            remaining_amount: parse_optional_f64(row, &header_map, "剩余课消金额", idx)?,
            absence_count: parse_optional_i32(row, &header_map, "缺课次数", idx)?,
            follower: get_optional_string(row, &header_map, "跟进人"),
            advisor: get_optional_string(row, &header_map, "学管师"),
            expire_date: parse_optional_date(row, &header_map, "到期时间", idx)?,
            gender: get_optional_string(row, &header_map, "性别"),
            wechat_bind_status: get_optional_string(row, &header_map, "微信绑定状态"),
            card_bind_status: get_optional_string(row, &header_map, "绑卡状态"),
            face_capture_status: get_optional_string(row, &header_map, "人脸采集状态"),
            alt_phone_owner_type: get_optional_string(row, &header_map, "备用手机号身份"),
            alt_phone: get_optional_string(row, &header_map, "备用手机号"),
            source: get_optional_string(row, &header_map, "来源"),
            birth_date: parse_optional_date(row, &header_map, "出生日期", idx)?,
            age: parse_optional_i32(row, &header_map, "年龄", idx)?,
            grade: get_optional_string(row, &header_map, "年级"),
            student_number: get_optional_string(row, &header_map, "学号"),
            school: get_optional_string(row, &header_map, "学校"),
            address: get_optional_string(row, &header_map, "住址"),
            tags: get_optional_string(row, &header_map, "标签"),
            remark: get_optional_string(row, &header_map, "备注"),
            created_by: get_optional_string(row, &header_map, "学员创建人"),
        };

        result.push(record);
    }

    if result.is_empty() {
        return Err(ImportError::InvalidExcel(
            "Excel 中未检测到有效数据行，请确认文件内容".into(),
        ));
    }

    Ok(result)
}

fn parse_swim_records(
    range: &calamine::Range<DataType>,
) -> Result<Vec<SwimCustomerImportRow>, ImportError> {
    let rows: Vec<Vec<DataType>> = range.rows().map(|r| r.to_vec()).collect();
    let header_idx = rows
        .iter()
        .position(|row| {
            row.iter()
                .any(|cell| matches!(cell_to_string(cell).as_deref(), Some("门店")))
        })
        .ok_or_else(|| ImportError::InvalidExcel("未找到“门店”表头，确认模板是否正确".into()))?;

    let header_map = build_header_map(&rows[header_idx]);
    if !header_map.contains_key("姓名") || !header_map.contains_key("手机号") {
        return Err(ImportError::InvalidExcel(
            "Excel 缺少“姓名”或“手机号”列，请使用提供的模板".into(),
        ));
    }

    let mut result = Vec::new();
    for (idx, row) in rows.iter().enumerate().skip(header_idx + 1) {
        if row_is_blank(row) {
            continue;
        }

        let store_name = get_required_string(row, &header_map, "门店", idx)?;
        let customer_name = get_required_string(row, &header_map, "姓名", idx)?;
        let mobile = get_required_string(row, &header_map, "手机号", idx)?;

        let record = SwimCustomerImportRow {
            store_name,
            customer_name,
            mobile,
            gender: get_optional_string(row, &header_map, "性别"),
            birth_date: parse_optional_date(row, &header_map, "出生日期", idx)?,
            id_type: get_optional_string(row, &header_map, "证件类型"),
            id_number: get_optional_string(row, &header_map, "证件号码"),
            age: parse_optional_i32(row, &header_map, "年龄", idx)?,
            acquisition_channel: get_optional_string(row, &header_map, "获取渠道"),
            sales_follow_staff: get_optional_string(row, &header_map, "跟进销售"),
            coach_follow_staff: get_optional_string(row, &header_map, "跟进教练"),
            follow_level: get_optional_string(row, &header_map, "跟进等级"),
            membership_start_date: parse_optional_date(row, &header_map, "入会时间", idx)?,
            membership_end_date: parse_optional_date(row, &header_map, "卡截止时间", idx)?,
            sales_follow_status: get_optional_string(row, &header_map, "销售跟进状态"),
            last_sales_follow_date: parse_optional_date(row, &header_map, "上次销售跟进日期", idx)?,
            course_expire_date: parse_optional_date(row, &header_map, "课过期时间", idx)?,
            remaining_private_sessions: parse_optional_i32(row, &header_map, "剩余私教节数", idx)?,
            coach_follow_status: get_optional_string(row, &header_map, "教练跟进状态"),
            last_coach_follow_date: parse_optional_date(row, &header_map, "上次教练跟进日期", idx)?,
            last_checkin_time: parse_optional_datetime(row, &header_map, "上次入场时间", idx)?,
            total_checkins: parse_optional_i32(row, &header_map, "入场总次数", idx)?,
            total_spent_amount: parse_optional_f64(row, &header_map, "消费总金额", idx)?,
            purchased_card_type: get_optional_string(row, &header_map, "已购卡种"),
            current_identity: get_optional_string(row, &header_map, "当前身份"),
            created_at: parse_optional_datetime(row, &header_map, "录入时间", idx)?,
            created_by: get_optional_string(row, &header_map, "录入人"),
            tags: get_optional_string(row, &header_map, "标签"),
            remark: get_optional_string(row, &header_map, "备注"),
        };

        result.push(record);
    }

    if result.is_empty() {
        return Err(ImportError::InvalidExcel(
            "Excel 中未检测到有效数据行，请确认文件内容".into(),
        ));
    }

    Ok(result)
}

fn build_header_map(row: &[DataType]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (idx, cell) in row.iter().enumerate() {
        if let Some(value) = cell_to_string(cell) {
            map.insert(value, idx);
        }
    }
    map
}

fn get_required_string(
    row: &[DataType],
    headers: &HashMap<String, usize>,
    header: &str,
    row_idx: usize,
) -> Result<String, ImportError> {
    headers
        .get(header)
        .and_then(|&idx| row.get(idx))
        .and_then(cell_to_string)
        .ok_or_else(|| {
            ImportError::Parse(format!(
                "第{}行缺少必填字段“{}”，请检查数据",
                row_idx + 1,
                header
            ))
        })
}

fn get_optional_string(
    row: &[DataType],
    headers: &HashMap<String, usize>,
    header: &str,
) -> Option<String> {
    headers
        .get(header)
        .and_then(|&idx| row.get(idx))
        .and_then(cell_to_string)
}

fn parse_optional_i32(
    row: &[DataType],
    headers: &HashMap<String, usize>,
    header: &str,
    row_idx: usize,
) -> Result<Option<i32>, ImportError> {
    match headers.get(header).and_then(|&idx| row.get(idx)) {
        None => Ok(None),
        Some(cell) => match cell_to_string(cell) {
            None => Ok(None),
            Some(value) => {
                if let Some(number) = parse_flexible_i32(&value) {
                    Ok(Some(number))
                } else {
                    Err(ImportError::Parse(format!(
                        "第{}行“{}”列的数值格式不正确: {}",
                        row_idx + 1,
                        header,
                        value
                    )))
                }
            }
        },
    }
}

fn parse_flexible_i32(raw: &str) -> Option<i32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut attempts: Vec<String> = Vec::with_capacity(4);
    attempts.push(trimmed.to_string());

    let trailing = trimmed.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '-' && c != '+');
    if trailing != trimmed && !trailing.is_empty() {
        attempts.push(trailing.to_string());
    }

    let leading = trimmed.trim_start_matches(|c: char| !c.is_ascii_digit() && c != '-' && c != '+');
    if leading != trimmed && !leading.is_empty() {
        attempts.push(leading.to_string());
    }

    if trailing != trimmed {
        let both =
            trailing.trim_start_matches(|c: char| !c.is_ascii_digit() && c != '-' && c != '+');
        if both != trailing && !both.is_empty() {
            attempts.push(both.to_string());
        }
    }

    for attempt in attempts {
        let compact = attempt.replace(',', "");
        if compact.is_empty() {
            continue;
        }
        if let Ok(num) = compact.parse::<i32>() {
            return Some(num);
        }
    }

    None
}

fn parse_optional_f64(
    row: &[DataType],
    headers: &HashMap<String, usize>,
    header: &str,
    row_idx: usize,
) -> Result<Option<f64>, ImportError> {
    match headers.get(header).and_then(|&idx| row.get(idx)) {
        None => Ok(None),
        Some(cell) => match cell_to_string(cell) {
            None => Ok(None),
            Some(value) => value
                .replace(',', "")
                .parse::<f64>()
                .map(Some)
                .map_err(|_| {
                    ImportError::Parse(format!(
                        "第{}行“{}”列的金额格式不正确: {}",
                        row_idx + 1,
                        header,
                        value
                    ))
                }),
        },
    }
}

fn parse_optional_date(
    row: &[DataType],
    headers: &HashMap<String, usize>,
    header: &str,
    row_idx: usize,
) -> Result<Option<NaiveDate>, ImportError> {
    match headers.get(header).and_then(|&idx| row.get(idx)) {
        None => Ok(None),
        Some(cell) => match parse_cell_as_date(cell) {
            Ok(value) => Ok(value),
            Err(message) => Err(ImportError::Parse(format!(
                "第{}行“{}”列的日期无法解析: {}",
                row_idx + 1,
                header,
                message
            ))),
        },
    }
}

fn parse_optional_datetime(
    row: &[DataType],
    headers: &HashMap<String, usize>,
    header: &str,
    row_idx: usize,
) -> Result<Option<NaiveDateTime>, ImportError> {
    match headers.get(header).and_then(|&idx| row.get(idx)) {
        None => Ok(None),
        Some(cell) => match parse_cell_as_datetime(cell) {
            Ok(value) => Ok(value),
            Err(message) => Err(ImportError::Parse(format!(
                "第{}行“{}”列的日期时间无法解析: {}",
                row_idx + 1,
                header,
                message
            ))),
        },
    }
}

fn parse_cell_as_date(cell: &DataType) -> Result<Option<NaiveDate>, String> {
    match cell {
        DataType::Empty => Ok(None),
        DataType::DateTime(value) => excel_serial_to_datetime(*value)
            .map(|dt| Some(dt.date()))
            .ok_or_else(|| value.to_string()),
        DataType::Float(value) => excel_serial_to_datetime(*value)
            .map(|dt| Some(dt.date()))
            .ok_or_else(|| value.to_string()),
        _ => match cell_to_string(cell) {
            None => Ok(None),
            Some(text) => {
                if let Some(date) = parse_date_from_str(&text) {
                    Ok(Some(date))
                } else if let Some(dt) = parse_datetime_from_str(&text) {
                    Ok(Some(dt.date()))
                } else {
                    Err(text)
                }
            }
        },
    }
}

fn parse_cell_as_datetime(cell: &DataType) -> Result<Option<NaiveDateTime>, String> {
    match cell {
        DataType::Empty => Ok(None),
        DataType::DateTime(value) => excel_serial_to_datetime(*value)
            .ok_or_else(|| value.to_string())
            .map(Some),
        DataType::Float(value) => excel_serial_to_datetime(*value)
            .ok_or_else(|| value.to_string())
            .map(Some),
        _ => match cell_to_string(cell) {
            None => Ok(None),
            Some(text) => parse_datetime_from_str(&text).ok_or(text).map(Some),
        },
    }
}

fn parse_date_from_str(value: &str) -> Option<NaiveDate> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed == "--" {
        return None;
    }

    const FORMATS: [&str; 5] = ["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d", "%Y年%m月%d日", "%Y%m%d"];

    for fmt in FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, fmt) {
            return Some(date);
        }
    }

    if trimmed.len() == 8 && trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        let year = trimmed[0..4].parse().ok()?;
        let month = trimmed[4..6].parse().ok()?;
        let day = trimmed[6..8].parse().ok()?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }

    None
}

fn parse_datetime_from_str(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed == "--" {
        return None;
    }

    const DATETIME_FORMATS: [&str; 4] = [
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M",
    ];

    for fmt in DATETIME_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            return Some(dt);
        }
    }

    const DATE_FORMATS: [&str; 3] = ["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d"];

    for fmt in DATE_FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, fmt) {
            return Some(date.and_hms_opt(0, 0, 0)?);
        }
    }

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y%m%d") {
        return Some(date.and_hms_opt(0, 0, 0)?);
    }

    None
}

fn cell_to_string(cell: &DataType) -> Option<String> {
    match cell {
        DataType::Empty => None,
        DataType::String(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() || trimmed == "-" || trimmed == "--" {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        DataType::Float(value) => {
            if !value.is_finite() {
                return None;
            }
            if (value.fract()).abs() < f64::EPSILON {
                Some(format!("{}", *value as i64))
            } else {
                Some(format_decimal(*value))
            }
        }
        DataType::Int(value) => Some(value.to_string()),
        DataType::Bool(flag) => Some(if *flag { "是".into() } else { "否".into() }),
        DataType::DateTime(value) => {
            excel_serial_to_datetime(*value).map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        }
        _ => None,
    }
}

fn row_is_blank(row: &[DataType]) -> bool {
    row.iter().all(|cell| cell_to_string(cell).is_none())
}

fn format_decimal(value: f64) -> String {
    let mut text = format!("{:.6}", value);
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

fn excel_serial_to_datetime(serial: f64) -> Option<NaiveDateTime> {
    if !serial.is_finite() {
        return None;
    }
    let base_date = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let days = serial.trunc() as i64;
    let fractional = serial - serial.trunc();
    let date = base_date.checked_add_signed(Duration::days(days))?;
    let seconds = (fractional * 86_400.0).round() as i64;
    let seconds = (seconds % 86_400 + 86_400) % 86_400;
    let time = NaiveTime::from_num_seconds_from_midnight_opt(seconds as u32, 0)?;
    Some(date.and_time(time))
}
