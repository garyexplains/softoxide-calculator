#[cfg(target_os = "windows")]
pub const DESCRIPTION: &str = "wgpu / Direct3D 12 (hardware preferred, software allowed)";
#[cfg(not(target_os = "windows"))]
pub const DESCRIPTION: &str = "Glow / OpenGL";

#[cfg(any(target_os = "windows", test))]
pub fn windows_configuration() -> eframe::egui_wgpu::WgpuConfiguration {
    use eframe::{egui_wgpu, wgpu};
    use std::sync::Arc;

    let mut setup = egui_wgpu::WgpuSetupCreateNew::default();
    // Avoid depending on OpenGL or third-party Vulkan drivers on a fresh Windows VM.
    setup.instance_descriptor.backends = wgpu::Backends::DX12;
    setup.native_adapter_selector = Some(Arc::new(|adapters, surface| {
        adapters
            .iter()
            .filter(|adapter| surface.is_none_or(|s| adapter.is_surface_supported(s)))
            // Keep CPU adapters (including Windows WARP) as a fallback.
            .min_by_key(|adapter| match adapter.get_info().device_type {
                wgpu::DeviceType::DiscreteGpu => 0,
                wgpu::DeviceType::IntegratedGpu => 1,
                wgpu::DeviceType::VirtualGpu => 2,
                wgpu::DeviceType::Other => 3,
                wgpu::DeviceType::Cpu => 4,
            })
            .cloned()
            .ok_or_else(|| {
                "No compatible Direct3D 12 hardware or software adapter was found".into()
            })
    }));
    egui_wgpu::WgpuConfiguration {
        wgpu_setup: setup.into(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn windows_renderer_uses_direct3d_with_adapter_selection() {
        let config = super::windows_configuration();
        let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = config.wgpu_setup else {
            panic!("Windows must create a new graphics instance");
        };
        assert_eq!(
            setup.instance_descriptor.backends,
            eframe::wgpu::Backends::DX12
        );
        assert!(setup.native_adapter_selector.is_some());
        let error = setup.native_adapter_selector.unwrap()(&[], None).unwrap_err();
        assert!(error.contains("Direct3D 12"));
    }
}
