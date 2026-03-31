# demovid

AI-agent-driven demo video production. Rust CLI + Kokoro TTS. Fully local, Apple Silicon.

## Requirements

- macOS (Apple Silicon)
- [Rust](https://rustup.rs) (stable)
- ffmpeg: `brew install ffmpeg`
- cmake: `brew install cmake` (needed to build the TTS engine)

## Install

```sh
git clone https://github.com/yourorg/demovid
cd demovid
make install
```

This builds a release binary and copies it to `~/.local/bin/demovid`.

Make sure `~/.local/bin` is in your `PATH`:

```sh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

To install to a custom location:

```sh
make install PREFIX=/usr/local  # may require sudo on macOS
```

To uninstall:

```sh
make uninstall
```

## Usage

```sh
demovid --help
```
