// mod api;
mod database;
mod logger;
mod model;
mod timer;
mod utils;

// use api::*;
use dotenv::dotenv;
use rocket::{
  self, catch, catchers,
  fairing::{Fairing, Info, Kind},
  http::Header,
  routes, {Request, Response},
};

extern crate bcrypt;

use bcrypt::{hash, verify, DEFAULT_COST};

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

/*
422: 資料格式不正確
*/
#[catch(422)]
fn handle_unprocessable_entity(_: &Request) -> &'static str {
  "Unprocessable Entity"
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  let hashed = hash("hunter2", DEFAULT_COST).unwrap();
  let valid = verify("hunter2", &hashed).unwrap();
  println!("{}  {}", hashed, valid);
  /*
  database::init_db();
  */
  logger::init_logger(log::LevelFilter::Info);
  let catchers = catchers![handle_unprocessable_entity];
  let routes = routes![];
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
