# Ben's Site

## Setup

The project uses [proto](https://moonrepo.dev/proto) to pin development tools.
Install the configured toolchain, then start the development server:

```sh
proto use
just dev
```

The site is available at <http://127.0.0.1:3000>. To use another port:

```sh
just dev 4610
```

## Commands

Run `just` or `just --list` to see the available commands.

```sh
just build
just release
just check
```
