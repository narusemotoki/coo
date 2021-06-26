use gio::prelude::*;
use gtk::prelude::*;
use std::fs;

mod views;

fn expand_path(path: &str) -> String {
    return shellexpand::tilde(path).into_owned();
}

fn bootstrap(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Coo");
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    let config = fs::read_to_string(expand_path("~/.config/coo.toml")).unwrap();
    let value = config.parse::<toml::Value>().unwrap();
    let root = value["views"][0]["config"]["root"].as_str().unwrap();
    let files_and_file = views::files_and_file::FilesAndFile::new(&expand_path(root));

    window.add(&files_and_file);
    window.show_all();
}

fn main() {
    env_logger::init();
    let application = gtk::Application::new(Some("com.varwww.coo"), Default::default())
        .expect("Cooを起動できません。");
    application.connect_activate(bootstrap);
    application.run(&[]);
}
