
use rocket_db_pools::{Database, deadpool_redis};

#[macro_use] extern crate rocket;

mod dict;
mod not_found;
mod routes;

#[derive(Database)]
#[database("redis_pool")]
pub(crate) struct Redis(deadpool_redis::Pool);

#[launch]
fn rocket() -> _ {
    dotenv::dotenv();
    rocket::build()
        .mount("/", routes![routes::res, routes::api, routes::index, routes::define])
        .register("/", catchers![not_found::general_not_found])
        .register("/api", catchers![not_found::api_not_found])
        .attach(Redis::init())
}
