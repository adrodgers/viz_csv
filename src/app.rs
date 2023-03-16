use egui_file::{FileDialog, Filter};
use polars::prelude::*;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct VizCsvApp {
    opened_file: Option<PathBuf>,
    #[serde(skip)]
    open_file_dialog: Option<FileDialog>,
    #[serde(skip)]
    dataframe: Option<DataFrame>,
    selected_column: Option<String>,
}

impl Default for VizCsvApp {
    fn default() -> Self {
        Self {
            open_file_dialog: None,
            opened_file: None,
            dataframe: None,
            selected_column: None,
        }
    }
}

impl VizCsvApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for VizCsvApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            open_file_dialog,
            opened_file,
            dataframe,
            selected_column,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if (ui.button("Open")).clicked() {
                let mut dialog = FileDialog::open_file(opened_file.clone()).filter(filter());
                dialog.open();
                *open_file_dialog = Some(dialog);
            }

            if let Some(dialog) = open_file_dialog {
                if dialog.show(ctx).selected() {
                    *opened_file = dialog.path();
                    if let Ok(df) = file_to_df(&dialog.path().unwrap()) {
                        *dataframe = Some(df);
                    }
                }
            }
            // The central panel the region left after adding TopPanel's and SidePanel's
            if let Some(df) = dataframe {
                let column_names = df.get_column_names();
                egui::ComboBox::from_label("Select column!")
                    .selected_text(format!("{:?}", selected_column))
                    .show_ui(ui, |ui| {
                        for name in column_names {
                            ui.selectable_value(selected_column, Some(name.to_string()), name);
                        }
                    });
                egui::plot::Plot::new("data").show(ui, |plot_ui| {
                    // plot_ui.line(
                    if let Some(column_name) = selected_column {
                        let y: Vec<f64> = df
                            .column(&column_name)
                            .unwrap()
                            .f64()
                            .unwrap()
                            .into_iter()
                            .map(|val| val.unwrap())
                            .collect();

                        let line = egui::plot::Line::new(egui::plot::PlotPoints::from_ys_f64(&y));
                        plot_ui.line(line);
                    }
                });
            }
            egui::warn_if_debug_build(ui);
        });
    }
}

pub fn filter() -> Filter {
    Box::new(|path: &Path| -> bool {
        return path.extension() == Some(OsStr::new("txt"))
            || path.extension() == Some(OsStr::new("csv"));
    })
}

fn file_to_df(path: &PathBuf) -> PolarsResult<DataFrame> {
    CsvReader::from_path(path)?
        .has_header(false)
        .with_delimiter(' ' as u8)
        .finish()
}
