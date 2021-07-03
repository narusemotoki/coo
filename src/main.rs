use gio::prelude::*;
use gtk::prelude::*;
use std::fs;

mod views;

struct View {
    widget: gtk::Widget,
    name: String,
    title: String,
}
struct ViewsIterator {
    index: usize,
    views: Vec<toml::Value>,
}

impl ViewsIterator {
    fn build_widget(&self, view_config: toml::Value) -> gtk::Widget {
        let root = view_config["config"]["root"].as_str().unwrap();
        match view_config["component"].as_str().unwrap() {
            "assorted_card" => views::assorted_card::View::new(&coo::libs::expand_path(root))
                .upcast::<gtk::Widget>(),
            "files_and_file" => {
                views::files_and_file::FilesAndFile::new(&coo::libs::expand_path(root))
                    .upcast::<gtk::Widget>()
            }
            _ => panic!(),
        }
    }

    fn new(config: &toml::Value) -> Self {
        Self {
            index: 0,
            views: config["views"].as_array().unwrap().to_owned(),
        }
    }
}

impl Iterator for ViewsIterator {
    type Item = View;

    fn next(&mut self) -> Option<Self::Item> {
        let view_config = self.views.get(self.index);
        self.index += 1;
        view_config.map(|config| View {
            widget: self.build_widget(config.clone()),
            name: uuid::Uuid::new_v4().to_string(),
            title: config["title"].as_str().unwrap().to_string(),
        })
    }
}

fn bootstrap(application: &gtk::Application) {
    let header_bar = gtk::HeaderBarBuilder::new()
        .title("Coo")
        .show_close_button(true)
        .build();

    let config = fs::read_to_string(coo::libs::expand_path("~/.config/coo.toml"))
        .unwrap()
        .parse::<toml::Value>()
        .unwrap();
    let stack = gtk::StackBuilder::new().expand(true).build();
    for view in ViewsIterator::new(&config) {
        stack.add_titled(&view.widget, &view.name, &view.title);
    }
    header_bar.add(&gtk::StackSwitcherBuilder::new().stack(&stack).build());

    let application_window = gtk::ApplicationWindowBuilder::new()
        .application(application)
        .title("Coo")
        .window_position(gtk::WindowPosition::Center)
        .default_width(1280)
        .default_height(720)
        .build();
    application_window.set_titlebar(Some(&header_bar));

    application_window.add(&stack);

    let css = include_str!("resources/coo.css").replace("{font}", config["font"].as_str().unwrap());
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(css.as_bytes()).unwrap();
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("CSSプロバイダの初期化に失敗しました。"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    application_window.show_all();
}

fn main() {
    env_logger::init();
    let application = gtk::Application::new(Some("com.varwww.coo"), Default::default())
        .expect("Cooを起動できません。");
    application.connect_activate(bootstrap);
    application.run(&[]);
}
