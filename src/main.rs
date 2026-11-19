#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use gpui_kit::{
	AppContext, WindowBounds, WindowOptions, application,
	component::{Theme, ThemeConfig, ThemeMode},
	init, size,
};

mod io;
mod ui;

struct Colors {
	background: u32,
	foreground: u32,
	primary: u32,
	primary_foreground: u32,
	secondary: u32,
	secondary_foreground: u32,
	accent: u32,
	accent_foreground: u32,
}
macro_rules! colors {
	($bg:expr, $fg:expr, $p:expr, $pf:expr, $s:expr, $sf:expr, $a:expr, $af:expr) => {
		Colors { background: $bg, foreground: $fg, primary: $p, primary_foreground: $pf, secondary: $s, secondary_foreground: $sf, accent: $a, accent_foreground: $af }
	};
}
fn get(name: &str, mode: ThemeMode) -> Colors {
	match name {
		"nord" => colors!(0xECEFF4, 0x2E3440, 0x5E81AC, 0xECEFF4, 0xE5E9F0, 0x3B4252, 0x88C0D0, 0x2E3440),
		"nord-dark" => colors!(0x2E3440, 0xECEFF4, 0x5E81AC, 0xECEFF4, 0x3B4252, 0xD8DEE9, 0x88C0D0, 0x2E3440),
		"ayu" => colors!(0xFCFCFC, 0x5C6166, 0xFF9940, 0xFCFCFC, 0xF3F4F5, 0x5C6166, 0x55B4D4, 0xFCFCFC),
		"ayu-dark" => colors!(0x0A0E14, 0xB3B1AD, 0xE6B450, 0x0A0E14, 0x10141C, 0xB3B1AD, 0x95E6CB, 0x0A0E14),
		"latte" => colors!(0xEFF1F5, 0x4C4F69, 0x1E66F5, 0xEFF1F5, 0xE6E9EF, 0x4C4F69, 0x8839EF, 0xEFF1F5),
		"mocha" => colors!(0x1E1E2E, 0xCDD6F4, 0x89B4FA, 0x1E1E2E, 0x313244, 0xCDD6F4, 0xCBA6F7, 0x1E1E2E),
		"macchiato" => colors!(0x24273A, 0xCAD3F5, 0x8AADF4, 0x24273A, 0x363A4F, 0xCAD3F5, 0xC6A0F6, 0x24273A),
		"frappe" => colors!(0x303446, 0xC6D0F5, 0x8CAAEE, 0x303446, 0x414559, 0xC6D0F5, 0xCA9EE6, 0x303446),
		_ => {
			if let ThemeMode::Light = mode {
				colors!(0xFCFCFC, 0x5C6166, 0xFF9940, 0xFCFCFC, 0xF3F4F5, 0x5C6166, 0x55B4D4, 0xFCFCFC)
			} else {
				colors!(0x2E3440, 0xECEFF4, 0x5E81AC, 0xECEFF4, 0x3B4252, 0xD8DEE9, 0x88C0D0, 0x2E3440)
			}
		}
	}
}

fn set(name: &str, mode: ThemeMode, c: &Colors) -> ThemeConfig {
	let hex = |v: u32| Some(format!("#{:06X}", v).into());
	let mut config = ThemeConfig::default();
	config.name = name.into();
	config.mode = mode;
	config.colors.background = hex(c.background);
	config.colors.foreground = hex(c.foreground);
	config.colors.primary = hex(c.primary);
	config.colors.primary_foreground = hex(c.primary_foreground);
	config.colors.secondary = hex(c.secondary);
	config.colors.secondary_foreground = hex(c.secondary_foreground);
	config.colors.accent = hex(c.accent);
	config.colors.accent_foreground = hex(c.accent_foreground);
	config
}

fn apply_theme(window: Option<&mut gpui_kit::Window>, cx: &mut gpui_kit::App, mode: Option<ThemeMode>) {
	if let Some(mode) = mode {
		Theme::change(mode, window, cx);
	} else {
		Theme::sync_system_appearance(window, cx);
	}
	Theme::update(cx, |theme| theme.colors.chart_1 = theme.colors.accent.opacity(0.2));
}

fn main() {
	application().run(|cx| {
		init(cx);

		let theme_config = io::DataFile::load().configs.theme;

		Theme::update(cx, |theme| {
			theme.light_theme = set(&theme_config.light, ThemeMode::Light, &get(&theme_config.light, ThemeMode::Light)).into();
			theme.dark_theme = set(&theme_config.dark, ThemeMode::Dark, &get(&theme_config.dark, ThemeMode::Dark)).into();
		});

		cx.spawn(async move |cx| {
			cx.background_executor().timer(std::time::Duration::from_millis(100)).await;

			cx.update(|cx| {
				let display = cx.primary_display().or_else(|| cx.displays().into_iter().next());

				let Some(display) = display else {
					eprintln!("No display found after Wayland initialization");
					return;
				};

				let display_size = display.bounds().size;
				let window_size = size(display_size.width * 0.5, display_size.height);

				cx.open_window(WindowOptions { titlebar: None, window_bounds: Some(WindowBounds::centered(window_size, cx)), window_min_size: Some(window_size), ..Default::default() }, |window, cx| {
					if theme_config.mode == "system" {
						apply_theme(Some(window), cx, None);
						window.observe_window_appearance(|window, cx| apply_theme(Some(window), cx, None)).detach();
					} else {
						apply_theme(Some(window), cx, Some(if theme_config.mode == "dark" { ThemeMode::Dark } else { ThemeMode::Light }));
					}
					cx.new(|ctx| ui::App::new(window, ctx))
				})
				.expect("Failed to open window");
			});
		})
		.detach();
	});
}
