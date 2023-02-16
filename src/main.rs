
use std::env;

use rocket_db_pools::{Database, deadpool_redis};

#[macro_use] extern crate rocket;

mod routes;
mod dict;

#[derive(Database)]
#[database("redis_pool")]
pub(crate) struct Redis(deadpool_redis::Pool);

#[launch]
fn rocket() -> _ {
    dotenv::dotenv();
    rocket::build()
        .mount("/", routes![routes::res, routes::api, routes::index, routes::define])
        .attach(Redis::init())
}
