# Thought scaffolding and publishing.
#   just thought new       scaffold a post (interactive or with args)
#   just thought publish   commit + push pending thought changes

default:
    @just --list thought

# Scaffold a new thoughts post (page module + mod + registry; date = today).
# No args → interactive prompts; or pass slug / title / teaser.
new slug="" title="" teaser="":
    bash {{justfile_directory()}}/scripts/thought-new.sh {{quote(slug)}} {{quote(title)}} {{quote(teaser)}}

# Commit and push pending changes under thoughts + posts registry.
publish:
    bash {{justfile_directory()}}/scripts/thought-publish.sh
