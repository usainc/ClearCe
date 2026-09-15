use super::errors::{err, ErrorCode, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct SafetyLimits {
    pub max_output_pixels: u64,
    pub max_intermediate_pixels: u64,
    pub max_working_bytes: u64,
    pub disk_margin_bytes: u64,
}
impl Default for SafetyLimits {
    fn default() -> Self {
        Self {
            max_output_pixels: 96_000_000,
            max_intermediate_pixels: 128_000_000,
            max_working_bytes: 2 * 1024 * 1024 * 1024,
            disk_margin_bytes: 128 * 1024 * 1024,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetEstimate {
    pub width: u32,
    pub height: u32,
    pub pixels: u64,
    pub intermediate_pixels: u64,
    pub working_bytes: u64,
    pub temporary_bytes: u64,
    pub final_bytes: u64,
}
impl SafetyLimits {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let value: Self = serde_json::from_reader(
            std::fs::File::open(path).map_err(|_| err(ErrorCode::IoError))?,
        )
        .map_err(|_| err(ErrorCode::UnsafeTargetResolution))?;
        // Configuration may tighten limits, never exceed decoder/architecture hard ceilings.
        if value.max_output_pixels == 0
            || value.max_output_pixels > 96_000_000
            || value.max_intermediate_pixels == 0
            || value.max_intermediate_pixels > 128_000_000
            || value.max_working_bytes == 0
            || value.max_working_bytes > 2 * 1024 * 1024 * 1024
            || value.disk_margin_bytes > 1024 * 1024 * 1024
        {
            return Err(err(ErrorCode::UnsafeTargetResolution));
        }
        Ok(value)
    }
    pub fn estimate(
        &self,
        width: u32,
        height: u32,
        scale: u32,
        native: u32,
    ) -> Result<TargetEstimate> {
        let unsafe_target = || err(ErrorCode::UnsafeTargetResolution);
        if ![2, 4, 8, 12].contains(&scale) || native != 4 {
            return Err(unsafe_target());
        }
        let w = width.checked_mul(scale).ok_or_else(unsafe_target)?;
        let h = height.checked_mul(scale).ok_or_else(unsafe_target)?;
        let input = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or_else(unsafe_target)?;
        let pixels = u64::from(w) * u64::from(h);
        let intermediate = input
            .checked_mul(if scale > 4 { 256 } else { 16 })
            .ok_or_else(unsafe_target)?;
        let working = intermediate
            .checked_mul(12)
            .and_then(|v| v.checked_add(pixels * 8))
            .and_then(|v| v.checked_add(input * 4))
            .ok_or_else(unsafe_target)?;
        if input == 0
            || pixels > self.max_output_pixels
            || intermediate > self.max_intermediate_pixels
            || working > self.max_working_bytes
        {
            let mut e = unsafe_target();
            e.message=format!("{scale}x would create {w} × {h} pixels and exceed the safe output, intermediate or working-memory limit.");
            return Err(e);
        }
        Ok(TargetEstimate {
            width: w,
            height: h,
            pixels,
            intermediate_pixels: intermediate,
            working_bytes: working,
            temporary_bytes: intermediate * 5 + input * 85,
            final_bytes: pixels * 5,
        })
    }
    pub fn check_disk(&self, estimate: &TargetEstimate, temp: &Path, output: &Path) -> Result<()> {
        let needed = estimate.temporary_bytes + estimate.final_bytes + self.disk_margin_bytes;
        for path in [temp, output] {
            if free_space(path).is_some_and(|bytes| bytes < needed) {
                return Err(err(ErrorCode::InsufficientDiskSpace));
            }
        }
        Ok(())
    }
}
#[cfg(windows)]
fn free_space(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut free = 0;
    unsafe {
        if windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) == 0
        {
            None
        } else {
            Some(free)
        }
    }
}
#[cfg(not(windows))]
fn free_space(_: &Path) -> Option<u64> {
    None
}
