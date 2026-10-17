#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
use eframe::egui;
fn main() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/icon.png"))
        .expect("The bundled application icon must be a valid PNG");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SoftOxide Calculator")
            .with_icon(icon)
            .with_inner_size([440.0, 600.0])
            .with_min_inner_size([400.0, 600.0])
            .with_app_id("com.softoxide.calculator"),
        ..Default::default()
    };
    eframe::run_native(
        "SoftOxide Calculator",
        options,
        Box::new(|cc| Ok(Box::new(app::Desktop::new(cc)))),
    )
}

#[cfg(test)]
mod icon_tests {
    #[test]
    fn embedded_icon_has_expected_size_and_transparency() {
        let icon =
            eframe::icon_data::from_png_bytes(include_bytes!("../../assets/icon.png")).unwrap();
        assert_eq!((icon.width, icon.height), (512, 512));
        assert_eq!(icon.rgba.len(), 512 * 512 * 4);
        assert_eq!(icon.rgba[3], 0, "Icon corners must be transparent");
        assert!(icon
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[3] == 255));
    }
    #[test]
    fn platform_icon_containers_are_complete() {
        let ico = include_bytes!("../../assets/icon.ico");
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 7, 0]);
        let mut previous_end = 6 + 7 * 16;
        for index in 0..7 {
            let entry = &ico[6 + 16 * index..6 + 16 * (index + 1)];
            let size = u32::from_le_bytes(entry[8..12].try_into().unwrap()) as usize;
            let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
            assert_eq!(offset, previous_end);
            let png = &ico[offset..offset + size];
            assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
            let icon = eframe::icon_data::from_png_bytes(png).unwrap();
            let dimension = if entry[0] == 0 { 256 } else { entry[0] as u32 };
            assert_eq!((icon.width, icon.height), (dimension, dimension));
            previous_end = offset + size;
        }
        assert_eq!(previous_end, ico.len());
        let icns = include_bytes!("../../assets/icon.icns");
        assert_eq!(&icns[..4], b"icns");
        assert_eq!(
            u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize,
            icns.len()
        );
        let mut offset = 8;
        let mut count = 0;
        while offset < icns.len() {
            let size =
                u32::from_be_bytes(icns[offset + 4..offset + 8].try_into().unwrap()) as usize;
            assert!(size > 8);
            let png = &icns[offset + 8..offset + size];
            eframe::icon_data::from_png_bytes(png).unwrap();
            offset += size;
            count += 1;
        }
        assert_eq!(offset, icns.len());
        assert_eq!(count, 11);
    }
}
