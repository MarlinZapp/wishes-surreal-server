use actix_web::{App, HttpServer};
use std::sync::LazyLock;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::TiKv;
use surrealdb::opt::auth::Root;
use surrealdb::opt::Config;
use surrealdb::Surreal;

mod error;
mod routes;

const NAMESPACE: &str = "test";
const DATABASE: &str = "test";

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

    DB.query(
        "
DEFINE TABLE person TYPE ANY SCHEMALESS
	PERMISSIONS
		FOR select, create
			WHERE $auth
		FOR update, delete
			WHERE created_by = $auth;
DEFINE FIELD name ON person TYPE string
	PERMISSIONS FULL;
DEFINE FIELD created_by ON person READONLY VALUE $auth
	PERMISSIONS FULL;
DEFINE INDEX unique_name ON user FIELDS name UNIQUE;
DEFINE ACCESS account ON DATABASE TYPE RECORD
    SIGNUP (CREATE user SET name = $name, pass = crypto::argon2::generate($pass))
    SIGNIN (SELECT * FROM user WHERE name = $name AND crypto::argon2::compare(pass, $pass))
    DURATION FOR TOKEN 15m, FOR SESSION 12h;
        ",
    )
    .await?;

    HttpServer::new(|| {
        App::new()
            .service(routes::create_person)
            .service(routes::read_person)
            .service(routes::update_person)
            .service(routes::delete_person)
            .service(routes::list_people)
            .service(routes::paths)
            .service(routes::session)
            .service(routes::make_new_user)
            .service(routes::get_new_token)
    })
    .bind(("localhost", 8080))?
    .run()
    .await?;

    Ok(())
}
