//! The site's canonical route list, derived from the page registries. The
//! 404's suggestion index and the snapshot manifest (`bens-site --routes`)
//! both read this, so a page that exists here but not in the router — or
//! vice versa — is a bug in exactly one place.

use crate::content::{interests::INTERESTS, posts::POSTS};

/// Every real route on the site, fixed pages first.
pub fn site_routes() -> Vec<String> {
    let mut routes = vec![
        "/".to_string(),
        "/thoughts".to_string(),
        "/resume".to_string(),
        "/interests".to_string(),
    ];
    routes.extend(POSTS.iter().map(|post| format!("/thoughts/{}", post.slug)));
    routes.extend(INTERESTS.iter().map(|i| format!("/interests/{}", i.slug)));
    routes
}

/// A route's snapshot file stem: "/" is "home"; any other path drops the
/// leading slash and dashes the rest ("/thoughts/pesky-code" →
/// "thoughts-pesky-code").
pub fn route_name(route: &str) -> String {
    if route == "/" {
        "home".to_string()
    } else {
        route.trim_start_matches('/').replace('/', "-")
    }
}
