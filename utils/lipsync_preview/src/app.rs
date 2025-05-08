use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::prelude::*;
use std::path::PathBuf;

use pikaxe::{Platform, SystemInfo};
use pikaxe::audio::open_wav;
use pikaxe::io::*;
use pikaxe::scene::{Anim, CharBoneSample, Object, ObjectDir, PackedObject, MeshAnim, MiloObject, Trans, Vector3};

use eframe::{egui::{self, Align, Align2, Color32, FontId, Pos2, RichText, Visuals, Widget}, glow};
use egui::widgets::global_dark_light_mode_switch;

use shared::*;

#[derive(Default)]
pub struct LipsyncApp {
    args: Vec<String>,
    weight_points: Vec<f64>,
    sample_points: Vec<Vec<f64>>,
    selected_channel: usize,
}

impl LipsyncApp {
    pub fn load_args(&mut self, args: &Vec<String>) -> Result<(), Box<dyn Error>> {
        let weights_data = {
            let weights_path = PathBuf::from(&args[0]);

            let mut weights_file = std::fs::OpenOptions::new()
                .read(true)
                .open(&weights_path)?;

            let mut buff = Vec::new();
            weights_file.read_to_end(&mut buff)?;

            buff
        };

        let (sample_rate, wav_channels) = {
            let wav_path = PathBuf::from(&args[1]);
            open_wav(&wav_path)?
        };

        let channel_idx = args[2].parse::<usize>().unwrap_or(1);

        self.selected_channel = channel_idx - 1;

        self.weight_points = weights_data
            .into_iter()
            .map(|w| w as f64 / 255.)
            .collect();

        self.sample_points = wav_channels
            .into_iter()
            .map(|sams| sams
                .into_iter()
                .step_by(sample_rate as usize / 60)
                .map(|s| (s as f64 / i16::MAX as f64).abs())
                /*.chunks(sample_rate as usize / 60)
                .map(|sc| sc
                    .iter()
                    .map(|s| (*s as f64 / i16::MAX as f64).abs())
                    .sum::<f64>() / sc.len() as f64)
                    //.max_by(|a, b| a.total_cmp(b)).unwrap_or_default())*/
                .collect())
            .collect();

        /*for (i, w) in weights_data.into_iter().enumerate() {
            let value = w as f64 / 255.;

            let r = w;
            let g = 255 - r;

            /*MsgSender::new(format!("weights"))
                .with_time(Timeline::new("milliseconds", TimeType::Time), Time::from_seconds_since_epoch(i as f64 / 60.))
                .with_splat(Scalar(w as f64 / 255.))?
                .with_splat(ColorRGBA::from_rgb(r, g, 0))?
                .send(&mut session)?;*/
        }*/

        /*for (c_idx, channel) in wav_channels.into_iter().enumerate() {
            if c_idx != channel_idx {
                continue;
            }

            for (s_idx, sample) in channel.into_iter().enumerate() {
                let normalized = sample as f64 / i32::MAX as f64;
                let time_s = s_idx as f64 / sample_rate as f64;

                /*MsgSender::new(format!("audio_channel_{c_idx}"))
                    .with_time(Timeline::new("milliseconds", TimeType::Time), Time::from_seconds_since_epoch(time_s))
                    .with_splat(Scalar(normalized))?
                    .with_splat(ColorRGBA::from_rgb(0, 0, 255 - (20 * c_idx as u8)))?
                    .send(&mut session)?;*/
            }
        }*/

        Ok(())
    }
}

impl eframe::App for LipsyncApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::containers::Frame::default()
            .inner_margin(egui::Vec2::ZERO);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let x_grid_spacer = egui_plot::uniform_grid_spacer(|_g| [60. * 60., 60., 20.]);
            let y_grid_spacer = egui_plot::uniform_grid_spacer(|_g| [0.5, 0.1, 0.05]);

            egui_plot::Plot::new("main_plot")
                .center_y_axis(false)
                .min_size(egui::Vec2::new(0., 1.))
                .x_grid_spacer(x_grid_spacer)
                .y_grid_spacer(y_grid_spacer)
                //.set_margin_fraction(egui::Vec2::ZERO)
                .show(ui, |plot_ui| {
                    // Draw audio samples
                    if let Some(channel_samples) = self.sample_points.get(self.selected_channel) {
                        let sample_points = convert_to_egui_points(channel_samples, 1, false);
                        //plot_ui.points(sample_points.name("audio").color(Color32::BLUE));

                        plot_ui.line(egui_plot::Line::new(sample_points).name("audio").color(Color32::BLUE));
                    }

                    // Draw weights
                    let weight_points = convert_to_egui_points(&self.weight_points, 1, false);
                    //plot_ui.points(egui::plot::Points::new(weight_points).name("weights").color(Color32::GREEN));

                    plot_ui.line(egui_plot::Line::new(weight_points).name("weights").color(Color32::GREEN));
                });
        });
    }
}

fn convert_to_egui_points(points: &Vec<f64>, step: usize, hide_zero: bool) -> egui_plot::PlotPoints {
    let series = points
        .iter()
        .enumerate()
        .map(|(i, p)| [i as f64, *p])
        .filter(|[_, y]| !hide_zero || y.ne(&0.0))
        .step_by(step)
        .collect::<Vec<_>>();

    series.into()
}