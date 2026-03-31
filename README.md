# demovid

AI-agent-driven demo video production. Rust CLI + Kokoro TTS. Fully local, Apple Silicon.

## Requirements

- macOS (Apple Silicon)
- [Rust](https://rustup.rs) (stable)
- ffmpeg: `brew install ffmpeg`

## Install

```sh
git clone https://github.com/yourorg/demovid
cd demovid
make install
```

This builds a release binary and copies it to `/usr/local/bin/demovid`.

To install to a custom location:

```sh
make install PREFIX=~/.local
```

To uninstall:

```sh
make uninstall
```

## Usage

```sh
demovid --help
```
