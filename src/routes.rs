use crate::error::Error;
use crate::{DATABASE, DB, NAMESPACE};
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put};
use faker_rand::en_us::names::FirstName;
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;
use surrealdb::RecordId;

const PERSON: &str = "person";

#[derive(Serialize, Deserialize)]
pub struct PersonData {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Person {
    name: String,
    id: RecordId,
}

#[get("/")]
pub async fn paths() -> &'static str {
    r#"

-----------------------------------------------------------------------------------------------------------------------------------------
        PATH                |           SAMPLE COMMAND
-----------------------------------------------------------------------------------------------------------------------------------------
/session: See session data  |  curl -X GET    -H "Content-Type: application/json"                          http://localhost:8080/session
                            |
/person/{id}:               |
  Create a person           |  curl -X POST   -H "Content-Type: application/json" -d '{"name":"John Doe"}' http://localhost:8080/person/one
  Get a person              |  curl -X GET    -H "Content-Type: application/json"                          http://localhost:8080/person/one
  Update a person           |  curl -X PUT    -H "Content-Type: application/json" -d '{"name":"Jane Doe"}' http://localhost:8080/person/one
  Delete a person           |  curl -X DELETE -H "Content-Type: application/json"                          http://localhost:8080/person/one
                            |
/people: List all people    |  curl -X GET    -H "Content-Type: application/json"                          http://localhost:8080/people

/new_user:  Create a new record user
/new_token: Get instructions for a new token if yours has expired"#
}

#[get("/session")]
pub async fn session() -> Result<Json<String>, Error> {
    let res: Option<String> = DB.query("RETURN <string>$session").await?.take(0)?;

    Ok(Json(res.unwrap_or("No session data found!".into())))
}

#[post("/person/{id}")]
pub async fn create_person(
    id: Path<String>,
    person: Json<PersonData>,
) -> Result<Json<Option<Person>>, Error> {
    let person = DB.create((PERSON, &*id)).content(person).await?;
    Ok(Json(person))
}

#[get("/person/{id}")]
pub async fn read_person(id: Path<String>) -> Result<Json<Option<Person>>, Error> {
    let person = DB.select((PERSON, &*id)).await?;
    Ok(Json(person))
}

#[put("/person/{id}")]
pub async fn update_person(
    id: Path<String>,
    person: Json<PersonData>,
) -> Result<Json<Option<Person>>, Error> {
    let person = DB.update((PERSON, &*id)).content(person).await?;
    Ok(Json(person))
}

#[delete("/person/{id}")]
pub async fn delete_person(id: Path<String>) -> Result<Json<Option<Person>>, Error> {
    let person = DB.delete((PERSON, &*id)).await?;
    Ok(Json(person))
}

#[get("/people")]
pub async fn list_people() -> Result<Json<Vec<Person>>, Error> {
    let people = DB.select(PERSON).await?;
    Ok(Json(people))
}

#[derive(Serialize, Deserialize)]
struct Params<'a> {
    name: &'a str,
    pass: &'a str,
}

#[get("/new_user")]
pub async fn make_new_user() -> Result<String, Error> {
    let name = rand::random::<FirstName>().to_string();
    let pass = rand::random::<FirstName>().to_string();
    let jwt = DB
        .signup(Record {
            access: "account",
            namespace: NAMESPACE,
            database: DATABASE,
            params: Params {
                name: &name,
                pass: &pass,
            },
        })
        .await?
        .into_insecure_token();
    Ok(format!("New user created!\n\nName: {name}\nPassword: {pass}\nToken: {jwt}\n\nTo log in, use this command:\n\nsurreal sql --namespace {NAMESPACE} --database {DATABASE} --pretty --token \"{jwt}\""))
}

#[get("/new_token")]
pub async fn get_new_token() -> String {
    let command = r#"curl -X POST -H "Accept: application/json" -d '{"ns":"{NAMESPACE}","db":"{DATABASE}","ac":"account","user":"your_username","pass":"your_password"}' http://localhost:8000/signin"#;
    format!("Need a new token? Use this command:\n\n{command}\n\nThen log in with surreal sql --namespace namespace --database database --pretty --token YOUR_TOKEN_HERE")
}
