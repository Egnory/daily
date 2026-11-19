use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	fs::{self, OpenOptions},
	io,
	path::PathBuf,
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct DayData {
	pub sleep: f64,
	pub awake: f64,
	pub weight: f64,
	#[serde(default)]
	pub spend: Vec<f64>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DataFile {
	pub dates: BTreeMap<String, DayData>,
	pub configs: Configs,

	#[serde(skip)]
	path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ThemeConfig {
	pub mode: String,
	pub light: String,
	pub dark: String,
}

impl Default for ThemeConfig {
	fn default() -> Self {
		Self { mode: "system".into(), light: "ayu".into(), dark: "nord".into() }
	}
}

#[derive(Serialize, Deserialize)]
pub struct Configs {
	pub length: usize,
	#[serde(default)]
	pub theme: ThemeConfig,
}

impl Default for Configs {
	fn default() -> Self {
		Self { length: 30, theme: ThemeConfig::default() }
	}
}

impl DataFile {
	fn data_paths() -> Vec<PathBuf> {
		let mut paths = vec![PathBuf::from("data/daily.json")];

		if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
			paths.push(PathBuf::from(path).join("daily.json"));
		}

		if let Some(base_dirs) = BaseDirs::new() {
			paths.push(base_dirs.data_local_dir().join("daily.json"));
		}

		paths
	}

	fn resolve_path() -> io::Result<PathBuf> {
		let paths = Self::data_paths();

		for path in &paths {
			if path.is_file() && OpenOptions::new().read(true).write(true).open(path).is_ok() {
				return Ok(path.clone());
			}
		}

		for path in paths.iter().rev() {
			let Some(parent) = path.parent() else {
				continue;
			};

			if fs::create_dir_all(parent).is_err() {
				continue;
			}

			if OpenOptions::new().read(true).write(true).create_new(true).open(path).is_ok() {
				return Ok(path.clone());
			}
		}

		Err(io::Error::other("无法找到或创建 daily.json"))
	}

	pub fn load() -> Self {
		let path = Self::resolve_path().expect("无法初始化 daily.json");
		eprintln!("Loading daily.json: {}", path.display());
		let json = fs::read_to_string(&path).unwrap_or_default();
		let should_save = json.trim().is_empty() || serde_json::from_str::<serde_json::Value>(&json).map_or(false, |value| value.pointer("/configs/theme").is_none());
		let mut data: Self = serde_json::from_str(&json).unwrap_or_default();
		data.path = path;
		if should_save {
			data.save();
		}
		data
	}

	pub fn save(&self) {
		fs::write(&self.path, serde_json::to_string(self).unwrap()).unwrap();
	}

	pub fn build(&mut self, date: &str) {
		self.dates.entry(date.to_owned()).or_default();
	}
}
