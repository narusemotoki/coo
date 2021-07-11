use gio::prelude::*;
use gio::ApplicationFlags;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell;
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

const APPLICATION_NAME: &str = "Coo";

impl ViewsIterator {
    fn build_widget(&self, view_config: toml::Value) -> gtk::Widget {
        let root = view_config["config"]["root"].as_str().unwrap();
        match view_config["component"].as_str().unwrap() {
            "assorted_card" => views::assorted_card::View::new(&coo::libs::expand_path(root))
                .upcast::<gtk::Widget>(),
            "files_and_file" => views::files_and_file::View::new(&coo::libs::expand_path(root))
                .upcast::<gtk::Widget>(),
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

fn bootstrap(application: &Coo, config_file_path: String) {
    let header_bar = gtk::HeaderBarBuilder::new()
        .title(APPLICATION_NAME)
        .show_close_button(true)
        .build();

    let config = fs::read_to_string(config_file_path)
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
        .title(APPLICATION_NAME)
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
        &gdk::Screen::default().expect("CSSプロバイダの初期化に失敗しました。"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    application_window.show_all();
}

fn main() {
    env_logger::init();

    let application = Coo::new();
    application.add_main_option(
        "config",
        glib::char::Char::from(b'c'),
        glib::OptionFlags::IN_MAIN,
        glib::OptionArg::String,
        "設定ファイルを指定します。",
        None,
    );
    application.run();
}

#[derive(Debug)]
pub struct CooExt {
    config_file_path: cell::RefCell<String>,
}

impl Default for CooExt {
    fn default() -> Self {
        Self {
            config_file_path: cell::RefCell::new(coo::libs::expand_path("~/.config/coo.toml")),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CooExt {
    const NAME: &'static str = APPLICATION_NAME;
    type Type = Coo;
    type ParentType = gtk::Application;
}

impl ObjectImpl for CooExt {}

impl ApplicationImpl for CooExt {
    fn activate(&self, application: &Self::Type) {
        self.parent_activate(application);

        let coo = application.downcast_ref::<Coo>().unwrap();
        bootstrap(coo, self.config_file_path.borrow().clone());
    }

    fn handle_local_options(&self, application: &Self::Type, options: &glib::VariantDict) -> i32 {
        if let Some(variant) = options.lookup_value("config", None) {
            self.config_file_path
                .replace(coo::libs::expand_path(&variant.get::<String>().unwrap()));
        }
        self.parent_handle_local_options(application, options)
    }
}

impl GtkApplicationImpl for CooExt {}

glib::wrapper! {
    pub struct Coo(ObjectSubclass<CooExt>)
        @extends gio::Application, gtk::Application;
}

impl Coo {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &"com.varwww.coo"),
            ("flags", &ApplicationFlags::empty()),
        ])
        .expect("Cooの起動に失敗しました。")
    }
}
