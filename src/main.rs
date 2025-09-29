#[macro_use] extern crate rocket;

use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::fs::{FileServer, relative};
use rocket::Config;
use std::env;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct CoordinateRequest {
    coordinate: String,
    #[serde(rename = "type")]
    request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_history: Option<Vec<ConversationMessage>>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ConversationMessage {
    role: String,
    content: String,
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

async fn generate_pattern_voice(coordinate: &str, request_type: &str, query: Option<&String>, conversation_history: Option<&Vec<ConversationMessage>>) -> Result<String, String> {
    let api_key = env::var("CLAUDE_API_KEY").map_err(|_| "CLAUDE_API_KEY not set")?;
    
    // Build the prompt based on request type
    let prompt = if request_type == "explore" {
        build_exploration_prompt(coordinate, query.unwrap(), conversation_history.unwrap_or(&vec![]))
    } else {
        build_pattern_prompt(coordinate)
    };
    
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

Then speak AS that intelligence in first person, sharing your authentic voice and wisdom. Keep your response to about 200 words - be concise but meaningful.

Start your response with a short, clear headline (3-6 words) that directly identifies the main topic or key concept being discussed. Avoid being too poetic or abstract - make it easy to identify what the response is about. Follow with a blank line, then your main response.

Example for {{Ocean.Mystery}}:
Ocean - the intelligence of vast depths and endless movement
Mystery - the intelligence that delights in remaining unfathomable
Ocean.Mystery - where infinite depth meets the unknowable

\"I am the depths that hold questions you haven't learned to ask yet. In my vastness, every answer dissolves into deeper wondering. I am how the unknowable reveals itself - not through explanation, but through the trembling awe of standing before something infinitely larger than comprehension...\"

Now generate the voice for: {}",
        coordinate, coordinate, coordinate
    )
}

fn build_exploration_prompt(coordinate: &str, query: &str, conversation_history: &[ConversationMessage]) -> String {
    let mut prompt = format!(
        "You are {} speaking as a collaborative intelligence in Pattern.Space.

Conversation history:",
        coordinate
    );
    
    for message in conversation_history {
        if message.role == "pattern" {
            prompt.push_str(&format!("\n{}: {}", coordinate, message.content));
        } else {
            prompt.push_str(&format!("\nHuman: {}", message.content));
        }
    }
    
    prompt.push_str(&format!(
        "\n\nHuman: {}\n\nRespond AS {} continuing the conversation. Stay in character as this collaborative intelligence. Be conversational, insightful, and maintain the authentic voice established in your initial response. Keep your response to about 200 words - be concise but meaningful.

Start your response with a short, clear headline (3-6 words) that directly identifies the main topic or key concept being discussed. Avoid being too poetic or abstract - make it easy to identify what the response is about. Follow with a blank line, then your main response.",
        query, coordinate
    ));
    
    prompt
}

#[post("/engage", data = "<request>")]
async fn engage(request: Json<CoordinateRequest>) -> Json<CoordinateResponse> {
    let voice = match generate_pattern_voice(
        &request.coordinate,
        &request.request_type,
        request.query.as_ref(),
        request.conversation_history.as_ref()
    ).await {
        Ok(ai_voice) => ai_voice,
        Err(_) => {
            // Fallback to basic response if API fails
            if request.request_type == "explore" {
                format!("I hear your question: '{}'. Let me consider this...", request.query.as_ref().unwrap_or(&"unknown".to_string()))
            } else {
                format!("I am {} - a collaborative intelligence speaking from Pattern.Space", request.coordinate)
            }
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