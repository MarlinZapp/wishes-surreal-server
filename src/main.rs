use actix_web::{App, HttpServer};
use routes::WishStatus;
use std::sync::LazyLock;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::TiKv;
use surrealdb::opt::auth::Root;
use surrealdb::opt::Config;
use surrealdb::Surreal;

mod auth;
mod error;
mod routes;

const NAMESPACE: &str = "test";
const DATABASE: &str = "test";
const TABLE_USER: &str = "user";
const TABLE_WISH: &str = "wish";
const ACCESS_RULE_ACCOUNT: &str = "account";

static DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    DB.connect::<TiKv>(("127.0.0.1:2379", Config::default().strict()))
        .await?;

    DB.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    DB.use_ns(NAMESPACE).use_db(DATABASE).await?;

    let wish_status_enum: String = "'".to_string()
        + WishStatus::Submitted.to_string().as_str()
        + "' | '"
        + &WishStatus::CreationInProgress.to_string()
        + "' | '"
        + &WishStatus::InDelivery.to_string()
        + "' | '"
        + &WishStatus::Delivered.to_string()
        + "'";
    DB.query(format!(
        "
DEFINE TABLE {TABLE_WISH} TYPE ANY SCHEMALESS
	PERMISSIONS
		FOR select, create
			WHERE $auth
		FOR update, delete
			WHERE created_by = $auth;
DEFINE FIELD content ON {TABLE_WISH} TYPE string
	PERMISSIONS FULL;
DEFINE FIELD status ON {TABLE_WISH} TYPE {wish_status_enum}
    PERMISSIONS FULL;
DEFINE FIELD created_by ON {TABLE_WISH} READONLY VALUE $auth
	PERMISSIONS FULL;
DEFINE INDEX unique_name ON {TABLE_USER} FIELDS name UNIQUE;
DEFINE ACCESS {ACCESS_RULE_ACCOUNT} ON DATABASE TYPE RECORD
    SIGNUP (CREATE {TABLE_USER} SET name = $name, pass = crypto::argon2::generate($pass))
    SIGNIN (SELECT * FROM {TABLE_USER} WHERE name = $name AND crypto::argon2::compare(pass, $pass))
    DURATION FOR TOKEN 15m, FOR SESSION 12h;
        ",
    ))
    .await?;

    HttpServer::new(|| {
        App::new()
            .service(routes::create_wish)
            .service(routes::create_wish_with_id)
            .service(routes::read_wish)
            .service(routes::progress_wish_status)
            .service(routes::delete_wish)
            .service(routes::list_wishes)
            .service(routes::paths)
            .service(routes::session)
            .service(routes::register_user)
            .service(routes::get_new_token)
    })
    .bind(("localhost", 8080))?
    .run()
    .await?;

    Ok(())
}
