mod api;
mod database;
mod logger;
mod model;
mod timer;
mod utils;

use api::*;
use dotenv::dotenv;
use rocket::{
  self, catch, catchers,
  fairing::{Fairing, Info, Kind},
  http::Header,
  routes, {Request, Response},
};

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

  database::init_db();

  logger::init_logger(log::LevelFilter::Info);
  let catchers = catchers![
    handle_unprocessable_entity,
    handle_forbidden,
    handle_not_found,
    handle_internal_server_error,
    handle_service_unavailable
  ];
  let routes = routes![
    register,
    show_current_seats_status,
    reserve_seat,
    show_seats_status_by_time
  ];
  let server = rocket::build()
    .register("/", catchers)
    .mount("/", routes)
    .attach(CORS)
    // .manage()
    .launch();

  tokio::select! {
      _ = server => {},
      _ = tokio::signal::ctrl_c() => {},
  }
}
