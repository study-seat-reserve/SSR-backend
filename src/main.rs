mod api;
mod database;
mod logger;
mod model;
mod timer;
mod utils;

use std::env;

use api::*;
use dotenv::dotenv;
use rocket::{
  self, catch, catchers,
  fairing::{Fairing, Info, Kind},
  http::Header,
  routes, {Request, Response},
};

use utils::*;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Add CORS headers to responses",
      kind: Kind::Response,
    }
  }

  async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    response.set_header(Header::new(
      "Access-Control-Allow-Methods",
      "POST, GET, PATCH, OPTIONS",
    ));
    response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
  }
}

#[catch(422)]
fn handle_unprocessable_entity(_: &Request) -> &'static str {
  // 請求格式錯誤
  "The request contains invalid parameters"
}
#[catch(403)]
fn handle_forbidden(_: &Request) -> &'static str {
  "You don't have permission to access this resource"
}
#[catch(401)]
fn unauthorized(_: &Request) -> &'static str {
  "Unauthorized access"
}
#[catch(404)]
fn handle_not_found(_: &Request) -> &'static str {
  "The resource was not found"
}
#[catch(500)]
fn handle_internal_server_error(_: &Request) -> &'static str {
  "Something went wrong"
}
#[catch(503)]
fn handle_service_unavailable(_: &Request) -> &'static str {
  // 伺服器當前無法處理請求
  "The server is currently unable to handle the request"
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  logger::init_logger(log::LevelFilter::Info);

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  // let pool = SqlitePool::connect_lazy(&database_url).expect("Failed to create pool.");

  let pool = sqlx::pool::PoolOptions::new()
    .max_lifetime(None)
    .idle_timeout(None)
    .connect(&database_url)
    .await
    .expect("Failed to create pool");

  // let pool = SqlitePool::connect(&database_url)
  //   .await
  //   .expect("Failed to create pool");
  let pool_clone = pool.clone();

  // database::init::clear_table(&pool).await;
  database::init::init_db(&pool_clone).await;

  tokio::spawn(async move {
    timer::start(&pool_clone).await;
  });

  let catchers = catchers![
    handle_unprocessable_entity,
    handle_forbidden,
    handle_not_found,
    handle_internal_server_error,
    handle_service_unavailable,
    unauthorized
  ];
  let routes = routes![
    register,
    login,
    show_current_seats_status,
    reserve_seat,
    show_seats_status_in_specific_timeslots,
    show_seat_reservations,
    update_reservation,
    delete_reservation_time,
    display_user_reservations,
    email_verify,
    resend_verification_email,
    set_unavailable_timeslots,
    set_seat_availability
  ];
  let server = rocket::build()
    .register("/", catchers)
    .mount("/", routes)
    .attach(CORS)
    .manage(pool)
    .launch();

  tokio::select! {
      _ = server => {},
      _ = tokio::signal::ctrl_c() => {},
  }
}
