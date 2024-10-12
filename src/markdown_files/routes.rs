use std::fs;
use axum::{Json, Router};
use axum::routing::get;
use crate::backend::{Backend, DBConnection};
use crate::error::{APIError, APIResult};
use crate::markdown_files::WORKSHOP_TEXT_SUB_PATH;
use crate::markdown_files::BACKEND_PATH;

#[utoipa::path(
    get,
    path = "/possible_workshops"
)]
pub async fn get_event_user(
    mut conn: DBConnection,
) -> APIResult<Json<Vec<String>>> {
    let entries = fs::read_dir(format!("{BACKEND_PATH}{WORKSHOP_TEXT_SUB_PATH}"))
        .map_err(APIError::internal)?;
    
    let file_names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();

    Ok(Json(file_names))
}

pub fn add_admin_markdown_files_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/possible_workshops", get(get_event_user))
}