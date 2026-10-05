use std::sync::Arc;

use tauri::http::header::{CACHE_CONTROL, CONTENT_TYPE, HeaderValue};
use tauri::http::{Request, Response, StatusCode};
use tauri::{Manager, UriSchemeContext, UriSchemeResponder, Wry};

use crate::harness::Harness;

fn image_response(bytes: Option<Vec<u8>>) -> Response<Vec<u8>> {
    let Some(bytes) = bytes else {
        let mut response = Response::new(Vec::new());

        *response.status_mut() = StatusCode::NOT_FOUND;

        return response;
    };

    let mut response = Response::new(bytes);

    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("image/webp"));

    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));

    response
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri requires owned protocol callback arguments"
)]
pub fn respond(
    context: UriSchemeContext<'_, Wry>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = context.app_handle().clone();

    let allowed = context.webview_label() == "main" && request.method() == "GET";

    let handle = request.uri().path().trim_start_matches('/').to_owned();

    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Arc<Harness>>();

        let bytes = match allowed {
            true => state
                .resource_registry
                .get(&handle)
                .ok()
                .and_then(|grant| {
                    state
                        .resource
                        .display(
                            &grant.reference,
                            grant.comic_id,
                            grant.page_id,
                            grant.request,
                            &state.image,
                        )
                        .ok()
                })
                .or_else(|| {
                    state
                        .image_task
                        .preview(&handle, &state.resource, &state.image)
                        .ok()
                }),
            false => None,
        };

        responder.respond(image_response(bytes));
    });
}
