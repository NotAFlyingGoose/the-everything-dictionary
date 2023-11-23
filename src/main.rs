use rocket_db_pools::{deadpool_redis, Database};

#[macro_use]
extern crate rocket;

mod dict;
mod not_found;
mod routes;

#[derive(Database)]
#[database("redis_pool")]
pub(crate) struct Redis(deadpool_redis::Pool);

#[launch]
fn rocket() -> _ {
    let _dotenv = dotenv::dotenv();
    rocket::build()
        .mount("/", routes![routes::guantanamo_bay, routes::res])
        .register("/", catchers![not_found::general_not_found])
        .register("/api", catchers![not_found::api_not_found])
        .attach(Redis::init())
}
