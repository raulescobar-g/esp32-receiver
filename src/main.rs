#[macro_use]
extern crate rocket;

use chrono::prelude::*;
use rocket::http::Status;
use rocket::serde::{json::Json, Serialize};
use rocket_db_pools::Connection;
use rocket_db_pools::{sqlx, Database};
use sqlx::Row;

#[derive(Database)]
#[database("temperatures_database")]
struct Temperatures(sqlx::PgPool);

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Temp {
    temp: i32,
    location: String,
    datetime: NaiveDateTime,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TempData {
    data: Vec<Temp>,
}

#[get("/temp/all/<location>")]
async fn get_all_temps(location: String, mut db: Connection<Temperatures>) -> Json<TempData> {
    let cursor = sqlx::query("SELECT temp_f, datetime FROM temperatures WHERE location = ?")
        .bind(location.clone())
        .fetch_all(&mut *db)
        .await
        .expect("pg rows");

    let data = cursor
        .iter()
        .map(|r| Temp {
            temp: r.get("temp_f"),
            location: location.clone(),
            datetime: r.get("datetime"),
        })
        .collect::<Vec<Temp>>();

    Json(TempData { data })
}

#[post("/temp/<location>/<temp>")]
async fn post_temp(location: String, temp: i32, mut db: Connection<Temperatures>) -> Status {
    let affected = sqlx::query("INSERT INTO temperatures(temp_f, location) VALUES (?,?)")
        .bind(location)
        .bind(temp)
        .execute(&mut *db)
        .await;

    match affected {
        Ok(num) => {
            if num.rows_affected() > 0 {
                Status::Accepted
            } else {
                Status::BadRequest
            }
        }
        _ => Status::InternalServerError,
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Temperatures::init())
        .mount("/api/v1/", routes![get_all_temps, post_temp])
}
