# Token-based Cloudflare Tunnel connector for Railway private networking.
# ENTRYPOINT is already `cloudflared --no-autoupdate`; override the image's
# default `version` CMD so the process stays up.
FROM cloudflare/cloudflared:latest
CMD ["tunnel", "run"]
