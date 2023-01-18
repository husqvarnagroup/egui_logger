use std::sync::Mutex;

use egui::Color32;
use log::SetLoggerError;

const LEVELS: [log::Level; log::Level::Trace as usize] = [
    log::Level::Error,
    log::Level::Warn,
    log::Level::Info,
    log::Level::Debug,
    log::Level::Trace,
];

struct EguiLogger;

impl log::Log for EguiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::STATIC_MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            //println!("{}: {}", record.level(), record.args());
            let mut log = LOG.lock().unwrap();

            let mut l: Vec<(log::Level, String)> = log.clone();
            l.push((record.level(), record.args().to_string()));

            *log = l;
        }
    }

    fn flush(&self) {}
}

/// Initilizes the global logger.
/// Should be called very early in the program
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&EguiLogger).map(|()| log::set_max_level(log::LevelFilter::Info))
}

static LOG: Mutex<Vec<(log::Level, String)>> = Mutex::new(Vec::new());

static LOGGER_UI: once_cell::sync::Lazy<Mutex<LoggerUi>> =
    once_cell::sync::Lazy::new(Default::default);

struct LoggerUi {
    loglevels: [bool;log::Level::Trace as usize],
    search_term: String,
    copy_text: String,
    max_log_length: usize,
}

impl Default for LoggerUi {
    fn default() -> Self {
        // Error, Warn and Info
        Self {
            loglevels: [true, true, true, false, false],
            search_term: String::new(),
            copy_text: String::new(),
            max_log_length: 1000,
        }
    }
}

impl LoggerUi {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut logs = LOG.lock().unwrap();

        logs.reverse();
        logs.truncate(self.max_log_length);
        logs.reverse();

        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                *logs = vec![];
            }
            ui.menu_button("Log Levels", |ui| {
              for level in LEVELS
              {
                  if ui.selectable_label( self.loglevels[level as usize - 1], level.as_str()).clicked()
                  {
                      self.loglevels[level as usize - 1] = !self.loglevels[level as usize - 1];
                  }
              }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Search: ");
            ui.text_edit_singleline(&mut self.search_term);
        });

        ui.horizontal(|ui| {
            ui.label("Max Log output");
            ui.add(egui::widgets::DragValue::new(&mut self.max_log_length).speed(1));
        });

        ui.horizontal(|ui| {
            if ui.button("Sort").clicked() {
                logs.sort()
            }
        });
        ui.separator();

        let mut logs_displayed: usize = 0;

        egui::ScrollArea::vertical()
            .auto_shrink([true; 2])
            .max_height(ui.available_height() - 30.0)
            .show(ui, |ui| {
                logs.iter().for_each(|(level, string)| {
                    let string_format = format!("[{}]: {}", level, string);

                    if !self.search_term.is_empty() && !string.contains(&self.search_term) {
                        return;
                    }

                    if !(self.loglevels[*level as usize - 1]) {
                        return;
                    }

                    match level {
                        log::Level::Warn => {
                            ui.colored_label(Color32::YELLOW, string_format);
                        }
                        log::Level::Error => {
                            ui.colored_label(Color32::RED, string_format);
                        }
                        _ => {
                            ui.label(string_format);
                        }
                    }

                    self.copy_text += &format!("{string} \n").to_string();
                    logs_displayed += 1;
                });
            });

        ui.horizontal(|ui| {
            ui.label(format!("Log size: {}", logs.len()));
            ui.label(format!("Displayed: {}", logs_displayed));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Copy").clicked() {
                    ui.output().copied_text = self.copy_text.to_string();
                }
            });
        });

        // has to be cleared after every frame
        self.copy_text.clear();
    }
}

/// Draws the logger ui
/// has to be called after [`init()`](init());
pub fn logger_ui(ui: &mut egui::Ui) {
    let mut logger_ui = LOGGER_UI.lock().unwrap();

    logger_ui.ui(ui);
}
