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
    temp: f32,
    location: String,
    datetime: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TempData {
    data: Vec<Temp>,
}

#[get("/temp/all/<location>")]
async fn get_all_temps(location: String, mut db: Connection<Temperatures>) -> Json<TempData> {
    let cursor = sqlx::query("SELECT temp_f, datetime FROM temperatures WHERE location = $1")
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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TempPostResponse {
    error: Option<String>,
    rows_affected: u64,
}

// make temp and humidity as json body
#[post("/temp/<location>/<temp>")]
async fn post_temp(
    location: String,
    temp: f32,
    mut db: Connection<Temperatures>,
) -> (Status, Json<TempPostResponse>) {
    let affected = sqlx::query("INSERT INTO temperatures(temp_f, location) VALUES ($1,$2)")
        .bind(temp)
        .bind(location)
        .execute(&mut *db)
        .await;

    match affected {
        Ok(num) => {
            if num.rows_affected() > 0 {
                (
                    Status::Created,
                    Json(TempPostResponse {
                        rows_affected: num.rows_affected(),
                        error: None,
                    }),
                )
            } else {
                (
                    Status::InternalServerError,
                    Json(TempPostResponse {
                        error: Some("Zero rows affected".to_string()),
                        rows_affected: num.rows_affected() as u64,
                    }),
                )
            }
        }
        Err(e) => {
            println!("{}", e.to_string());
            (
                Status::InternalServerError,
                Json(TempPostResponse {
                    error: Some(e.to_string()),
                    rows_affected: 0,
                }),
            )
        }
    }
}

#[launch]
fn rocket() -> _ {
    println!("Launching...");
    rocket::build()
        .attach(Temperatures::init())
        .mount("/api/v1", routes![get_all_temps, post_temp])
}
