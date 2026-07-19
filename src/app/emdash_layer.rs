//! Response layer: turn em dashes in HTML page bodies into `/llms` links.

use topcoat::{
    Result,
    context::CxBuilder,
    router::{Body, Next, Response, header, layer, to_bytes},
};

use crate::emdash::link_em_dashes;

#[layer("/")]
async fn emdash_links(cx: &mut CxBuilder, body: Body, next: Next<'_>) -> Result<Response> {
    let response = next.run(cx, body).await?;
    if !is_html(response.headers().get(header::CONTENT_TYPE)) {
        return Ok(response);
    }

    let (mut parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.map_err(|err| {
        std::io::Error::other(format!("emdash layer: failed to read body: {err}"))
    })?;
    let html = String::from_utf8_lossy(&bytes);
    let rewritten = link_em_dashes(&html);
    parts.headers.remove(header::CONTENT_LENGTH);
    Ok(Response::from_parts(parts, Body::from(rewritten)))
}

fn is_html(content_type: Option<&header::HeaderValue>) -> bool {
    match content_type {
        None => true,
        Some(value) => value
            .to_str()
            .ok()
            .and_then(|s| s.split(';').next())
            .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/html")),
    }
}
