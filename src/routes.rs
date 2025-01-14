use std::fmt;

use crate::auth::AuthToken;
use crate::error::Error;
use crate::{DATABASE, DB, NAMESPACE, TABLE_WISH};
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put, HttpRequest};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;
use surrealdb::RecordId;

#[derive(Serialize, Deserialize)]
pub enum WishStatus {
    Submitted,
    CreationInProgress,
    InDelivery,
    Delivered,
}

impl fmt::Display for WishStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WishStatus::Submitted => write!(f, "Submitted"),
            WishStatus::CreationInProgress => write!(f, "CreationInProgress"),
            WishStatus::InDelivery => write!(f, "InDelivery"),
            WishStatus::Delivered => write!(f, "Delivered"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WishCreateRequest {
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    name: String,
    pass: String,
}

#[derive(Serialize, Deserialize)]
pub struct WishContent {
    content: String,
    status: WishStatus,
}

#[derive(Serialize, Deserialize)]
pub struct Wish {
    content: String,
    status: WishStatus,
    id: RecordId,
}

#[get("/")]
pub async fn paths() -> &'static str {
    r#"

-----------------------------------------------------------------------------------------------------------------------------------------
        PATH                |           SAMPLE COMMAND
-----------------------------------------------------------------------------------------------------------------------------------------
/api/session: See session data  |  curl -X GET    -H "Content-Type: application/json"                           http://localhost:8080/session
                            |
/api/wish/{id}:                 |
  Create a wish             |  curl -X POST   -H "Content-Type: application/json" -d '{"content":"Buch"}'     http://localhost:8080/wish/one
  Get a wish                |  curl -X GET    -H "Content-Type: application/json"                             http://localhost:8080/wish/one
  Progress to next status   |  curl -X PUT    -H "Content-Type: application/json"                             http://localhost:8080/wish/one
  Submitted -> CreationInProgress -> InDelivery -> Delivered
  Delete a wish             |  curl -X DELETE -H "Content-Type: application/json"                             http://localhost:8080/wish/one
                            |
/api/wishes: List all wishes    |  curl -X GET    -H "Content-Type: application/json"                           http://localhost:8080/wishes

/api/register: Register a new user.
curl -X POST -H "Content-Type: application/json" -d '{"name": "Test", "pass":"123"}' http://localhost:8080/register

/api/new_token: Get instructions for a new token if yours has expired"#
}

#[get("/api/session")]
pub async fn session() -> Result<Json<String>, Error> {
    let res: Option<String> = DB.query("RETURN <string>$session").await?.take(0)?;

    Ok(Json(res.unwrap_or("No session data found!".into())))
}

#[post("/api/wish")]
pub async fn create_wish(
    wish: Json<WishCreateRequest>,
    req: HttpRequest,
) -> Result<Json<Option<Wish>>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let wish = WishContent {
        content: wish.content.clone(),
        status: WishStatus::Submitted,
    };
    let response = DB.create(TABLE_WISH).content(wish).await?;
    DB.invalidate().await?;
    Ok(Json(response))
}

#[post("/api/wish/{id}")]
pub async fn create_wish_with_id(
    id: Path<String>,
    wish: Json<WishCreateRequest>,
    req: HttpRequest,
) -> Result<Json<Option<Wish>>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let wish = WishContent {
        content: wish.content.clone(),
        status: WishStatus::Submitted,
    };
    let response = DB.create((TABLE_WISH, &*id)).content(wish).await?;
    DB.invalidate().await?;
    Ok(Json(response))
}

#[get("/api/wish/{id}")]
pub async fn read_wish(id: Path<String>, req: HttpRequest) -> Result<Json<Option<Wish>>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let wish = DB.select((TABLE_WISH, &*id)).await?;
    DB.invalidate().await?;
    Ok(Json(wish))
}

#[delete("/api/wish/{id}")]
pub async fn delete_wish(id: Path<String>, req: HttpRequest) -> Result<Json<Option<Wish>>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let wish = DB.delete((TABLE_WISH, &*id)).await?;
    DB.invalidate().await?;
    Ok(Json(wish))
}

#[put("/api/wish/{id}/status/progress")]
/// Update wish progress status
pub async fn progress_wish_status(
    id: Path<String>,
    req: HttpRequest,
) -> Result<Json<Option<Wish>>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let wish: Option<Wish> = DB.select((TABLE_WISH, &*id)).await?;
    match wish {
        None => {
            DB.invalidate().await?;
            return Ok(Json(None));
        }
        Some(mut wish) => {
            match wish.status {
                WishStatus::Delivered => {
                    wish.status = WishStatus::CreationInProgress;
                }
                WishStatus::CreationInProgress => {
                    wish.status = WishStatus::InDelivery;
                }
                WishStatus::InDelivery => {
                    wish.status = WishStatus::Delivered;
                }
                WishStatus::Submitted => {
                    DB.invalidate().await?;
                    return Ok(Json(None));
                }
            };
            let wish = DB.update((TABLE_WISH, &*id)).content(wish).await?;
            DB.invalidate().await?;
            Ok(Json(wish))
        }
    }
}

#[get("/api/wishes")]
pub async fn list_wishes(req: HttpRequest) -> Result<Json<Vec<Wish>>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let wishes = DB.select(TABLE_WISH).await?;
    DB.invalidate().await?;
    Ok(Json(wishes))
}

#[derive(Serialize, Deserialize)]
struct Params<'a> {
    name: &'a str,
    pass: &'a str,
}

#[post("/api/register")]
pub async fn register_user(req: Json<Credentials>) -> Result<Json<String>, Error> {
    let jwt = DB
        .signup(Record {
            access: "account",
            namespace: NAMESPACE,
            database: DATABASE,
            params: Params {
                name: &req.name,
                pass: &req.pass,
            },
        })
        .await?
        .into_insecure_token();
    Ok(Json(jwt))
}

#[post("/api/login")]
pub async fn get_new_token(req: Json<Credentials>) -> Result<Json<String>, Error> {
    let jwt = DB
        .signin(Record {
            access: "account",
            namespace: NAMESPACE,
            database: DATABASE,
            params: Params {
                name: &req.name,
                pass: &req.pass,
            },
        })
        .await?
        .into_insecure_token();
    Ok(Json(jwt))
}
