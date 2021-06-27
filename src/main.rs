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

    let css = include_str!("resources/coo.css").replace("{font}", value["font"].as_str().unwrap());
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(css.as_bytes()).unwrap();
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("CSSプロバイダの初期化に失敗しました。"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

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
