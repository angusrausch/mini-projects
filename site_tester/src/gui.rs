use eframe::egui;
use site_tester::{get_client, normalise_url, make_requests, get_average};
use std::sync::Arc;

pub struct SiteTesterApp {
    url: String,
    force: bool,
    number: u32,
    processes: u32,
    ignore_ssl: bool,
    timeout: f32,
    verbose: bool,
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
        }
    }
}

impl eframe::App for SiteTesterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Site Tester GUI");
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

            if ui.button("Start").clicked() {
                self.url = normalise_url(self.url.clone(), self.force);
                let url = Arc::new(self.url.clone());
                let timeout: u16 = (self.timeout * 1000.0) as u16;

                let client = get_client(timeout, self.ignore_ssl);

                let times = make_requests(
                    url,
                    self.number,
                    self.processes,
                    Arc::clone(&client),
                    self.verbose,
                );

                    let (average_value, fails, max_time) = get_average(&times);
                println!(
                    "Completed a total of {number_requests} requests, \
                    with an average time of {:?} for successful requests and \
                    {fails} failed requests and a max request time of {:?}",
                    average_value,
                    max_time,
                    number_requests = self.number
                );
            }
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