use anyhow::Result;
use std::path::Path;

pub fn run(
    _text: &str,
    _output: &Path,
    _voice: &str,
    _speed: f64,
    _provider: &str,
    _json_mode: bool,
) -> Result<()> {
    anyhow::bail!("tts command not yet implemented (planned for M2)")
}
