fn main() {
	if std::env::var("CARGO_CFG_WINDOWS").is_ok() && std::env::var("CI").is_ok() {
		winres::WindowsResource::new().set_icon("app.ico").compile().unwrap();
	}
}
