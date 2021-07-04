use chrono::prelude::*;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell;
use std::fs;
use std::io::prelude::*;
use std::rc;

#[derive(Debug, Default)]
pub struct ViewExt {
    widget: cell::RefCell<gtk::Grid>,
    path: cell::RefCell<Option<String>>,
}

#[derive(Debug, serde::Serialize)]
struct Card {
    key: String,
    text: String,
}

impl Card {
    fn new(key: String, text: String) -> Self {
        Self { key, text }
    }
}

#[derive(Debug, serde::Serialize)]
struct DailyBucket {
    version: usize,
    date: chrono::NaiveDate,
    cards: Vec<Card>,
}

impl DailyBucket {
    fn new(date: NaiveDate, cards: Vec<Card>) -> Self {
        Self {
            version: 1,
            date,
            cards,
        }
    }
}

fn compute_last_sunday(today: chrono::NaiveDate) -> chrono::NaiveDate {
    for i in 1..7 {
        let date = today - chrono::Duration::days(i);
        if date.weekday() == chrono::Weekday::Sun {
            return date;
        }
    }

    today
}

fn weekday_to_japanese(weekday: chrono::Weekday) -> String {
    match weekday {
        chrono::Weekday::Mon => "月",
        chrono::Weekday::Tue => "火",
        chrono::Weekday::Wed => "水",
        chrono::Weekday::Thu => "木",
        chrono::Weekday::Fri => "金",
        chrono::Weekday::Sat => "土",
        chrono::Weekday::Sun => "日",
    }
    .to_string()
}

fn build_text_view(save: Save) -> gtk::TextView {
    let text_view = gtk::TextViewBuilder::new()
        .hexpand(true)
        .wrap_mode(gtk::WrapMode::Char)
        .build();
    let buffer = text_view.buffer().unwrap();
    let last = rc::Rc::new(cell::Cell::new(chrono::Utc::now()));
    let dration_in_seconds = 1;
    let auto_save_skip_duration = chrono::Duration::seconds(dration_in_seconds as i64);
    let sleep_duration = std::time::Duration::from_secs(dration_in_seconds);
    let save = std::sync::Arc::new(save);
    buffer.connect_changed(move |_buffer| {
        let now = chrono::Utc::now();
        last.replace(now);
        let last = last.clone();
        let save = save.clone();
        glib::MainContext::default().spawn_local(async move {
            async_std::task::sleep(sleep_duration).await;
            if last.get() + auto_save_skip_duration > chrono::Utc::now() {
                log::debug!("最終入力から十分に時間が経過していないので、保存処理を省略します。");
                return;
            }
            save();
        });
    });

    text_view
}

type Save = Box<dyn Fn()>;
type SaveFactory = Box<dyn Fn(chrono::NaiveDate, &gtk::ListBox) -> Save>;
fn save_column_factory_factory(root: &str) -> SaveFactory {
    let root = root.to_string();
    Box::new(
        move |date: chrono::NaiveDate, list_box: &gtk::ListBox| -> Save {
            let dir = coo::libs::expand_path(&format!("{}/{}", &root, &date.format("%Y/%Y-%m")));
            let dest = format!("{}/{}.toml", &dir, &date.format("%Y-%m-%d"));
            fs::create_dir_all(&dir).unwrap();
            let list_box = list_box.clone();

            Box::new(move || {
                let mut cards: Vec<Card> = vec![];
                for child in list_box.children() {
                    let row = child.downcast::<gtk::ListBoxRow>().unwrap();
                    let box_ = row.child().unwrap().downcast::<gtk::Box>().unwrap();

                    let box_children = box_.children();
                    let combo_box_text = box_children
                        .get(0)
                        .unwrap()
                        .clone()
                        .downcast::<gtk::ComboBoxText>()
                        .unwrap();
                    let key = CARD_KEYS
                        .get(combo_box_text.active().unwrap() as usize)
                        .unwrap()
                        .to_string();

                    let text_view = box_children
                        .get(1)
                        .unwrap()
                        .clone()
                        .downcast::<gtk::TextView>()
                        .unwrap();
                    let text_buffer = text_view.buffer().unwrap();
                    let (start, end) = text_buffer.bounds();
                    let text = text_buffer.text(&start, &end, false).unwrap().to_string();

                    cards.push(Card::new(text, key));
                }

                let content = toml::to_string_pretty(&DailyBucket::new(date, cards)).unwrap();
                log::debug!("保存先: {}, 保存内容:\n{}", &dest, &content);
                let mut file = fs::File::create(&dest).unwrap();
                file.write_all(content.as_bytes()).unwrap();
                file.flush().unwrap();
            })
        },
    )
}

