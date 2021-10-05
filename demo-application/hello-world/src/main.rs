#[macro_use] extern crate rocket;

fn hn() -> String {
    hostname::get()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

#[get("/")]
fn index() -> String {

    format!("Hello, world, from {}!\n", hn())
}

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}, from {}!\n", name, hn())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![hello])
}
