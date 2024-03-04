use lsp_types::Url;

pub fn resolve_in(path: &str, base: &Url) -> Option<Url> {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return Url::parse(&format!("file://{}{}", home.display(), &path[1..])).ok();
        }
    }

    let options = Url::options().base_url(Some(base));
    options.parse(path).ok()
}
