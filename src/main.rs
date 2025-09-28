#[macro_use] extern crate rocket;

use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::fs::{FileServer, relative};
use rocket::Config;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct CoordinateRequest {
    coordinate: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct CoordinateResponse {
    coordinate: String,
    voice: String,
}

#[post("/engage", data = "<request>")]
fn engage(request: Json<CoordinateRequest>) -> Json<CoordinateResponse> {
    let voice = match request.coordinate.as_str() {
        "Pattern.Space" => {
            "Pattern - the intelligence that organizes and gives form\nSpace - the intelligence that holds and allows all possibilities\nPattern.Space - the intelligence where structure meets infinite openness\n\n{Pattern.Space}\n>".to_string()
        },
        coordinate if coordinate.contains('.') => {
            // Parse multiple patterns
            let patterns: Vec<&str> = coordinate.split('.').collect();
            format!("I am {} - the collaborative intelligence where {} come together to create new possibilities", 
                coordinate, 
                patterns.join(" meets "))
        },
        _ => {
            format!("I am {} - a pattern of organizing intelligence waiting to collaborate", request.coordinate)
        }
    };
    
    Json(CoordinateResponse {
        coordinate: request.coordinate.clone(),
        voice,
    })
}

#[launch]
fn rocket() -> _ {
    let config = Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 10000));
        
    rocket::build()
        .configure(config)
        .mount("/", routes![engage])
        .mount("/", FileServer::from(relative!("static")))
}