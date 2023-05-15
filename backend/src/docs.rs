use crate::payloads;
use crate::routes;

use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(title = "Crab Fit API"),
    paths(
        routes::stats::get_stats,
        routes::event::create_event,
        routes::event::get_event,
        routes::person::get_people,
        routes::person::get_person,
        routes::person::update_person,
    ),
    components(schemas(
        payloads::StatsResponse,
        payloads::EventResponse,
        payloads::PersonResponse,
        payloads::EventInput,
        payloads::PersonInput,
    )),
    tags(
        (name = "info"),
        (name = "event"),
        (name = "person"),
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

// Add password auth spec
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().add_security_scheme(
            "password",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("base64")
                    .build(),
            ),
        );
    }
}
