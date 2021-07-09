use gtk::prelude::*;

pub fn expand_path(path: &str) -> String {
    return shellexpand::tilde(path).into_owned();
}

pub fn find_first_child_by_name<T: glib::IsA<gtk::Widget>>(
    widget: &gtk::Widget,
    name: &str,
) -> Option<T> {
    find_first_widget_by_name(widget, name).and_then(|widget| widget.downcast().ok())
}

fn find_first_widget_by_name<T: glib::IsA<gtk::Widget>>(
    widget: &T,
    name: &str,
) -> Option<gtk::Widget> {
    if let Ok(container) = widget.clone().dynamic_cast::<gtk::Container>() {
        for child in container.children() {
            if child.widget_name() == name {
                return Some(child);
            }
            if let Some(widget) = find_first_widget_by_name(&child, name) {
                return Some(widget);
            }
        }
    }
    if let Ok(bin) = widget.clone().dynamic_cast::<gtk::Bin>() {
        if let Some(child) = bin.child() {
            if child.widget_name() == name {
                return Some(child);
            }
            if let Some(widget) = find_first_widget_by_name(&child, name) {
                return Some(widget);
            }
        }
    }
    None
}
