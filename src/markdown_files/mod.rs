pub mod routes;

use std::collections::HashMap;
use std::fs;
use markdown_meta_parser::MetaData;
use crate::error::{APIError, APIResult};
use crate::events::public::EventDate;
use crate::events::users::{EventUser, EventUserState};
use crate::user_data::UserData;

pub const BACKEND_PATH: &str = "../hugo_website/content/backend";
pub const WORKSHOP_TEXT_SUB_PATH: &str = "/workshops";
pub const EVENT_BASE_TEXT_SUB_PATH: &str = "/event_base";

pub fn get_file_content(path_form_backend: &str) -> APIResult<String> {
    fs::read_to_string(format!("{BACKEND_PATH}{path_form_backend}"))
        .map_err(APIError::internal)
}

pub struct MailFileMataData {
    pub title: String
}

pub fn get_mail_file_meta_data(content: String) -> APIResult<(MailFileMataData, String)> {
    let required = vec!["title".to_string()];

    let mut type_mark = HashMap::new();
    type_mark.insert("title".into(), "string");

    let meta = MetaData {
        content,
        required,
        type_mark,
    };

    let (res, rest_content) = meta.parse().map_err(APIError::internal)?;

    Ok((MailFileMataData {
        title: res.get("title".into()).cloned().unwrap().as_string().unwrap(),
    }, rest_content))
}


pub fn populate_mail_file_with_email(mut content: String, email: &str) -> String {
    content.replace("{EMail}", email)
}

pub fn populate_mail_file_with_user_data(mut content: String, user_data: &UserData) -> String {
    content.replace("{Name}", &user_data.name)
        .replace("{Fetlife Name}", &user_data.fetlife_name)
}

pub fn populate_mail_file_with_event_data(mut content: String, event_data: &EventDate) -> String {
    content.replace("{Event Date}", &event_data.date.format("%d.%m.%Y").to_string())
        .replace("{Event Time}", &event_data.date.format("%H:%M").to_string())
}

pub fn populate_mail_file_with_event_user(mut content: String, event_user: &EventUser) -> String {
    let slots = if event_user.state == EventUserState::New || event_user.state == EventUserState::WaitingNew
    {
        event_user.new_slot.to_string()
    }  else {
        event_user.slot.to_string()
    };

    content.replace("{Event User State}", match event_user.state {
        EventUserState::Registered => {"Zum Event angenommen"}
        EventUserState::Waiting => {"Warteliste"}
        EventUserState::Rejected => {"Abgelent"}
        EventUserState::New => {"Platz als Neuling"}
        EventUserState::WaitingNew => {"Warteliste als Neuling"}
    })
        .replace("{Guests}", &event_user.guests.to_string())
        .replace("{Attended}", &event_user.attended.to_string())
        .replace("{Slot}", &slots)
}

pub fn populate_mail_file_with_url(mut content: String, url: &str) -> String {
    content.replace("{URL}", url)
}

pub fn populate_workshop_file_with_workshop(mut content: String, workshop: &str) -> String {
    content.replace("{Workshop}", workshop)
}


pub fn expect_content_populated(content: &str) -> APIResult<()> {
    if content.contains(['{', '}']) {
        Err(APIError::internal("Content still has Braces"))
    } else {
        Ok(())
    }
}

