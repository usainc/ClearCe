use super::errors::{err, ErrorCode, ProcessingError, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Queued,
    Preparing,
    Processing,
    Completed,
    Failed,
    Cancelled,
}
impl TaskStatus {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
    pub fn can_transition(self, next: Self) -> bool {
        !self.terminal()
            && (matches!(
                (self, next),
                (Self::Queued, Self::Preparing)
                    | (Self::Preparing, Self::Processing)
                    | (Self::Processing, Self::Completed)
            ) || matches!(next, Self::Failed | Self::Cancelled))
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    PNG,
    JPG,
    WEBP,
}
impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::PNG => "png",
            Self::JPG => "jpg",
            Self::WEBP => "webp",
        }
    }
    pub fn image_format(self) -> image::ImageFormat {
        match self {
            Self::PNG => image::ImageFormat::Png,
            Self::JPG => image::ImageFormat::Jpeg,
            Self::WEBP => image::ImageFormat::WebP,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessRequest {
    pub input_path: String,
    pub output_dir: Option<String>,
    pub scale: u32,
    pub format: OutputFormat,
    pub mode: String,
    pub gpu_index: Option<u32>,
    #[serde(default)]
    pub gpu_id: Option<String>,
    #[serde(default)]
    pub tile_size: u32,
    #[serde(default = "default_model")]
    pub model_id: String,
    #[serde(default = "default_engine_mode")]
    pub engine_mode: String,
}
fn default_model() -> String {
    "auto".into()
}
fn default_engine_mode() -> String {
    "Auto".into()
}
impl ProcessRequest {
    pub fn validate(&self) -> Result<()> {
        if ![2, 4, 8, 12].contains(&self.scale) {
            return Err(err(ErrorCode::UnsupportedScale));
        }
        if !matches!(self.mode.as_str(), "Photo" | "Anime / Illustration") {
            return Err(err(ErrorCode::UnsupportedMode));
        }
        if !matches!(
            self.model_id.as_str(),
            "auto" | "realesrgan-x4plus" | "realesrgan-x4plus-anime"
        ) {
            return Err(err(ErrorCode::UnsupportedMode));
        }
        if !matches!(self.engine_mode.as_str(), "Auto" | "Manual") {
            return Err(err(ErrorCode::EngineInvalid));
        }
        if self.gpu_index.is_some_and(|v| v > 31) {
            return Err(err(ErrorCode::InvalidGpuSelection));
        }
        if ![0, 32, 64, 128, 256].contains(&self.tile_size) {
            return Err(err(ErrorCode::UnsafeTargetResolution));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub path: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(default)]
    pub stages: Vec<PipelineStage>,
    #[serde(default)]
    pub activity: Vec<Activity>,
    #[serde(default)]
    pub target: Option<super::safety::TargetEstimate>,
    #[serde(default)]
    pub selected_gpu: String,
    pub id: String,
    pub request: ProcessRequest,
    pub status: TaskStatus,
    pub phase: String,
    #[serde(default)]
    pub phase_code: PhaseCode,
    pub progress: Option<u8>,
    pub input: Option<ImageInfo>,
    pub output: Option<ImageInfo>,
    pub engine_id: String,
    pub engine_version: Option<String>,
    pub model: String,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub error: Option<ProcessingError>,
    pub warning: Option<String>,
}
impl Task {
    pub fn queued(request: ProcessRequest) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            request,
            status: TaskStatus::Queued,
            phase: "Waiting".into(),
            phase_code: PhaseCode::Waiting,
            progress: None,
            input: None,
            output: None,
            engine_id: String::new(),
            engine_version: None,
            model: String::new(),
            created_at: now_ms(),
            started_at: None,
            completed_at: None,
            error: None,
            warning: None,
            stages: vec![],
            activity: vec![],
            target: None,
            selected_gpu: String::new(),
        }
    }
    pub fn record(&mut self, phase: &str) {
        self.phase = phase.into();
        self.phase_code = match self.status {
            TaskStatus::Completed => PhaseCode::Completed,
            TaskStatus::Failed => PhaseCode::Failed,
            TaskStatus::Cancelled => PhaseCode::Cancelled,
            _ if phase.starts_with("Pass ") => PhaseCode::Inference,
            _ if phase.starts_with("Selected GPU") => PhaseCode::Device,
            _ if phase.contains("Checking") => PhaseCode::Checking,
            _ if phase.contains("Validat") => PhaseCode::Validating,
            _ if phase.contains("output") => PhaseCode::Finalizing,
            _ => PhaseCode::Preparing,
        };
        if self.activity.last().is_none_or(|e| e.message != phase) {
            self.activity.push(Activity {
                at: now_ms(),
                message: phase.into(),
                code: self.phase_code,
            });
            if self.activity.len() > 64 {
                self.activity.remove(0);
            }
        }
    }
    pub fn transition(&mut self, status: TaskStatus, phase: &str) -> Result<()> {
        if !self.status.can_transition(status) {
            return Err(err(ErrorCode::IoError));
        }
        self.status = status;
        self.record(phase);
        self.progress = if status == TaskStatus::Completed {
            Some(100)
        } else {
            None
        };
        if status == TaskStatus::Preparing {
            self.started_at = Some(now_ms());
        }
        if status.terminal() {
            self.completed_at = Some(now_ms());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(default)]
    pub code: PhaseCode,
    pub at: u64,
    pub message: String,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseCode {
    #[default]
    Recorded,
    Waiting,
    Preparing,
    Checking,
    Inference,
    Device,
    Finalizing,
    Validating,
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStage {
    pub label: String,
    pub scale: u32,
    pub status: TaskStatus,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transitions_are_explicit_and_terminal_states_do_not_restart() {
        assert!(TaskStatus::Queued.can_transition(TaskStatus::Preparing));
        assert!(!TaskStatus::Queued.can_transition(TaskStatus::Completed));
        for s in [
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Cancelled,
        ] {
            assert!(!s.can_transition(TaskStatus::Processing));
        }
        assert!(TaskStatus::Processing.can_transition(TaskStatus::Cancelled));
    }
    #[test]
    fn reject_unavailable_scale_mode_and_format() {
        let mut r = ProcessRequest {
            input_path: "a.png".into(),
            output_dir: None,
            scale: 7,
            format: OutputFormat::PNG,
            mode: "Photo".into(),
            gpu_index: None,
            gpu_id: None,
            tile_size: 0,
            model_id: "auto".into(),
            engine_mode: "Auto".into(),
        };
        assert_eq!(r.validate().unwrap_err().code, ErrorCode::UnsupportedScale);
        r.scale = 2;
        assert!(r.validate().is_ok());
        r.mode = "Portrait".into();
        assert_eq!(r.validate().unwrap_err().code, ErrorCode::UnsupportedMode);
        assert!(serde_json::from_str::<OutputFormat>("\"GIF\"").is_err());
    }
}
