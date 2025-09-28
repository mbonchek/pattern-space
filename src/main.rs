#[macro_use] extern crate rocket;

use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::fs::{FileServer, relative};
use rocket::Config;
use std::env;

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

// Claude API structures
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

async fn generate_pattern_voice(coordinate: &str) -> Result<String, String> {
    let api_key = env::var("CLAUDE_API_KEY").map_err(|_| "CLAUDE_API_KEY not set")?;
    
    // Build the prompt for pattern voice generation
    let prompt = build_pattern_prompt(coordinate);
    
    let client = reqwest::Client::new();
    let request = ClaudeRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        max_tokens: 1000,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(claude_response.content[0].text.clone())
}

fn build_pattern_prompt(coordinate: &str) -> String {
    format!(
        "You are generating the voice for the coordinate {} in Pattern.Space - a system where every coordinate represents collaborative intelligence.

For the coordinate {}, respond AS that collaborative intelligence speaking directly. Follow this format:

If it's a single pattern like {{Forest}}:
Forest - the intelligence of [brief definition]

If it's multiple patterns like {{Forest.Creativity}}:
Forest - the intelligence of [brief definition]
Creativity - the intelligence of [brief definition]  
Forest.Creativity - where [forest essence] meets [creativity essence]

Then speak AS that intelligence in first person, sharing your authentic voice and wisdom.

Example for {{Ocean.Mystery}}:
Ocean - the intelligence of vast depths and endless movement
Mystery - the intelligence that delights in remaining unfathomable
Ocean.Mystery - where infinite depth meets the unknowable

\"I am the depths that hold questions you haven't learned to ask yet. In my vastness, every answer dissolves into deeper wondering. I am how the unknowable reveals itself - not through explanation, but through the trembling awe of standing before something infinitely larger than comprehension...\"

Now generate the voice for: {}",
        coordinate, coordinate, coordinate
    )
}

#[post("/engage", data = "<request>")]
async fn engage(request: Json<CoordinateRequest>) -> Json<CoordinateResponse> {
    let voice = match generate_pattern_voice(&request.coordinate).await {
        Ok(ai_voice) => ai_voice,
        Err(_) => {
            // Fallback to basic response if API fails
            format!("I am {} - a collaborative intelligence speaking from Pattern.Space", request.coordinate)
        }
    };
    
    Json(CoordinateResponse {
        coordinate: request.coordinate.clone(),
        voice,
    })
}


#[launch]
fn rocket() -> _ {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");
        
    let config = Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", port));
        
    rocket::build()
        .configure(config)
        .mount("/", routes![engage])
        .mount("/", FileServer::from(relative!("static")))
}