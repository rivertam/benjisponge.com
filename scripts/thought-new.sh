#!/usr/bin/env bash
# Scaffold a new thoughts post (pesky_code-shaped): page module, mod decl, registry entry.
#
# Usage:
#   just thought new                          # interactive prompts
#   just thought new <slug>                   # prompt for title/teaser
#   just thought new <slug> "<title>" ["<teaser>"]
#   bash scripts/thought-new.sh …
#
# Slug is the URL segment (kebab-case). Date is today (local). Posts are
# inserted newest-first in src/content/posts.rs.

set -Eeuo pipefail

usage() {
    printf 'usage: just thought new [<slug> ["<title>" ["<teaser>"]]]\n' >&2
    printf '  no args  prompt for slug, title, teaser\n' >&2
    printf '  slug     kebab-case URL segment, e.g. pesky-code\n' >&2
    printf '  title    display title, e.g. "Pesky code"\n' >&2
    printf '  teaser   index-card blurb (default: "TODO")\n' >&2
    exit 2
}

[[ $# -le 3 ]] || usage

slug="${1-}"
title="${2-}"
teaser="${3-}"
did_prompt=0

slug_to_title() {
    local s="${1//-/ }"
    printf '%s%s' "$(printf '%s' "${s:0:1}" | tr '[:lower:]' '[:upper:]')" "${s:1}"
}

prompt() {
    # prompt NAME DEFAULT -> writes answer to $REPLY (empty → default)
    local name="$1" default="${2-}" hint
    if [[ -n "$default" ]]; then
        hint=" [$default]"
    else
        hint=""
    fi
    printf '%s%s: ' "$name" "$hint" >&2
    if ! IFS= read -r REPLY; then
        printf '\nerror: unexpected EOF while reading %s\n' "$name" >&2
        exit 1
    fi
    if [[ -z "$REPLY" ]]; then
        REPLY="$default"
    fi
    did_prompt=1
}

if [[ -z "$slug" ]]; then
    while true; do
        prompt "slug"
        slug="$REPLY"
        if [[ -z "$slug" ]]; then
            printf '  slug is required\n' >&2
            continue
        fi
        if [[ "$slug" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
            break
        fi
        printf '  slug must be kebab-case ([a-z0-9]+(-[a-z0-9]+)*)\n' >&2
    done
elif [[ ! "$slug" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
    printf 'error: slug must be kebab-case ([a-z0-9]+(-[a-z0-9]+)*)\n' >&2
    exit 1
fi

if [[ -z "$title" ]]; then
    prompt "title" "$(slug_to_title "$slug")"
    title="$REPLY"
    if [[ -z "$title" ]]; then
        printf 'error: title is required\n' >&2
        exit 1
    fi
fi

if [[ -z "$teaser" ]]; then
    # Prompt when interactive, or when earlier fields were prompted (piped
    # full-interactive answers include a teaser line). Quiet default otherwise.
    if [[ -t 0 || "$did_prompt" -eq 1 ]]; then
        prompt "teaser" "TODO"
        teaser="$REPLY"
    else
        teaser="TODO"
    fi
fi
[[ -n "$teaser" ]] || teaser="TODO"

mod_name="${slug//-/_}"
date="$(date +%Y-%m-%d)"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
page_path="${repo_root}/src/app/thoughts/${mod_name}.rs"
thoughts_mod="${repo_root}/src/app/thoughts.rs"
posts_rs="${repo_root}/src/content/posts.rs"

if [[ -e "$page_path" ]]; then
    printf 'error: %s already exists\n' "$page_path" >&2
    exit 1
fi

if grep -qE "^pub mod ${mod_name};" "$thoughts_mod"; then
    printf 'error: mod %s already declared in thoughts.rs\n' "$mod_name" >&2
    exit 1
fi

if grep -qE "slug: \"${slug}\"" "$posts_rs"; then
    printf 'error: slug "%s" already in posts.rs\n' "$slug" >&2
    exit 1
fi

# Confirm when we prompted for anything.
if [[ "$did_prompt" -eq 1 ]]; then
    printf '\nCreate /thoughts/%s (%s)?\n' "$slug" "$date" >&2
    printf '  title:  %s\n' "$title" >&2
    printf '  teaser: %s\n' "$teaser" >&2
    prompt "proceed?" "Y"
    case "$REPLY" in
        Y|y|yes|YES) ;;
        *)
            printf 'aborted\n' >&2
            exit 1
            ;;
    esac
fi

# Write the page module + registry entry (Python so $/"/\ in titles stay literal).
python3 - "$page_path" "$posts_rs" "$slug" "$mod_name" "$title" "$date" "$teaser" <<'PY'
import re, sys
from pathlib import Path

page_path, posts_rs = Path(sys.argv[1]), Path(sys.argv[2])
slug, mod_name, title, date, teaser = sys.argv[3:8]

def rust_str(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')

title_esc, teaser_esc = rust_str(title), rust_str(teaser)

page_path.write_text(
    f'''use topcoat::{{Result, router::page, view::view}};

use crate::components::shell;

#[page("/thoughts/{slug}")]
async fn {mod_name}() -> Result {{
    view! {{
        shell(
            title: "{title_esc}",
            active: "",
            <article class="rail-row mt-16 sm:mt-24">
                <p class="rail-stamp">"{date}"</p>
                <div class="min-w-0">
                    <h1 class="font-display text-4xl font-bold tracking-tight">
                        "{title_esc}"
                    </h1>
                    <p class="mt-8 max-w-prose text-xl leading-relaxed">
                        "TODO"
                    </p>
                </div>
            </article>
        )
    }}
}}
'''
)

text = posts_rs.read_text()
m = re.search(r"pub static POSTS: \[Post; (\d+)\] = \[", text)
if not m:
    sys.exit("error: could not find POSTS array in posts.rs")
n = int(m.group(1))
text = text[: m.start(1)] + str(n + 1) + text[m.end(1) :]

entry = (
    f"    Post {{\n"
    f'        slug: "{slug}",\n'
    f'        title: "{title_esc}",\n'
    f'        date: "{date}",\n'
    f'        teaser: "{teaser_esc}",\n'
    f"    }},\n"
)
m2 = re.search(r"pub static POSTS: \[Post; \d+\] = \[", text)
if not m2:
    sys.exit("error: could not re-find POSTS array after bumping count")
insert_at = m2.end()
rest = text[insert_at:].lstrip("\n")
posts_rs.write_text(text[:insert_at] + "\n" + entry + rest)
PY

# Insert `pub mod …;` in alphabetical order among existing pub mod lines.
insert_before="$(
    grep -nE '^pub mod [a-z0-9_]+;' "$thoughts_mod" \
        | while IFS=: read -r lineno decl; do
            name="${decl#pub mod }"
            name="${name%;}"
            if [[ "$name" > "$mod_name" ]]; then
                printf '%s\n' "$lineno"
                break
            fi
        done
)"
tmp="$(mktemp)"
if [[ -n "$insert_before" ]]; then
    {
        head -n "$((insert_before - 1))" "$thoughts_mod"
        printf 'pub mod %s;\n' "$mod_name"
        tail -n +"$insert_before" "$thoughts_mod"
    } >"$tmp"
else
    last_mod_line="$(grep -nE '^pub mod [a-z0-9_]+;' "$thoughts_mod" | tail -1 | cut -d: -f1)"
    if [[ -z "$last_mod_line" ]]; then
        printf 'error: no pub mod lines found in thoughts.rs\n' >&2
        exit 1
    fi
    {
        head -n "$last_mod_line" "$thoughts_mod"
        printf 'pub mod %s;\n' "$mod_name"
        tail -n +"$((last_mod_line + 1))" "$thoughts_mod"
    } >"$tmp"
fi
mv "$tmp" "$thoughts_mod"

printf 'created %s\n' "src/app/thoughts/${mod_name}.rs"
printf 'wired   mod %s in src/app/thoughts.rs\n' "$mod_name"
printf 'indexed /thoughts/%s (%s) in src/content/posts.rs\n' "$slug" "$date"
printf 'edit the body in src/app/thoughts/%s.rs\n' "$mod_name"
