use super::errors::{err, ErrorCode, Result};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
#[derive(Clone, Default)]
pub struct Cancellation(pub Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn check(&self) -> Result<()> {
        if self.0.load(Ordering::SeqCst) {
            Err(err(ErrorCode::Cancelled))
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Availability {
    Available,
    Missing,
    Invalid,
    Unsupported,
    Error,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    #[serde(default)]
    pub devices: Vec<GpuDevice>,
    pub id: String,
    pub name: String,
    pub model: String,
    pub version: Option<String>,
    pub availability: Availability,
    pub message: String,
    pub location: Option<String>,
    pub install_dir: String,
    pub source: Option<String>,
    pub scales: Vec<u32>,
    pub formats: Vec<String>,
    pub gpu_devices: Vec<String>,
    pub gpu_selection: String,
}
pub struct EngineInput<'a> {
    pub tile_size: u32,
    pub input: &'a Path,
    pub output: &'a Path,
    pub gpu_index: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuDevice {
    pub id: String,
    pub index: u32,
    pub name: String,
}
pub trait EnhancementEngine: Send + Sync {
    fn status(&self) -> EngineStatus;
    fn native_scale(&self) -> u32;
    fn process(&self, input: EngineInput<'_>, cancel: &Cancellation) -> Result<()>;
    fn cancel(&self, token: &Cancellation) {
        token.cancel();
    }
}
