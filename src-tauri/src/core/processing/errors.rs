use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCode {
    UnsafeTargetResolution,
    InsufficientDiskSpace,
    PipelinePassFailed,
    QueueItemNotFound,
    DuplicateQueueItem,
    InterruptedJob,
    InvalidGpuSelection,
    QueueLimitExceeded,
    EngineNotFound,
    EngineInvalid,
    UnsupportedFormat,
    UnsupportedScale,
    UnsupportedMode,
    InvalidImage,
    ImageTooLarge,
    OutputNotWritable,
    ProcessLaunchFailed,
    InferenceFailed,
    OutputMissing,
    DimensionMismatch,
    Cancelled,
    IoError,
    Busy,
    TaskNotFound,
    GpuUnavailable,
    ContainmentUnavailable,
    TimedOut,
    StorageError,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingError {
    pub code: ErrorCode,
    pub message: String,
}
impl ProcessingError {
    pub fn new(code: ErrorCode) -> Self {
        let message = match code {
            ErrorCode::UnsafeTargetResolution => "The requested target or intermediate image exceeds safe pixel or memory limits. Choose a smaller image or scale.",
            ErrorCode::InsufficientDiskSpace => "There is not enough free disk space for intermediate images and the final output.",
            ErrorCode::PipelinePassFailed => "An AI pipeline pass failed. No final output was accepted.",
            ErrorCode::QueueItemNotFound => "This queue item is no longer available.",
            ErrorCode::DuplicateQueueItem => "This image is already in the queue.",
            ErrorCode::InterruptedJob => "The previous session ended during processing. Retry this item to start a new attempt.",
            ErrorCode::InvalidGpuSelection => "The selected GPU is no longer available. Select Auto or a detected device in Settings.",
            ErrorCode::QueueLimitExceeded => "The queue is limited to 200 items. Clear finished items before adding more.",
            ErrorCode::EngineNotFound => "Real-ESRGAN engine is not installed. Open Models for setup instructions.",
            ErrorCode::EngineInvalid => "The engine installation is incomplete or invalid. Reinstall the verified engine package.",
            ErrorCode::UnsupportedFormat => "Select a JPG, PNG, or WebP image and supported output format.",
            ErrorCode::UnsupportedScale => "Choose a supported scale: 2x, 4x, 8x or 12x.",
            ErrorCode::UnsupportedMode => "This engine supports Photo mode only. Other modes arrive in a later phase.",
            ErrorCode::InvalidImage => "The image cannot be read or decoded. Select a valid local image.",
            ErrorCode::ImageTooLarge => "This image exceeds the Phase 2 limit of 100 MB or 6 million pixels.",
            ErrorCode::OutputNotWritable => "The output folder cannot be written to. Choose an existing writable folder.",
            ErrorCode::ProcessLaunchFailed => "The local enhancement process could not start.",
            ErrorCode::InferenceFailed => "Local AI enhancement failed. Check the engine installation and Vulkan driver.",
            ErrorCode::OutputMissing => "The engine finished without producing a readable image.",
            ErrorCode::DimensionMismatch => "The output dimensions do not match the requested scale. No result was accepted.",
            ErrorCode::Cancelled => "Enhancement cancelled. The original image was not changed.",
            ErrorCode::Busy => "Another enhancement is active. Wait for it or cancel it first.",
            ErrorCode::TaskNotFound => "This task is no longer available.",
            ErrorCode::GpuUnavailable => "No Vulkan GPU is available. Install a compatible graphics driver.",
            ErrorCode::ContainmentUnavailable => "Windows process containment could not be established. Processing was not started.",
            ErrorCode::TimedOut => "The local engine exceeded its time limit and was stopped.",
            ErrorCode::StorageError => "The local job history could not be saved. Check available disk space.",
            ErrorCode::IoError => "A local file operation failed. Check the file and folder permissions.",
        };
        Self {
            code,
            message: message.into(),
        }
    }
}
impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for ProcessingError {}
pub type Result<T> = std::result::Result<T, ProcessingError>;
pub fn err(code: ErrorCode) -> ProcessingError {
    ProcessingError::new(code)
}
