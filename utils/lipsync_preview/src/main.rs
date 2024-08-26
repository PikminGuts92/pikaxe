use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::prelude::*;
use std::path::PathBuf;

mod app;
use app::*;

use eframe::{NativeOptions, run_native};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();

    if args.len() < 1 {
        println!("lipsync_preview.exe [input_weights_path]");
        return Ok(());
    }

    let mut app = LipsyncApp::default();
    app.load_args(&args)?;

    let ops = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_min_inner_size([320.0, 240.0])
            .with_drag_and_drop(true),
        // icon_data: Some(icon),
        // min_window_size: Some([1000., 600.].into()),
        follow_system_theme: false, // Always dark by default
        default_theme: eframe::Theme::Dark,
        ..NativeOptions::default()
    };

    run_native(
        "Lipsync Preview",
        ops,
        Box::new(|_cc| Ok(Box::new(app)))
    )
    .map_err(|e| e.into())
}
