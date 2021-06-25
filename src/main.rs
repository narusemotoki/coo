use gio::prelude::*;
use gtk::prelude::*;

mod views;

fn bootstrap(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Coo");
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    let files_and_file = views::files_and_file::FilesAndFile::new(".");

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
