use ash::{vk, Entry};
use std::ffi::CStr;
#[derive(Clone)]
pub struct Identity {
    pub name: String,
    pub id: String,
    pub vendor: String,
    pub device_type: String,
    pub dedicated_memory_bytes: Option<u64>,
}
pub fn devices() -> Vec<String> {
    identities().into_iter().map(|d| d.name).collect()
}
pub fn recommendation_devices() -> Vec<crate::core::processing::engine::GpuDevice> {
    identities()
        .into_iter()
        .enumerate()
        .map(
            |(index, identity)| crate::core::processing::engine::GpuDevice {
                id: identity.id,
                index: index as u32,
                name: identity.name,
                vendor: identity.vendor,
                device_type: identity.device_type,
                dedicated_memory_bytes: identity.dedicated_memory_bytes,
            },
        )
        .collect()
}
pub fn identities() -> Vec<Identity> {
    // Query the Vulkan loader itself. Dedicated memory is reported only for a
    // discrete device; shared iGPU heaps are intentionally not called VRAM.
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
                let memory = instance.get_physical_device_memory_properties(device);
                let dedicated_memory_bytes =
                    (p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU).then(|| {
                        memory.memory_heaps[..memory.memory_heap_count as usize]
                            .iter()
                            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
                            .map(|heap| heap.size)
                            .sum::<u64>()
                    });
                let vendor = match p.vendor_id {
                    0x10de => "NVIDIA",
                    0x1002 | 0x1022 => "AMD",
                    0x8086 => "Intel",
                    _ => "Other",
                }
                .to_string();
                let device_type = match p.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => "discrete",
                    vk::PhysicalDeviceType::INTEGRATED_GPU => "integrated",
                    vk::PhysicalDeviceType::VIRTUAL_GPU => "virtual",
                    _ => "other",
                }
                .to_string();
                Some(Identity {
                    name,
                    id: format!("{:x}:{:x}:{uuid}", p.vendor_id, p.device_id),
                    vendor,
                    device_type,
                    dedicated_memory_bytes,
                })
            })
            .collect();
        instance.destroy_instance(None);
        names
    }
}
pub fn map_ncnn(
    text: &str,
    identities: &[Identity],
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
        let matches: Vec<_> = identities.iter().filter(|d| d.name == name).collect();
        if matches.len() == 1
            && !devices
                .iter()
                .any(|d: &crate::core::processing::engine::GpuDevice| d.index == index)
        {
            devices.push(crate::core::processing::engine::GpuDevice {
                id: matches[0].id.clone(),
                index,
                name: name.into(),
                vendor: matches[0].vendor.clone(),
                device_type: matches[0].device_type.clone(),
                dedicated_memory_bytes: matches[0].dedicated_memory_bytes,
            });
        }
    }
    devices
}
