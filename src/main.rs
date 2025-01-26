use actix_web::{App, HttpServer};
use routes::UserRole;
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
    let host_with_port = std::env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:2379".to_string());
    DB.connect::<TiKv>((format!("{host_with_port}"), Config::default().strict()))
        .await?;

    DB.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    DB.use_ns(NAMESPACE).use_db(DATABASE).await?;

    let wish_status_enum: String = format!(
        "'{:?}' | '{:?}' | '{:?}' | '{:?}'",
        WishStatus::Submitted,
        WishStatus::CreationInProgress,
        WishStatus::InDelivery,
        WishStatus::Delivered,
    );
    let user_roles_enum: String = format!("'{:?}' | '{:?}' ", UserRole::Default, UserRole::Admin,);
    DB.query(format!(
        "
DEFINE TABLE OVERWRITE {TABLE_WISH} SCHEMAFULL
	PERMISSIONS
		FOR create
			WHERE $auth
		FOR select, update, delete
		  WHERE created_by.id = $auth.id OR 'Admin' IN $auth.roles;
DEFINE FIELD OVERWRITE content ON {TABLE_WISH} TYPE string
	PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON {TABLE_WISH} TYPE {wish_status_enum}
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE created_by ON {TABLE_WISH} READONLY VALUE $auth
	PERMISSIONS FULL;

DEFINE TABLE OVERWRITE {TABLE_USER} SCHEMAFULL
	PERMISSIONS
		FOR select, update, delete WHERE id = $auth.id OR 'Admin' IN $auth.roles;
DEFINE FIELD OVERWRITE name ON {TABLE_USER} TYPE string
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE pass ON {TABLE_USER} TYPE string
    PERMISSIONS FULL;
DEFINE FIELD OVERWRITE roles ON {TABLE_USER} TYPE array<{user_roles_enum}>
    PERMISSIONS FULL;
DEFINE INDEX OVERWRITE unique_name ON {TABLE_USER} FIELDS name UNIQUE;
DEFINE ACCESS OVERWRITE {ACCESS_RULE_ACCOUNT} ON DATABASE TYPE RECORD
    SIGNUP (
        CREATE {TABLE_USER} SET name = $name, pass = crypto::argon2::generate($pass), roles = $roles
    )
    SIGNIN (SELECT * FROM {TABLE_USER} WHERE name = $name AND crypto::argon2::compare(pass, $pass))
    DURATION FOR TOKEN 15m, FOR SESSION 12h;
        ",
    ))
    .await?;

    println!("SurrealDB server connected to TiKv Cluster on 127.0.0.1:{host_with_port}. Accessible via http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .service(routes::check_auth)
            .service(routes::create_wish)
            .service(routes::create_wish_with_id)
            .service(routes::read_wish)
            .service(routes::progress_wish_status)
            .service(routes::delete_wish)
            .service(routes::list_wishes)
            .service(routes::paths)
            .service(routes::register_user)
            .service(routes::login)
    })
    .bind(("localhost", 8080))?
    .run()
    .await?;
    Ok(())
}
