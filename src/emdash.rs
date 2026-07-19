//! Post-process HTML so every em dash in page content becomes a link to `/llms`.
//!
//! Full documents: only text under `<main>`. Shard fragments (no `<main>`): the
//! whole body. Tags and text inside `a` / `script` / `style` / `textarea` /
//! `svg` are left alone.

const EM_DASH: char = '\u{2014}';
const LINK: &str =
    "<a href=\"/llms\" class=\"emdash-link\" title=\"about LLMs on this site\">\u{2014}</a>";

/// Tags whose text content must not get em-dash links (nested `<a>`, scripts).
const SKIP_TAGS: &[&str] = &["a", "script", "style", "textarea", "svg"];

/// Wrap each eligible em dash in `html` with a link to the LLMs disclosure page.
pub fn link_em_dashes(html: &str) -> String {
    match main_bounds(html) {
        Some((start, end)) => {
            let mut out = String::with_capacity(html.len() + 64);
            out.push_str(&html[..start]);
            rewrite_region(&html[start..end], &mut out);
            out.push_str(&html[end..]);
            out
        }
        None => {
            let mut out = String::with_capacity(html.len() + 64);
            rewrite_region(html, &mut out);
            out
        }
    }
}

/// Byte range of the first `<main…>` … `</main>` pair, including both tags.
fn main_bounds(html: &str) -> Option<(usize, usize)> {
    let bytes = html.as_bytes();
    let open = find_open_tag(bytes, "main")?;
    let after_open = skip_tag(bytes, open)?;
    let close = find_close_tag(bytes, after_open, "main")?;
    let end = skip_tag(bytes, close)?;
    Some((open, end))
}

fn rewrite_region(region: &str, out: &mut String) {
    let bytes = region.as_bytes();
    let mut i = 0;
    let mut skip_depth: usize = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            let tag_end = skip_tag(bytes, i).unwrap_or(bytes.len());
            if let Some(kind) = classify_tag(&bytes[i..tag_end]) {
                match kind {
                    TagKind::Open(name) if is_skip_tag(name) => skip_depth += 1,
                    TagKind::Close(name) if is_skip_tag(name) => {
                        skip_depth = skip_depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }
            out.push_str(&region[i..tag_end]);
            i = tag_end;
            continue;
        }

        // Text node: scan for em dashes when not inside a skip tag.
        let text_start = i;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }
        let text = &region[text_start..i];
        if skip_depth == 0 {
            push_linked_text(text, out);
        } else {
            out.push_str(text);
        }
    }
}

fn push_linked_text(text: &str, out: &mut String) {
    for ch in text.chars() {
        if ch == EM_DASH {
            out.push_str(LINK);
        } else {
            out.push(ch);
        }
    }
}

fn is_skip_tag(name: &str) -> bool {
    SKIP_TAGS.iter().any(|t| name.eq_ignore_ascii_case(t))
}

enum TagKind<'a> {
    Open(&'a str),
    Close(&'a str),
    Other,
}

/// Classify `<…>` at the start of `tag` (includes `<` and ideally `>`).
fn classify_tag(tag: &[u8]) -> Option<TagKind<'_>> {
    if tag.first() != Some(&b'<') {
        return None;
    }
    let mut i = 1;
    // Comments / doctype / processing instructions: leave alone.
    if matches!(tag.get(i), Some(b'!') | Some(b'?')) {
        return Some(TagKind::Other);
    }
    let closing = tag.get(i) == Some(&b'/');
    if closing {
        i += 1;
    }
    let name_start = i;
    while i < tag.len() && tag[i].is_ascii_alphanumeric() {
        i += 1;
    }
    if i == name_start {
        return Some(TagKind::Other);
    }
    let name = std::str::from_utf8(&tag[name_start..i]).ok()?;
    // Self-closing skip tags (rare) must not bump depth.
    let self_closing = tag[..tag.len().saturating_sub(1)]
        .iter()
        .rev()
        .find(|&&b| !b.is_ascii_whitespace())
        == Some(&b'/');
    if closing {
        Some(TagKind::Close(name))
    } else if self_closing {
        Some(TagKind::Other)
    } else {
        Some(TagKind::Open(name))
    }
}

/// Index of `<name` (case-insensitive) as an open tag, or `None`.
fn find_open_tag(bytes: &[u8], name: &str) -> Option<usize> {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = skip_tag(bytes, i)?;
            if let Some(TagKind::Open(n)) = classify_tag(&bytes[i..end])
                && n.eq_ignore_ascii_case(name)
            {
                return Some(i);
            }
            i = end;
        } else {
            i += 1;
        }
    }
    None
}

fn find_close_tag(bytes: &[u8], from: usize, name: &str) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = skip_tag(bytes, i)?;
            if let Some(TagKind::Close(n)) = classify_tag(&bytes[i..end])
                && n.eq_ignore_ascii_case(name)
            {
                return Some(i);
            }
            i = end;
        } else {
            i += 1;
        }
    }
    None
}

/// End index just past `>` of the tag starting at `start`, respecting quotes.
fn skip_tag(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'<') {
        return None;
    }
    let mut i = start + 1;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
        } else {
            match b {
                b'"' | b'\'' => quote = Some(b),
                b'>' => return Some(i + 1),
                _ => {}
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_em_dash_in_main_prose() {
        let html = "<html><head><title>x — y</title></head><body>\
                    <main><p>hello — world</p></main></body></html>";
        let out = link_em_dashes(html);
        assert!(out.contains("hello <a href=\"/llms\" class=\"emdash-link\""));
        assert!(out.contains("title=\"about LLMs on this site\">—</a> world"));
        // Title outside main stays plain.
        assert!(out.contains("<title>x — y</title>"));
    }

    #[test]
    fn leaves_attribute_em_dashes_alone() {
        let html = "<main><img alt=\"a — b\" src=\"x\"><p>c — d</p></main>";
        let out = link_em_dashes(html);
        assert!(out.contains("alt=\"a — b\""));
        assert!(out.contains("c <a href=\"/llms\""));
    }

    #[test]
    fn skips_text_inside_existing_anchors() {
        let html = "<main><a href=\"/x\">keep — plain</a><p>link — me</p></main>";
        let out = link_em_dashes(html);
        assert!(out.contains("<a href=\"/x\">keep — plain</a>"));
        assert!(out.contains("link <a href=\"/llms\""));
    }

    #[test]
    fn links_multiple_dashes() {
        let html = "<main>one — two — three</main>";
        let out = link_em_dashes(html);
        assert_eq!(out.matches("class=\"emdash-link\"").count(), 2);
    }

    #[test]
    fn self_link_page_still_gets_links() {
        let html = "<main><p>I use LLMs — for drafts</p></main>";
        let out = link_em_dashes(html);
        assert!(out.contains("href=\"/llms\""));
        assert!(out.contains("LLMs <a href=\"/llms\""));
    }

    #[test]
    fn shard_fragment_without_main_is_rewritten() {
        let html = "<div>tip — value</div>";
        let out = link_em_dashes(html);
        assert!(out.contains("tip <a href=\"/llms\""));
    }

    #[test]
    fn header_and_footer_outside_main_untouched() {
        let html = "<header>nav — here</header><main>body — text</main><footer>foot — er</footer>";
        let out = link_em_dashes(html);
        assert!(out.contains("<header>nav — here</header>"));
        assert!(out.contains("<footer>foot — er</footer>"));
        assert!(out.contains("body <a href=\"/llms\""));
    }
}
