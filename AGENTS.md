@/home/daniel/.codex/RTK.md

# Repository Guidelines

## Project Layout

- `src/*`: sources
- `data/`: desktop launcher and application icon.
- `scripts/check-deps.sh`: local dependency check.
- `.github/workflows/`: CI and nightly release automation.

## Build and Run

```bash
./scripts/check-deps.sh
make build
make test
make run
cargo run -- --box-url http://dreambox.local
```

Run `make build` after code changes. Run `make test` after changes that touch tested helpers or behavior, and use failing tests to guide the fix before finishing.

## Style

Use the existing Rust style: `rustfmt`, `snake_case` names, small focused functions, and GTK/libmpv helpers where already used. Keep changes in the matching module: Enigma2 HTTP logic in `src/enigma2/`, settings UI in `src/settings/`, shared player controls in `src/common/player/`, and channel overlay behavior in `src/ui/channel_overlay.rs`.

## Testing

Automated tests live in `tests/` and are wired through Cargo. Use `make test` to run them.

Manually smoke test startup, receiver configuration, bouquet loading, channel switching, stream playback, EPG text decoding, overlay scrolling, fullscreen, window dragging, resizing, mute, and volume scrolling when the change affects runtime UI or playback behavior.
