use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: u32,
    pub slug: String,
    pub step: String,
    pub outcome: String,
    pub scenario: String,
    pub voice: VoiceConfig,
    pub shots: Vec<Shot>,
    pub narrative: Vec<NarrativeSegment>,
    pub clips: Vec<Clip>,
    pub mapping: Vec<Mapping>,
    pub renders: Vec<Render>,
    pub r#final: FinalOutput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub provider: String,
    pub voice: String,
    pub speed: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shot {
    pub id: u32,
    pub label: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NarrativeSegment {
    pub id: u32,
    pub shot_id: u32,
    pub text: String,
    pub pause_after: f64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clip {
    pub shot_id: u32,
    pub file: String,
    pub duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub clip_id: u32,
    pub narration_ids: Vec<u32>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Render {
    pub clip_id: u32,
    pub audio_file: String,
    pub render_file: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinalOutput {
    pub file: String,
    pub status: String,
}
