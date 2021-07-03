pub fn expand_path(path: &str) -> String {
    return shellexpand::tilde(path).into_owned();
}
