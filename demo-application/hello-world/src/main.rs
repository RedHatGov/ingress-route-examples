#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!\n"
}

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!\n", name)
}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![hello])
}
