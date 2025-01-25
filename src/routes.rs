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

#[derive(Serialize, Deserialize)]
pub struct WishWithUsername {
    id: RecordId,
    content: String,
    status: WishStatus,
    created_by: Option<RecordId>,
    username: Option<String>,
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
====================================================================================================================================================================================================================================
    PATH            |   METHOD      |   DESCRIPTION                                 |   SAMPLE COMMAND
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/api/wish/{id}      |   POST        | Create a wish                                 | curl -X POST -H "Content-Type: application/json" -H "Authorization: YOUR_JWT_GOES_HERE" -d '{"content":"Buch"}' http://localhost:8080/api/wish
                    |               | [Needs authenticated session]                 |
                    |               |                                               |
                    |   GET         | Get a wish                                    | curl -X GET -H "Content-Type: application/json" -H "Authorization: YOUR_JWT_GOES_HERE" http://localhost:8080/api/wish/wish_id
                    |               | [Needs authenticated session]                 |
                    |               |                                               |
                    |   PATCH       | Progress to next status                       | curl -X PATCH -H "Content-Type: application/json" -H "Authorization: YOUR_JWT_GOES_HERE" http://localhost:8080/api/wish/wish_id/status/progress
                    |               | Submitted -> CreationInProgress ->            |
                    |               | InDelivery -> Delivered                       |
                    |               | [Needs authenticated session]                 |
                    |               |                                               |
                    |               |                                               |
                    |   DELETE      | Delete a wish                                 | curl -X DELETE -H "Content-Type: application/json" -H "Authorization: YOUR_JWT_GOES_HERE" http://localhost:8080/api/wish/wish_id
                    |               | [Needs authenticated session]                 |
                    |               |                                               |
-------------------------------------------------------------------------------------------------------------------------
/api/wishes         |   GET         | List all wishes                               | curl -X GET -H "Content-Type: application/json" -H "Authorization: YOUR_JWT_GOES_HERE" http://localhost:8080/api/wishes
                    |               | [Needs authenticated session]                 |
                    |               |                                               |
-------------------------------------------------------------------------------------------------------------------------
/api/register       |   POST        | Register a new user                           | curl -X POST   -H "Content-Type: application/json" -d '{"name": "Test", "pass":"123"}' http://localhost:8080/api/register
                    |               |                                               |
-------------------------------------------------------------------------------------------------------------------------
/api/login          |   POST        | Login as existing user                        | curl -X POST   -H "Content-Type: application/json" -d '{"name": "Test", "pass":"123"}' http://localhost:8080/api/login
                    |               |                                               |
=========================================================================================================================
    "#
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

#[derive(Serialize, Deserialize)]
struct WishesQueryParams {
    with_username: bool,
}
#[get("/api/wishes")]
pub async fn list_wishes(
    req: HttpRequest,
    query_params: actix_web::web::Query<WishesQueryParams>,
) -> Result<Json<Vec<WishWithUsername>>, Error> {
    let auth: Result<AuthToken, &'static str> = req.try_into();
    let auth = auth.map_err(|e| return Error::Db(e.into()))?;
    DB.authenticate(auth.0).await?;
    let query = if query_params.with_username {
        format!("SELECT *, created_by.name AS username FROM {TABLE_WISH}")
    } else {
        format!("SELECT * FROM {TABLE_WISH}")
    };
    let wishes: Vec<WishWithUsername> = DB.query(query).await?.take(0)?;
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
