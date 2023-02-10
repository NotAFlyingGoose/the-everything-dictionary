#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

mod routes;
mod dict;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![routes::res, routes::api, routes::index, routes::define])
}
