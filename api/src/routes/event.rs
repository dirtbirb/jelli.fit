use axum::{
    extract::{self, Path},
    http::StatusCode,
    Json,
};
use common::{Adaptor, Event};
use rand::{seq::SliceRandom, thread_rng, Rng};
use regex::Regex;

use crate::{
    errors::ApiError,
    payloads::{ApiResult, EventInput, EventResponse},
    State,
};

#[utoipa::path(
    get,
    path = "/event/{event_id}",
    params(
        ("event_id", description = "The ID of the event"),
    ),
    responses(
        (status = 200, description = "Ok", body = EventResponse),
        (status = 404, description = "Not found"),
        (status = 429, description = "Too many requests"),
    ),
    tag = "event",
)]
/// Get details about an event
pub async fn get_event<A: Adaptor>(
    extract::State(state): State<A>,
    Path(event_id): Path<String>,
) -> ApiResult<EventResponse, A> {
    let adaptor = &state.lock().await.adaptor;

    let event = adaptor
        .get_event(event_id)
        .await
        .map_err(ApiError::AdaptorError)?;

    match event {
        Some(event) => Ok(Json(event.into())),
        None => Err(ApiError::NotFound),
    }
}

#[utoipa::path(
    post,
    path = "/event",
    request_body(content = EventInput, description = "New event details"),
    responses(
        (status = 201, description = "Created", body = EventResponse),
        (status = 415, description = "Unsupported input format"),
        (status = 422, description = "Invalid input provided"),
        (status = 429, description = "Too many requests"),
    ),
    tag = "event",
)]
/// Create a new event
pub async fn create_event<A: Adaptor>(
    extract::State(state): State<A>,
    Json(input): Json<EventInput>,
) -> Result<(StatusCode, Json<EventResponse>), ApiError<A>> {
    let adaptor = &state.lock().await.adaptor;

    // Get the current timestamp
    let now = chrono::offset::Utc::now();

    // Generate a name if none provided
    let name = match input.name {
        Some(x) if !x.is_empty() => x.trim().to_string(),
        _ => generate_name(),
    };

    // Generate an ID
    let mut id = generate_id(&name);

    // Check the ID doesn't already exist
    while (adaptor
        .get_event(id.clone())
        .await
        .map_err(ApiError::AdaptorError)?)
    .is_some()
    {
        id = generate_id(&name);
    }

    let event = adaptor
        .create_event(Event {
            id,
            name,
            created_at: now,
            visited_at: now,
            times: input.times,
            timezone: input.timezone,
        })
        .await
        .map_err(ApiError::AdaptorError)?;

    // Update stats
    adaptor
        .increment_stat_event_count()
        .await
        .map_err(ApiError::AdaptorError)?;

    Ok((StatusCode::CREATED, Json(event.into())))
}

// Generate a random name based on an adjective and a jelly species
fn generate_name() -> String {
    let adjectives: Vec<String> =
        serde_json::from_slice(include_bytes!("../res/adjectives.json")).unwrap();
    let jellies: Vec<String> = serde_json::from_slice(include_bytes!("../res/jellies.json")).unwrap();

    format!(
        "{} {} Jelly",
        adjectives.choose(&mut thread_rng()).unwrap(),
        jellies.choose(&mut thread_rng()).unwrap()
    )
}

// Generate a slug for the jelli fit
fn generate_id(name: &str) -> String {
    let mut id = encode_name(name.to_string());
    if id.replace('-', "").is_empty() {
        id = encode_name(generate_name());
    }
    let number = thread_rng().gen_range(100000..=999999);
    format!("{}-{}", id, number)
}

// Use punycode to encode the name
fn encode_name(name: String) -> String {
    let pc = punycode::encode(&name.trim().to_lowercase())
        .unwrap_or(String::from(""))
        .trim()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "");
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(&pc, "-").to_string()
}
