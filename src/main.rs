#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use gpui_kit::{
	AppContext, WindowBounds, WindowOptions, application,
	component::{Theme, ThemeConfig, ThemeMode},
	init, size,
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
	pub fn get(name: &str) -> Colors {
		match name {
			"nord" => Colors { background: 0xECEFF4, foreground: 0x2E3440, primary: 0x5E81AC, primary_foreground: 0xECEFF4, secondary: 0xE5E9F0, secondary_foreground: 0x3B4252, accent: 0x88C0D0, accent_foreground: 0x2E3440 },
			"nord-dark" => Colors { background: 0x2E3440, foreground: 0xECEFF4, primary: 0x5E81AC, primary_foreground: 0xECEFF4, secondary: 0x3B4252, secondary_foreground: 0xD8DEE9, accent: 0x88C0D0, accent_foreground: 0x2E3440 },
			"ayu" => Colors { background: 0xFCFCFC, foreground: 0x5C6166, primary: 0xFF9940, primary_foreground: 0xFCFCFC, secondary: 0xF3F4F5, secondary_foreground: 0x5C6166, accent: 0x55B4D4, accent_foreground: 0xFCFCFC },
			"ayu-dark" => Colors { background: 0x0A0E14, foreground: 0xB3B1AD, primary: 0xE6B450, primary_foreground: 0x0A0E14, secondary: 0x10141C, secondary_foreground: 0xB3B1AD, accent: 0x95E6CB, accent_foreground: 0x0A0E14 },
			"latte" => Colors { background: 0xEFF1F5, foreground: 0x4C4F69, primary: 0x1E66F5, primary_foreground: 0xEFF1F5, secondary: 0xE6E9EF, secondary_foreground: 0x4C4F69, accent: 0x8839EF, accent_foreground: 0xEFF1F5 },
			"mocha" => Colors { background: 0x1E1E2E, foreground: 0xCDD6F4, primary: 0x89B4FA, primary_foreground: 0x1E1E2E, secondary: 0x313244, secondary_foreground: 0xCDD6F4, accent: 0xCBA6F7, accent_foreground: 0x1E1E2E },
			"macchiato" => Colors { background: 0x24273A, foreground: 0xCAD3F5, primary: 0x8AADF4, primary_foreground: 0x24273A, secondary: 0x363A4F, secondary_foreground: 0xCAD3F5, accent: 0xC6A0F6, accent_foreground: 0x24273A },
			"frappe" => Colors { background: 0x303446, foreground: 0xC6D0F5, primary: 0x8CAAEE, primary_foreground: 0x303446, secondary: 0x414559, secondary_foreground: 0xC6D0F5, accent: 0xCA9EE6, accent_foreground: 0x303446 },
			_ => Colors { background: 0x2E3440, foreground: 0xECEFF4, primary: 0x5E81AC, primary_foreground: 0xECEFF4, secondary: 0x3B4252, secondary_foreground: 0xD8DEE9, accent: 0x88C0D0, accent_foreground: 0x2E3440 },
		}
	}
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

		let data = io::DataFile::load();
		let theme_config = data.configs.theme;
		let system_mode = theme_config.mode == "system";

		Theme::update(cx, |theme| {
			theme.light_theme = set(&theme_config.light, ThemeMode::Light, &theme::get(&theme_config.light)).into();
			theme.dark_theme = set(&theme_config.dark, ThemeMode::Dark, &theme::get(&theme_config.dark)).into();
			if !system_mode {
				theme.mode = if theme_config.mode == "dark" { ThemeMode::Dark } else { ThemeMode::Light };
			}
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
		})
		.detach();
	});
}
