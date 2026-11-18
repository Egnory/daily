use gpui_kit::{
	AppContext, WindowBounds, WindowOptions, application,
	component::{Theme, ThemeConfig, ThemeMode},
	init, open_window, size,
};

mod io;
mod ui;

mod theme {
	#![allow(dead_code)]
	pub struct Colors {
		pub background: u32,
		pub foreground: u32,
		pub primary: u32,
		pub primary_foreground: u32,
		pub secondary: u32,
		pub secondary_foreground: u32,
		pub accent: u32,
		pub accent_foreground: u32,
	}
	pub const NORD: Colors = Colors { background: 0xECEFF4, foreground: 0x2E3440, primary: 0x5E81AC, primary_foreground: 0xECEFF4, secondary: 0xE5E9F0, secondary_foreground: 0x3B4252, accent: 0x88C0D0, accent_foreground: 0x2E3440 };
	pub const NORD_DARK: Colors = Colors { background: 0x2E3440, foreground: 0xECEFF4, primary: 0x5E81AC, primary_foreground: 0xECEFF4, secondary: 0x3B4252, secondary_foreground: 0xD8DEE9, accent: 0x88C0D0, accent_foreground: 0x2E3440 };
	pub const AYU: Colors = Colors { background: 0xFCFCFC, foreground: 0x5C6166, primary: 0xFF9940, primary_foreground: 0xFCFCFC, secondary: 0xF3F4F5, secondary_foreground: 0x5C6166, accent: 0x55B4D4, accent_foreground: 0xFCFCFC };
	pub const AYU_DARK: Colors = Colors { background: 0x0A0E14, foreground: 0xB3B1AD, primary: 0xE6B450, primary_foreground: 0x0A0E14, secondary: 0x10141C, secondary_foreground: 0xB3B1AD, accent: 0x95E6CB, accent_foreground: 0x0A0E14 };
	pub const LATTE: Colors = Colors { background: 0xEFF1F5, foreground: 0x4C4F69, primary: 0x1E66F5, primary_foreground: 0xEFF1F5, secondary: 0xE6E9EF, secondary_foreground: 0x4C4F69, accent: 0x8839EF, accent_foreground: 0xEFF1F5 };
	pub const MOCHA: Colors = Colors { background: 0x1E1E2E, foreground: 0xCDD6F4, primary: 0x89B4FA, primary_foreground: 0x1E1E2E, secondary: 0x313244, secondary_foreground: 0xCDD6F4, accent: 0xCBA6F7, accent_foreground: 0x1E1E2E };
	pub const MACCHIATO: Colors = Colors { background: 0x24273A, foreground: 0xCAD3F5, primary: 0x8AADF4, primary_foreground: 0x24273A, secondary: 0x363A4F, secondary_foreground: 0xCAD3F5, accent: 0xC6A0F6, accent_foreground: 0x24273A };
	pub const FRAPPE: Colors = Colors { background: 0x303446, foreground: 0xC6D0F5, primary: 0x8CAAEE, primary_foreground: 0x303446, secondary: 0x414559, secondary_foreground: 0xC6D0F5, accent: 0xCA9EE6, accent_foreground: 0x303446 };
}
fn set(name: &str, mode: ThemeMode, colors: &theme::Colors) -> ThemeConfig {
	let mut config = ThemeConfig::default();
	config.mode = mode;
	config.name = name.into();
	config.colors.background = Some(format!("#{:06X}", colors.background).into());
	config.colors.foreground = Some(format!("#{:06X}", colors.foreground).into());
	config.colors.primary = Some(format!("#{:06X}", colors.primary).into());
	config.colors.primary_foreground = Some(format!("#{:06X}", colors.primary_foreground).into());
	config.colors.secondary = Some(format!("#{:06X}", colors.secondary).into());
	config.colors.secondary_foreground = Some(format!("#{:06X}", colors.secondary_foreground).into());
	config.colors.accent = Some(format!("#{:06X}", colors.accent).into());
	config.colors.accent_foreground = Some(format!("#{:06X}", colors.accent_foreground).into());
	config
}

fn main() {
	application().run(|cx| {
		init(cx);

		Theme::update(cx, |theme| {
			theme.light_theme = set("Light", ThemeMode::Light, &theme::AYU).into();
			theme.dark_theme = set("Dark", ThemeMode::Dark, &theme::NORD_DARK).into();
		});

		let display = cx.primary_display().expect("No display found");
		let display_size = display.bounds().size;
		let window_size = size(display_size.width * 0.5, display_size.height);

		open_window(WindowOptions { titlebar: None, window_bounds: Some(WindowBounds::centered(window_size, cx)), window_min_size: Some(window_size), ..Default::default() }, cx, |window, cx| {
			Theme::sync_system_appearance(Some(window), cx);
			Theme::update(cx, |theme| {
				theme.colors.chart_1 = theme.colors.accent.opacity(0.2);
			});

			window
				.observe_window_appearance(|window, cx| {
					Theme::sync_system_appearance(Some(window), cx);
					Theme::update(cx, |theme| {
						theme.colors.chart_1 = theme.colors.accent.opacity(0.2);
					});
				})
				.detach();

			cx.new(|ctx| ui::App::new(window, ctx))
		})
		.expect("Failed to open window");
	});
}
