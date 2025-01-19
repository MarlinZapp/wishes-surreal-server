use crate::auth::AuthToken;
use crate::error::Error;
use crate::{DATABASE, DB, NAMESPACE, TABLE_WISH};
use actix_web::web::{Json, Path};
use actix_web::{delete, get, patch, post, HttpRequest};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;
use surrealdb::RecordId;

#[derive(Serialize, Deserialize, Debug)]
pub enum WishStatus {
    Submitted,
    CreationInProgress,
    InDelivery,
    Delivered,
}

#[derive(Serialize, Deserialize)]
pub struct WishCreateRequest {
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct InfoResponse {
    info: String,
    user: Option<User>,
    session: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserRole {
    Default,
    Admin,
}

#[derive(Serialize, Deserialize, Clone)]
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
    id: RecordId,
    content: String,
    status: WishStatus,
    created_by: Option<RecordId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: RecordId,
    name: String,
    pass: String,
    roles: Vec<UserRole>,
}

#[get("/")]
pub async fn paths() -> &'static str {
    r#"

-----------------------------------------------------------------------------------------------------------------------------------------
        PATH                |           SAMPLE COMMAND
-----------------------------------------------------------------------------------------------------------------------------------------                            |
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

#[get("/api/check/auth")]
pub async fn check_auth(req: HttpRequest) -> Result<Json<InfoResponse>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let session: Option<String> = DB.query("RETURN <string>$session").await?.take(0)?;
    let user: Option<User> = DB.query("SELECT * FROM $auth").await?.take(0)?;
    DB.invalidate().await?;
    Ok(Json(InfoResponse {
        info: "Success!".into(),
        user,
        session,
    }))
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

#[patch("/api/wish/{id}/status/progress")]
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
                WishStatus::Submitted => {
                    wish.status = WishStatus::CreationInProgress;
                }
                WishStatus::CreationInProgress => {
                    wish.status = WishStatus::InDelivery;
                }
                WishStatus::InDelivery => {
                    wish.status = WishStatus::Delivered;
                }
                WishStatus::Delivered => {
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
struct SignupParams<'a> {
    name: &'a str,
    pass: &'a str,
    roles: Vec<UserRole>,
}

#[derive(Serialize, Deserialize)]
struct SigninParams<'a> {
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
            params: SignupParams {
                name: &req.name,
                pass: &req.pass,
                roles: vec![UserRole::Default],
            },
        })
        .await?
        .into_insecure_token();
    Ok(Json(jwt))
}

#[post("/api/login")]
pub async fn login(req: Json<Credentials>) -> Result<Json<String>, Error> {
    let jwt = DB
        .signin(Record {
            access: "account",
            namespace: NAMESPACE,
            database: DATABASE,
            params: SigninParams {
                name: &req.name,
                pass: &req.pass,
            },
        })
        .await?
        .into_insecure_token();
    Ok(Json(jwt))
}
