#[macro_use]
extern crate rocket;

mod dict;
mod not_found;
mod routes;

#[launch]
fn rocket() -> _ {
    let _dotenv = dotenv::dotenv();
    rocket::build()
        .mount("/", routes![routes::guantanamo_bay, routes::res])
        .register("/", catchers![not_found::general_not_found])
        .register("/api", catchers![not_found::api_not_found])
}
