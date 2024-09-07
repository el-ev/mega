use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use russh_keys::parse_public_key_base64;

use common::model::CommonResult;

use crate::api::{error::ApiError, oauth::model::LoginUser};
use crate::api::user::model::AddSSHKey;
use crate::api::user::model::ListSSHKey;
use crate::api::MonoApiServiceState;

pub fn routers() -> Router<MonoApiServiceState> {
    Router::new()
        .route("/user", get(user))
        .route("/user/ssh", get(list_key))
        .route("/user/ssh", post(add_key))
        .route("/user/ssh/:key_id/delete", post(remove_key))
}

async fn user(
    user: LoginUser,
    _: State<MonoApiServiceState>,
) -> Result<Json<CommonResult<LoginUser>>, ApiError> {
    Ok(Json(CommonResult::success(Some(user))))
}

async fn add_key(
    user: LoginUser,
    state: State<MonoApiServiceState>,
    Json(json): Json<AddSSHKey>,
) -> Result<Json<CommonResult<String>>, ApiError> {
    let key_data = json
        .ssh_key
        .split_whitespace()
        .nth(1)
        .ok_or("Invalid key format")
        .unwrap();

    let key = parse_public_key_base64(key_data)?;

    let res = state
        .context
        .services
        .user_storage
        .save_ssh_key(user.user_id, &json.ssh_key, &key.fingerprint())
        .await;
    let res = match res {
        Ok(_) => CommonResult::success(None),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn remove_key(
    user: LoginUser,
    state: State<MonoApiServiceState>,
    Path(key_id): Path<i64>,
) -> Result<Json<CommonResult<String>>, ApiError> {
    let res = state
        .context
        .services
        .user_storage
        .delete_ssh_key(user.user_id, key_id)
        .await;
    let res = match res {
        Ok(_) => CommonResult::success(None),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn list_key(
    user: LoginUser,
    state: State<MonoApiServiceState>,
) -> Result<Json<CommonResult<Vec<ListSSHKey>>>, ApiError> {
    let res = state
        .context
        .services
        .user_storage
        .list_user_ssh(user.user_id)
        .await;
    let res = match res {
        Ok(data) => CommonResult::success(Some(data.into_iter().map(|x| x.into()).collect())),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}