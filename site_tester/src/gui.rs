use clap::builder::Str;
use eframe::egui;
use site_tester::{get_client, normalise_url, make_requests, get_average};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::VecDeque;
use std::thread;

const ASCII_BANNER: &str = r#"
     _____ _____ _______ ______ _______ ______  _____ _______ ______ _____     
    / ____|_   _|__   __|  ____|__   __|  ____|/ ____|__   __|  ____|  __ \     ____ _   _ ___
   | (___   | |    | |  | |__     | |  | |__  | (___    | |  | |__  | |__) |   / ___| | | |_ _|
    \___ \  | |    | |  |  __|    | |  |  __|  \___ \   | |  |  __| |  _  /   | |  _| | | || | 
    ____) |_| |_   | |  | |____   | |  | |____ ____) |  | |  | |____| | \ \   | |_| | |_| || | 
   |_____/|_____|  |_|  |______|  |_|  |______|_____/   |_|  |______|_|  \_\   \____|\___/|___|
"#;

const LOGS_MAX_CAPACITY: usize = 100000;

pub struct SiteTesterApp {
    url: String,
    force: bool,
    number: u32,
    processes: u32,
    ignore_ssl: bool,
    timeout: f32,
    verbose: bool,
    times: Arc<Mutex<Vec<u32>>>,
    running: bool,
    message: String,
    cancel_flag: Arc<AtomicBool>,
    logs: Arc<Mutex<VecDeque<String>>>,
}

impl Default for SiteTesterApp {
    fn default() -> Self {
        Self {
            url: String::from("https://"),
            force: false,
            number: 100,
            processes: 10,
            ignore_ssl: false,
            timeout: 10.0,
            verbose: false,
            times: Arc::new(Mutex::new(Vec::new())),
            running: false,
            message: String::new(),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(LOGS_MAX_CAPACITY))),
        }
    }
}

impl eframe::App for SiteTesterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(egui::RichText::new(ASCII_BANNER).monospace().strong());
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(&mut self.url);
                ui.checkbox(&mut self.force, "Force URL");
            });
            ui.horizontal(|ui| {
                ui.label("Number of requests:");
                ui.add(egui::DragValue::new(&mut self.number));
            });
            ui.horizontal(|ui| {
                ui.label("Concurrent processes:");
                ui.add(egui::DragValue::new(&mut self.processes));
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.ignore_ssl, "Ignore SSL");
                ui.checkbox(&mut self.verbose, "Verbose Output");
                ui.label("Timeout (seconds):");
                ui.add(egui::DragValue::new(&mut self.timeout));
            });

            ui.add_space(8.0);
            if !self.message.is_empty() {
                let lines: Vec<&str> = self.message.split('\n').collect();
                for (i, line) in lines.iter().enumerate() {
                    if line.contains("Completed a total") {
                        ui.label(
                            egui::RichText::new(*line)
                                .size(24.0)
                                .color(egui::Color32::LIGHT_GREEN)
                                .strong()
                        );
                    } else if line.contains("Average time") {
                        ui.label(
                            egui::RichText::new(*line)
                                .size(20.0)
                                .color(egui::Color32::LIGHT_BLUE)
                                .strong()
                        );
                    } else if line.contains("Maximum time") {
                        ui.label(
                            egui::RichText::new(*line)
                                .size(20.0)
                                .color(egui::Color32::LIGHT_YELLOW)
                                .strong()
                        );
                    } else if line.contains("failed") || line.contains("Failed") {
                        ui.label(
                            egui::RichText::new(*line)
                                .size(20.0)
                                .color(egui::Color32::RED)
                                .strong()
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(*line)
                                .size(20.0)
                                .color(egui::Color32::WHITE)
                        );
                    }
                }
            }
            ui.add_space(8.0);

            // // Debug info section
            // ui.separator();
            // ui.heading("Debug Info");
            // ui.label(format!("URL: {}", self.url));
            // ui.label(format!("Force URL: {}", self.force));
            // ui.label(format!("Number of requests: {}", self.number));
            // ui.label(format!("Processes: {}", self.processes));
            // ui.label(format!("Ignore SSL: {}", self.ignore_ssl));
            // ui.label(format!("Timeout (seconds): {}", self.timeout));
            // ui.label(format!("Verbose: {}", self.verbose));
            // ui.label(format!("Running: {}", self.running));
            // {
            //     let t = self.times.lock().unwrap();
            //     ui.label(format!("Times vector length: {}", t.len()));
            //     ui.label(format!("Completed requests: {}", t.iter().filter(|&&x| x != 0).count()));
            //     // Show first few times for inspection
            //     ui.label(format!("First 5 times: {:?}", t.iter().take(5).collect::<Vec<_>>()));
            // }

            if ui.button("Start").clicked() {
                self.cancel_flag.store(true, Ordering::SeqCst);

                // Reset state for new run
                self.times = Arc::new(Mutex::new(vec![0; self.number as usize]));
                self.cancel_flag = Arc::new(AtomicBool::new(false));
                self.running = true; // <-- Ensure running is set to true on start

                {
                    let mut logs_guard = self.logs.lock().unwrap();
                    logs_guard.clear();
                }
                self.url = normalise_url(self.url.clone(), self.force);
                let url = Arc::new(self.url.clone());
                let timeout: u16 = (self.timeout * 1000.0) as u16;

                let client = get_client(timeout, self.ignore_ssl);
                let number = self.number;
                let processes = self.processes;
                let verbose = self.verbose;
                let logs = Arc::clone(&self.logs);

                let ok_closure = {
                    let logs = Arc::clone(&logs);
                    move |msg: String| {
                        let mut logs = logs.lock().unwrap();
                        if logs.len() == LOGS_MAX_CAPACITY {
                            logs.pop_front();
                        }
                        logs.push_back(format!("OUTPUT: {}", msg));
                    }
                };
                let err_closure = {
                    let logs = Arc::clone(&logs);
                    move |msg: String| {
                        let mut logs = logs.lock().unwrap();
                        if logs.len() == LOGS_MAX_CAPACITY {
                            logs.pop_front();
                        }
                        logs.push_back(format!("ERROR: {}", msg));
                    }
                };

                make_requests(
                    url,
                    number,
                    processes,
                    client,
                    verbose,
                    (ok_closure, err_closure),
                    Arc::clone(&self.times),
                    Arc::clone(&self.cancel_flag),
                );
            }

            let times_len = {
                let t = self.times.lock().unwrap();
                t.iter().filter(|&&x| x != 0).count()
            };
            ui.label(format!("{}/{}", times_len, self.number));

            if self.running {
                ctx.request_repaint();
            }

            // Fix: Only set self.message and self.running when all requests are done
            if self.running && times_len >= self.number as usize {
                let times = self.times.lock().unwrap();
                let (average_value, fails, max_time) = get_average(&times);
                self.message = format!(
                    "Completed a total of {} requests \n\
                    Average time of successful requests: {:?} \n\
                    Maximum time of successful request: {:?} \n\
                    Total failed requests: {}",
                    self.number, average_value, max_time, fails
                );
                self.running = false;
            }

            ui.separator();
            ui.heading("Logs");
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let logs_guard = self.logs.lock().unwrap();
                    for log in logs_guard.iter() {
                        ui.label(log);
                    }
                });
        });
    }
}

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Site Tester GUI",
        options,
        Box::new(|_cc| Box::new(SiteTesterApp::default())),
    )
}