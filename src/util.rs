//! Small shared helpers with no better home.

/// Percent-encode a URL query value: everything outside the unreserved set
/// (RFC 3986) becomes `%XX`, so arbitrary tag/tech names round-trip.
pub fn urlencode(raw: &str) -> String {
    let mut encoded = String::new();
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
