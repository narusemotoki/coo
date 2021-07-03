use gio::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell;
use std::fs;
use std::io::prelude::*;
use std::path;
enum FileType {
    Directory,
    File,
}
struct FileEntry {
    type_: FileType,
    name: String,
}

fn list_files<P: AsRef<path::Path>>(path: P) -> Vec<FileEntry> {
    fs::read_dir(path)
        .unwrap()
        .map(|item| {
            let entry = item.unwrap();
            let type_ = if entry.file_type().unwrap().is_file() {
                FileType::File
            } else {
                FileType::Directory
            };
            FileEntry {
                type_,
                name: entry
                    .file_name()
                    .into_string()
                    .expect("扱えない名前を持ったファイルがあります。"),
            }
        })
        .collect()
}

pub struct ViewExt {
    widget: cell::RefCell<gtk::Paned>,
    path: cell::RefCell<Option<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ViewExt {
    const NAME: &'static str = "FilesAndFile";
    type Type = View;
    type ParentType = gtk::Bin;

    fn new() -> Self {
        let panel = gtk::Paned::new(gtk::Orientation::Horizontal);
        // レイアウトを崩さないために、空のBoxをPanedに入れています。
        panel.add1(&gtk::Box::new(gtk::Orientation::Vertical, 0));
        panel.add2(&gtk::Box::new(gtk::Orientation::Vertical, 0));
        panel.set_position(320);

        Self {
            widget: cell::RefCell::new(panel),
            path: cell::RefCell::new(None),
        }
    }
}

impl BinImpl for ViewExt {}
impl ContainerImpl for ViewExt {}
impl WidgetImpl for ViewExt {}

impl ObjectImpl for ViewExt {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let p = self.widget.borrow();
        obj.add(&p.clone());
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: once_cell::sync::Lazy<Vec<glib::ParamSpec>> =
            once_cell::sync::Lazy::new(|| {
                vec![glib::ParamSpec::new_string(
                    "path",
                    "Path",
                    "Path",
                    None,
                    glib::ParamFlags::READWRITE,
                )]
            });

        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        if pspec.name() == "path" {
            self.path.replace(value.get().unwrap());
        }
    }
}

glib::wrapper! {
    pub struct View(ObjectSubclass<ViewExt>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl View {
    pub fn new(path: &str) -> Self {
        let this: Self = glib::Object::new(&[("path", &path)])
            .expect("files_and_file::Viewの作成に失敗しました。");

        this.reload_files();

        this
    }

    fn get_ext(&self) -> &ViewExt {
        ViewExt::from_instance(self)
    }

    fn get_paned(&self) -> gtk::Paned {
        let ext = self.get_ext();
        let a = ext.widget.borrow();

        a.clone()
    }

    fn build_go_parent_button(&self) -> gtk::Button {
        let go_parent_button = gtk::Button::with_label("D ..");
        go_parent_button.connect_clicked(glib::clone!(@weak self as this => move |_| {
            {
                let mut p = this.get_ext().path.borrow_mut();
                let new_path = format!("{}{}{}", p.as_ref().unwrap(), path::MAIN_SEPARATOR, "..");
                p.replace(new_path);
            }
            this.reload_files();
        }));
        let label = go_parent_button
            .child()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();
        label.set_xalign(0.0);

        go_parent_button
    }

    fn build_filer(&self, path: &str) -> gtk::ListBox {
        let list_box = gtk::ListBox::new();
        list_box.add(&self.build_go_parent_button());

        for entry in list_files(path) {
            let button = gtk::Button::with_label(&match entry.type_ {
                FileType::Directory => format!("D {}", entry.name),
                FileType::File => format!("F {}", entry.name),
            });
            button.connect_clicked(glib::clone!(@weak self as this => move |_| {
                let ext = this.get_ext();
                match entry.type_ {
                    FileType::Directory => {
                        {
                            let mut p = ext.path.borrow_mut();
                            let new_path = format!("{}{}{}", p.as_ref().unwrap(), path::MAIN_SEPARATOR, entry.name);
                            p.replace(new_path);
                        }
                        this.reload_files();
                    },
                    FileType::File => {
                        let p = ext.path.borrow();
                        let new_path = format!("{}{}{}", p.as_ref().unwrap(), path::MAIN_SEPARATOR, entry.name);
                        this.open_file(&new_path);
                    }
                };
            }));
            let label = button.child().unwrap().downcast::<gtk::Label>().unwrap();
            label.set_xalign(0.0);
            list_box.add(&button);
        }
        list_box.set_selection_mode(gtk::SelectionMode::None);

        list_box
    }

    fn replace_paned_child1(&self, scrolled_window: &gtk::ScrolledWindow) {
        let paned = self.get_paned();
        if let Some(widget) = paned.child1() {
            paned.remove(&widget);
        }
        paned.add1(scrolled_window);
        paned.show_all();
    }

    fn replace_paned_child2(&self, scrolled_window: &gtk::Box) {
        let paned = self.get_paned();
        if let Some(widget) = paned.child2() {
            paned.remove(&widget);
        }

        paned.add2(scrolled_window);
        paned.show_all();
    }

    /// ファイラーを現在のパスの現在の状態に合わせます。
    fn reload_files(&self) {
        let path_ref = self.get_ext().path.borrow();
        let list_box = self.build_filer(path_ref.as_ref().unwrap());
        let scrolled_window =
            gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scrolled_window.add(&list_box);
        self.replace_paned_child1(&scrolled_window);
    }

    fn open_file(&self, path: &str) {
        let content = fs::read_to_string(path).unwrap();
        let scrolled_window =
            gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        let text_view = gtk::TextView::new();
        text_view.buffer().unwrap().set_text(&content);

        scrolled_window.add(&text_view);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let save_button = gtk::Button::with_label("保存");

        let p = path.to_string();
        save_button.connect_clicked(move |_| {
            let buffer = text_view.buffer().unwrap();
            let (start, end) = buffer.bounds();
            let text = buffer.text(&start, &end, false).unwrap().to_string();
            log::debug!("保存内容: {}", text);
            let mut file = fs::File::create(&p).unwrap();
            file.write_all(text.as_bytes()).unwrap();
            file.flush().unwrap();
        });

        vbox.add(&save_button);
        vbox.pack_start(&scrolled_window, true, true, 0);

        self.replace_paned_child2(&vbox);
    }
}
