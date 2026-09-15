use ash::{vk, Entry};
use std::ffi::CStr;
pub fn devices() -> Vec<String> {
    identities().into_iter().map(|(name, _)| name).collect()
}
pub fn identities() -> Vec<(String, String)> {
    // Query the Vulkan loader itself. No VRAM estimates or assumed NCNN device indices.
    unsafe {
        let Ok(entry) = Entry::load() else {
            return vec![];
        };
        let info = vk::InstanceCreateInfo::default();
        let Ok(instance) = entry.create_instance(&info, None) else {
            return vec![];
        };
        let names = instance
            .enumerate_physical_devices()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|device| {
                let p = instance.get_physical_device_properties(device);
                if p.device_type == vk::PhysicalDeviceType::CPU {
                    return None;
                }
                let name = CStr::from_ptr(p.device_name.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                let uuid = p
                    .pipeline_cache_uuid
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>();
                Some((name, format!("{:x}:{:x}:{uuid}", p.vendor_id, p.device_id)))
            })
            .collect();
        instance.destroy_instance(None);
        names
    }
}
pub fn map_ncnn(
    text: &str,
    identities: &[(String, String)],
) -> Vec<crate::core::processing::engine::GpuDevice> {
    let mut devices = vec![];
    for line in text.lines().filter(|line| line.contains("queueC=")) {
        let Some(header) = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']').map(|p| p.0))
        else {
            continue;
        };
        let Some((index, name)) = header.split_once(' ') else {
            continue;
        };
        let Ok(index) = index.parse::<u32>() else {
            continue;
        };
        let matches: Vec<_> = identities.iter().filter(|(n, _)| n == name).collect();
        if matches.len() == 1
            && !devices
                .iter()
                .any(|d: &crate::core::processing::engine::GpuDevice| d.index == index)
        {
            devices.push(crate::core::processing::engine::GpuDevice {
                id: matches[0].1.clone(),
                index,
                name: name.into(),
            });
        }
    }
    devices
}
