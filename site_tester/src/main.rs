mod gui;
mod cli;

fn main() {
    let cli_flag = std::env::args().any(|arg| arg == "--cli");

    if !cli_flag {
        gui::run_gui().unwrap();
        return;
    } else {
        cli::run_cli();
    }
}
