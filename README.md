# Enigma2 Player

A lightweight GTK4/libmpv desktop player for Enigma2 receivers. It loads bouquets and EPG data, and puts bouquet switching, the channel list, and the current EPG into an on-player overlay.

<p align="center">
  <img src="data/local.enigma2-player.svg" alt="Enigma2 Player icon" width="46">
</p>

## Receiver

The receiver URL is stored in the user settings file:

```text
~/.config/enigma2-player/settings.json
```

The first start is intentionally empty. Open the Settings button in the top-left overlay and enter the Enigma2 receiver URL there.

For a one-off launch you can also pass a receiver URL on the command line:

```bash
cargo run -- --box-url http://enigma2.local
```

## Dependencies

On Ubuntu/Debian:

```bash
sudo apt install build-essential cargo rustc pkg-config libgtk-4-dev libmpv-dev libepoxy-dev mpv
```

Check the local machine:

```bash
./scripts/check-deps.sh
```

## Build

```bash
make build
```

This uses `cargo build --release` and strips the resulting binary.

Or build and copy the binary to `~/.local/bin`:

```bash
make install
```

`make install` only copies the binary to `~/.local/bin` and strips the installed copy.

## Run

```bash
make run
```

Or run the installed binary:

```bash
enigma2-player
```

Configure the receiver from Settings or pass `--box-url` for a temporary session.

## Test

```bash
make test
```

Manual smoke tests should cover receiver loading, bouquet switching, channel playback, EPG overlay scrolling, settings save/load, fullscreen, window dragging, resizing, mute, and volume scrolling.
