#[macro_use]
extern crate rocket;

use collector_core::CollectorCore;
use rocket::{http::Status, State};

#[get("/metrics")]
fn metrics(
    state: &State<std::sync::Arc<std::sync::Mutex<CollectorCore>>>,
) -> Result<String, Status> {
    if let Ok(core) = state.try_lock() {
        core.get_metrics().map_err(|e| {
            eprintln!("failed to get metrics: {:#?}", e);
            Status::InternalServerError
        })
    } else {
        Err(Status::InternalServerError)
    }
}

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv()
        .map_err(|e| {
            println!("failed to load .env file : {:#?}", e);
        })
        .ok();

    let core = collector_core::CollectorCore::new();

    rocket::build()
        .mount("/", routes![metrics])
        .manage(std::sync::Arc::new(std::sync::Mutex::new(core)))
}
