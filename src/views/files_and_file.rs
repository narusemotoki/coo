use gio::prelude::*;
use glib::translate::{FromGlibPtrFull, ToGlib, ToGlibPtr};
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

pub struct FilesAndFileExt {
    widget: cell::RefCell<gtk::Paned>,
    path: cell::RefCell<Option<String>>,
}

static PROPERTIES: [glib::subclass::Property; 1] = [glib::subclass::Property("path", |path| {
    glib::ParamSpec::string(path, "Path", "Path", None, glib::ParamFlags::READWRITE)
})];

impl glib::subclass::types::ObjectSubclass for FilesAndFileExt {
    const NAME: &'static str = "FilesAndFile";
    type ParentType = gtk::Bin;
    type Instance = glib::subclass::simple::InstanceStruct<Self>;
    type Class = glib::subclass::simple::ClassStruct<Self>;

    glib::glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

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

impl gtk::subclass::bin::BinImpl for FilesAndFileExt {}
impl gtk::subclass::container::ContainerImpl for FilesAndFileExt {}
impl gtk::subclass::widget::WidgetImpl for FilesAndFileExt {}

impl glib::subclass::object::ObjectImpl for FilesAndFileExt {
    glib::glib_object_impl!();

    fn constructed(&self, obj: &glib::Object) {
        self.parent_constructed(obj);
        let self_ = obj.downcast_ref::<FilesAndFile>().unwrap();
        let p = self.widget.borrow();
        self_.add(&p.clone());
    }

    fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            glib::subclass::Property("path", ..) => {
                let path = value
                    .get()
                    .expect("FilesAndFileExtのset_propertyに渡されたpathの型が期待と違います。");
                self.path.replace(path);
            }
            _ => unimplemented!(),
        }
    }
}

glib::glib_wrapper! {
    pub struct FilesAndFile(
        Object<glib::subclass::simple::InstanceStruct<FilesAndFileExt>,
        glib::subclass::simple::ClassStruct<FilesAndFileExt>,
        FilesAndFileClass>)
        @extends gtk::Widget, gtk::Container, gtk::Bin;
    match fn {
        get_type => || FilesAndFileExt::get_type().to_glib(),
    }
}

impl FilesAndFile {
    pub fn new(path: &str) -> Self {
        let this: FilesAndFile = glib::Object::new(Self::static_type(), &[("path", &path)])
            .expect("FilesAndFileの作成に失敗しました。")
            .downcast()
            .expect("FilesAndFileの型が間違っています。");

        this.reload_files();

        return this;
    }

    fn get_ext(&self) -> &FilesAndFileExt {
        FilesAndFileExt::from_instance(self)
    }

    fn get_paned(&self) -> gtk::Paned {
        let ext = self.get_ext();
        let a = ext.widget.borrow();
        return a.clone();
    }

    fn build_filer(&self, path: &str) -> gtk::ListBox {
        let list_box = gtk::ListBox::new();

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
            let label = button
                .get_child()
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap();
            label.set_xalign(0.0);
            list_box.add(&button);
        }
        list_box.set_selection_mode(gtk::SelectionMode::None);

        return list_box;
    }

    fn replace_paned_child1(&self, scrolled_window: &gtk::ScrolledWindow) {
        let paned = self.get_paned();
        match paned.get_child1() {
            Some(widget) => {
                paned.remove(&widget);
            }
            None => {}
        }
        paned.add1(scrolled_window);
        paned.show_all();
    }

    fn replace_paned_child2(&self, scrolled_window: &gtk::Box) {
        let paned = self.get_paned();
        match paned.get_child2() {
            Some(widget) => {
                paned.remove(&widget);
            }
            None => {}
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
        text_view.get_buffer().unwrap().set_text(&content);

        scrolled_window.add(&text_view);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let save_button = gtk::Button::with_label("保存");

        let p = path.to_string();
        save_button.connect_clicked(move |_| {
            let buffer = text_view.get_buffer().unwrap();
            let (start, end) = buffer.get_bounds();
            let text = buffer.get_text(&start, &end, true).unwrap().to_string();
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