fn build_column(date: chrono::NaiveDate, save_factory: std::sync::Arc<SaveFactory>) -> gtk::Box {
    let vbox = gtk::BoxBuilder::new()
        .orientation(gtk::Orientation::Vertical)
        .expand(true)
        .build();

    let title = format!("{}日 ({})", date.day(), weekday_to_japanese(date.weekday()));
    vbox.add(&gtk::Label::new(Some(&title)));

    let list_box = gtk::ListBoxBuilder::new()
        .expand(true)
        .selection_mode(gtk::SelectionMode::None)
        .build();
    list_box.connect_add(|list_box, _| {
        // ListBoxRowをフォーカス不可にしないと、ListBoxにaddしたTextViewが選択後即座にフォーカスを失います。
        for child in list_box.children() {
            child.set_can_focus(false);
        }
    });

    let hbox = gtk::BoxBuilder::new()
        .orientation(gtk::Orientation::Horizontal)
        .expand(true)
        .build();
    list_box.add(&hbox);
    let combo_box_text = gtk::ComboBoxTextBuilder::new().build();
    for key in CARD_KEYS {
        combo_box_text.append_text(key);
    }
    combo_box_text.set_active(Some(0));
    hbox.add(&combo_box_text);
    hbox.add(&build_text_view(save_factory(date, &list_box)));

    let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
    scrolled_window.add(&list_box);
    vbox.add(&scrolled_window);

    vbox
}

static CARD_KEYS: &[&str] = &["📝", "✅", "⭕", "❌", "➕", "➖"];

#[glib::object_subclass]
impl ObjectSubclass for ViewExt {
    const NAME: &'static str = "AssortedCard";
    type Type = View;
    type ParentType = gtk::Bin;

    fn new() -> Self {
        let grid = gtk::Grid::new();
        grid.set_column_spacing(4);
        grid.set_row_spacing(4);

        Self {
            widget: cell::RefCell::new(grid),
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
        obj.add(&p.clone().upcast::<gtk::Widget>());
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
        let this = glib::Object::new(&[("path", &path)])
            .expect("assorted_card::Viewの作成に失敗しました。");

        let ext = ViewExt::from_instance(&this);
        let grid = ext.widget.borrow().clone().downcast::<gtk::Grid>().unwrap();
        let save_factory = std::sync::Arc::new(save_column_factory_factory(path));

        let calendar = gtk::CalendarBuilder::new().expand(true).build();
        calendar.set_display_options(
            gtk::CalendarDisplayOptions::SHOW_WEEK_NUMBERS
                | gtk::CalendarDisplayOptions::SHOW_HEADING,
        );
        let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
        scrolled_window.add(&calendar);
        grid.attach(&scrolled_window, 0, 0, 1, 1);

        let sunday = compute_last_sunday(chrono::Local::today().naive_local());
        for (i, (left, top)) in vec![(1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1), (3, 1)]
            .iter()
            .enumerate()
        {
            grid.attach(
                &build_column(
                    sunday + chrono::Duration::days(i as i64),
                    save_factory.clone(),
                ),
                *left,
                *top,
                1,
                1,
            );
        }

        this
    }
}
