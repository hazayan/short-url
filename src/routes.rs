use crc::{crc32, Hasher32};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use tera::Tera;
use tide::{Redirect, Request, Response, StatusCode};
use tide_tera::prelude::*;

use crate::db::*;

#[derive(Deserialize)]
struct Form {
    input_url: String,
}

#[derive(Serialize)]
struct Error {
    status: StatusCode,
    msg: String,
}

#[derive(Clone)]
pub struct Globals {
    pub tera: Tera,
    pub pool: SqlitePool,
}

fn hash_url(url: &String) -> String {
    let mut digest = crc32::Digest::new(crc32::IEEE);
    digest.write(url.as_bytes());
    format!("{:x}", digest.sum32())
}

fn short_url(url: &String) -> String {
    let scheme = dotenv::var("URL_SCHEME").unwrap();
    let base_redirect_url = dotenv::var("BASE_REDIRECT_URL").unwrap();
    format!("{}://{}/r/{}", scheme, base_redirect_url, hash_url(url))
}

pub async fn render_home(req: Request<Globals>) -> tide::Result {
    let tera = &req.state().tera;
    tera.render_response("input.html", &context! {})
}

pub async fn render_hashed_url(mut req: Request<Globals>) -> tide::Result {
    let content: Form = req.body_form().await?;
    let tera = &req.state().tera;
    let pool = &req.state().pool;
    let hash = hash_url(&content.input_url);
    add_hash(&pool, &content.input_url, &hash).await?;
    tera.render_response(
        "output.html",
        &context! {"short_url" => short_url(&content.input_url)},
    )
}

pub async fn redirect(req: Request<Globals>) -> tide::Result {
    let tera = &req.state().tera;
    let pool = &req.state().pool;
    let hash = req.param("hash").unwrap().to_string();
    let url = url_from_hash(&pool, &hash).await;
    match url {
        Err(_) => tera.render_response("not_found.html", &context! {}),
        Ok(url) => Ok(Redirect::new(url).into()),
    }
}

pub async fn handle_error(mut res: Response) -> tide::Result {
    let status = res.status();
    match status.is_client_error() || status.is_server_error() {
        true => {
            let msg: String = match status {
                StatusCode::UnprocessableEntity => "Unprocessable entity",
                StatusCode::NotFound => "Not found",
                StatusCode::Forbidden => "Forbidden access",
                StatusCode::Unauthorized => "Unauthorized access",
                StatusCode::BadRequest => "Bad request",
                _ => "Internal server error",
            }
            .to_string();
            res.set_body(tide::convert::json!(Error { status, msg }));
            Ok(res)
        }
        false => Ok(res),
    }
}
