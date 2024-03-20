#[macro_use]
extern crate rocket;


use std::io::Read;

use rocket::{http::Status, State};

#[get("/metrics")]
fn metrics(
    state: &State<std::sync::Arc<std::sync::Mutex<std::process::ChildStdout>>>,
) -> Result<String, Status> {
    let mut result = String::new();
    {
        let mut stdout = state.try_lock().map_err(|e| {
            println!("コアのstdoutがロックされています。: {:?}", e);
            Status::InternalServerError
        })?;

        let mut buf = [0; 2];
        while 0 < stdout.read(&mut buf).unwrap_or(0) {
            if (buf[0] == '\n' as u8 || buf[0] == '\r' as u8) && buf[1] == '\n' as u8 {
                break;
            }
            result.push(buf[0] as char);
            result.push(buf[1] as char);
            buf = [0; 2];
        }
    }

    println!("{}", result);

    Ok(result)
}

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv()
        .map_err(|e| {
            println!("failed to load .env file : {:#?}", e);
        })
        .ok();

    let mut child = std::process::Command::new(std::env::var("CORE_PATH").unwrap_or("./collector-core".to_string()))
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("コアの起動に失敗しました。");

    let stdout = child.stdout.take().expect("コアのstdoutがありません。");

    rocket::build()
        .mount("/", routes![metrics])
        .manage(std::sync::Arc::new(std::sync::Mutex::new(stdout)))
}
