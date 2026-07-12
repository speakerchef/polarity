use crate::{generators::PostFx, labeled_enum};

labeled_enum!(OsciWindowSize {
    R512 => "512",
    R1024 => "1024",
    R2048 => "2048",
    R4096 => "4096",
}, R1024);

labeled_enum!(OscilloscopeKind {
    Waveform => "Waveform",
    CircularWaveform => "Circular Waveform" 
}, Waveform);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Oscilloscope {
    pub kind: OscilloscopeKind,
    pub window_sz: OsciWindowSize,
    pub efx: PostFx,
}
