use chrono::{Datelike, Local, NaiveDate};
use directories::BaseDirs;
use gpui_kit::component::calendar::{Calendar, CalendarEvent, CalendarState, Date, Matcher};
use gpui_kit::component::chart::{AreaChart, CandlestickChart};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
struct DataFile {
    dates: BTreeMap<String, DayData>,
    configs: Configs,

    #[serde(skip)]
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct DayData {
    sleep: f64,
    awake: f64,
    weight: f64,
    #[serde(default)]
    spend: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
struct Configs {
    length: usize,
}

impl Default for Configs {
    fn default() -> Self {
        Self { length: 30 }
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

            if OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(path)
                .is_ok()
            {
                return Ok(path.clone());
            }
        }

        Err(io::Error::other("无法找到或创建 daily.json"))
    }

    fn load() -> Self {
        let path = Self::resolve_path().expect("无法初始化 daily.json");
        let json = fs::read_to_string(&path).unwrap_or_default();

        let mut data: Self = serde_json::from_str(&json).unwrap_or_else(|_| Self {
            dates: BTreeMap::new(),
            configs: Configs::default(),
            path: PathBuf::new(),
        });

        data.path = path;

        data
    }

    fn save(&self) {
        fs::write(&self.path, serde_json::to_string(self).unwrap()).unwrap();
    }

    fn build(&mut self, date: &str) {
        if date >= "1970-01-01" && !self.dates.contains_key(date) {
            self.dates.insert(date.to_owned(), DayData::default());
        }
    }
}

pub struct App {
    data: DataFile,
    date: String,
    offset: usize,

    calendar: Entity<CalendarState>,
    weight_input: Entity<InputState>,
    sleep_input: Entity<InputState>,
    awake_input: Entity<InputState>,
    spend_input: Entity<InputState>,
    length_input: Entity<InputState>,
    offset_input: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl App {
    fn today() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    fn offset_for_date(data: &DataFile, date: &str) -> usize {
        let keys: Vec<_> = data.dates.keys().collect();

        keys.iter()
            .position(|key| **key == date)
            .map(|i| keys.len() - i - 1)
            .unwrap_or(0)
    }

    fn max_offset(&self) -> usize {
        self.data.dates.len().saturating_sub(1)
    }

    fn max_length(&self) -> usize {
        self.data.dates.len().saturating_sub(self.offset).max(1)
    }

    fn sync_range_inputs(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.length_input.update(cx, |s, cx| {
            s.set_value(self.data.configs.length.to_string(), window, cx)
        });

        self.offset_input
            .update(cx, |s, cx| s.set_value(self.offset.to_string(), window, cx));
    }

    fn keys(&self) -> Vec<String> {
        let keys: Vec<String> = self.data.dates.keys().cloned().collect();
        let end = keys.len().saturating_sub(self.offset);
        let start = end.saturating_sub(self.data.configs.length);

        keys[start..end].to_vec()
    }

    fn num_to_time(value: f64) -> String {
        let value = value as i64 + if (value as i64) < 0 { 1440 } else { 0 };
        format!("{:02}:{:02}", (value - value % 60) / 60, value % 60)
    }

    fn str_to_time(value: &str) -> Option<f64> {
        let parts: Vec<&str> = value.split(':').collect();

        if parts.len() != 2 {
            return None;
        }

        let h: f64 = parts[0].parse().ok()?;
        let m: f64 = parts[1].parse().ok()?;

        if !(0.0..=23.0).contains(&h) || !(0.0..=59.0).contains(&m) {
            return None;
        }

        Some(h * 60.0 + m)
    }

    fn check_sleep(sleep: f64) -> f64 {
        sleep - if sleep < 720. { 0. } else { 1440. }
    }

    fn avg(values: impl Iterator<Item = f64>) -> f64 {
        let (sum, count) = values.fold((0., 0), |(s, c), v| (s + v, c + 1));

        if count == 0 { 0. } else { sum / count as f64 }
    }

    fn sync_inputs(&self, window: &mut Window, cx: &mut Context<Self>) {
        let day = &self.data.dates[&self.date];

        self.weight_input
            .update(cx, |s, cx| s.set_value(day.weight.to_string(), window, cx));

        self.sleep_input.update(cx, |s, cx| {
            s.set_value(Self::num_to_time(day.sleep), window, cx)
        });

        self.awake_input.update(cx, |s, cx| {
            s.set_value(Self::num_to_time(day.awake), window, cx)
        });

        self.spend_input.update(cx, |s, cx| {
            s.set_value(
                day.spend
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" "),
                window,
                cx,
            )
        });

        self.sync_range_inputs(window, cx);
    }

    fn update_date(&mut self, date: String, window: &mut Window, cx: &mut Context<Self>) {
        if date < "1970-01-01".to_string() || date > Self::today() {
            return;
        }

        self.data.build(&date);
        self.date = date;
        self.offset = Self::offset_for_date(&self.data, &self.date);

        self.data.configs.length = self.data.configs.length.clamp(1, self.max_length());

        self.sync_inputs(window, cx);
        self.data.save();
        cx.notify();
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut data = DataFile::load();

        let date = data
            .dates
            .keys()
            .next_back()
            .cloned()
            .unwrap_or_else(Self::today);

        data.build(&date);

        let offset = Self::offset_for_date(&data, &date);

        let max_length = data.dates.len().saturating_sub(offset).max(1);
        data.configs.length = data.configs.length.clamp(1, max_length);

        let calendar = cx.new(|cx| {
            let min_date = data
                .dates
                .keys()
                .next()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                .unwrap_or_else(|| Local::now().naive_local().date());

            let max_date = Local::now().naive_local().date();

            let mut calendar = CalendarState::new(window, cx)
                .disabled_matcher(Matcher::custom(move |date| {
                    *date < min_date || *date > max_date
                }))
                .year_range((min_date.year(), max_date.year() + 1));

            if let Ok(d) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                calendar.set_date(d, window, cx);
            }

            calendar
        });

        let weight_input = cx.new(|cx| InputState::new(window, cx).placeholder("WEIGHT"));
        let sleep_input = cx.new(|cx| InputState::new(window, cx).placeholder("SLEEP"));
        let awake_input = cx.new(|cx| InputState::new(window, cx).placeholder("AWAKE"));
        let spend_input = cx.new(|cx| InputState::new(window, cx).placeholder("SPEND"));

        let length_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("LENGTH")
                .default_value(&data.configs.length.to_string())
        });

        let offset_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("OFFSET")
                .default_value(&offset.to_string())
        });

        let mut app = Self {
            data,
            date: date.clone(),
            offset,
            calendar,
            weight_input,
            sleep_input,
            awake_input,
            spend_input,
            length_input,
            offset_input,
            _subscriptions: Vec::new(),
        };

        app.sync_inputs(window, cx);

        let date_sub = cx.subscribe_in(&app.calendar, window, |this, _state, event, window, cx| {
            if let CalendarEvent::Selected(Date::Single(Some(date))) = event {
                this.update_date(date.format("%Y-%m-%d").to_string(), window, cx);
            }
        });

        let weight_sub = cx.subscribe_in(&app.weight_input, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                let v = state.read(cx).value();

                if let Ok(w) = v.parse::<f64>() {
                    if w >= 0.0 && w < 1000.0 {
                        this.data.dates.get_mut(&this.date).unwrap().weight = w;
                        this.data.save();
                        cx.notify();
                    }
                }
            }
        });

        let sleep_sub = cx.subscribe_in(&app.sleep_input, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                if let Some(t) = Self::str_to_time(&state.read(cx).value()) {
                    this.data.dates.get_mut(&this.date).unwrap().sleep = t;
                    this.data.save();
                    cx.notify();
                }
            }
        });

        let awake_sub = cx.subscribe_in(&app.awake_input, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                if let Some(t) = Self::str_to_time(&state.read(cx).value()) {
                    this.data.dates.get_mut(&this.date).unwrap().awake = t;
                    this.data.save();
                    cx.notify();
                }
            }
        });

        let spend_sub = cx.subscribe_in(&app.spend_input, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                let v = state.read(cx).value().to_string();

                if v.ends_with('.') || v.ends_with(".0") {
                    return;
                }

                let spend: Vec<f64> = v
                    .split_whitespace()
                    .filter_map(|s| s.parse::<f64>().ok())
                    .filter(|n| *n >= 0.0)
                    .collect();

                this.data.dates.get_mut(&this.date).unwrap().spend = spend;
                this.data.save();
                cx.notify();
            }
        });

        let length_sub = cx.subscribe_in(
            &app.length_input,
            window,
            |this, state, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Ok(len) = state.read(cx).value().parse::<usize>() {
                        this.data.configs.length = len.clamp(1, this.max_length());
                        this.sync_range_inputs(window, cx);
                        this.data.save();
                        cx.notify();
                    }
                }
            },
        );

        let offset_sub = cx.subscribe_in(
            &app.offset_input,
            window,
            |this, state, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Ok(offset) = state.read(cx).value().parse::<usize>() {
                        this.offset = offset.min(this.max_offset());
                        this.data.configs.length =
                            this.data.configs.length.clamp(1, this.max_length());

                        this.sync_range_inputs(window, cx);
                        this.data.save();
                        cx.notify();
                    }
                }
            },
        );

        app._subscriptions = vec![
            date_sub, weight_sub, sleep_sub, awake_sub, spend_sub, length_sub, offset_sub,
        ];

        app
    }
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let keys = self.keys();

        let weight_avg = Self::avg(keys.iter().map(|d| self.data.dates[d].weight));

        let sleep_avg = Self::avg(
            keys.iter()
                .map(|d| Self::check_sleep(self.data.dates[d].sleep)),
        );

        let awake_avg = Self::avg(keys.iter().map(|d| self.data.dates[d].awake));

        let spend_avg = Self::avg(
            keys.iter()
                .map(|d| self.data.dates[d].spend.iter().sum::<f64>()),
        );

        let weight: Vec<_> = keys
            .iter()
            .map(|d| (d.clone(), self.data.dates[d].weight))
            .collect();

        let sleep: Vec<_> = keys
            .iter()
            .map(|d| {
                let day = &self.data.dates[d];
                let sleep = Self::check_sleep(day.sleep);

                (d.clone(), sleep, day.awake - sleep, day.awake)
            })
            .collect();

        let spend: Vec<_> = keys
            .iter()
            .map(|d| (d.clone(), self.data.dates[d].spend.iter().sum::<f64>()))
            .collect();

        let w_min = weight.iter().map(|d| d.1).fold(f64::INFINITY, f64::min) - 0.5;
        let w_max = weight.iter().map(|d| d.1).fold(f64::NEG_INFINITY, f64::max) + 0.5;

        div()
            .flex()
            .flex_row()
            .gap_3()
            .p(px(16.))
            .size_full()
            .font_family("Maple Mono NF")
            .child(
                div()
                    .py(px(60.))
                    .w(px(310.))
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .child(head("TODAY IS"))
                            .child(Calendar::new(&self.calendar).w_full().border_0()),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(head("WRITE DAILY"))
                            .child(data_input(
                                "AVG WEIGHT",
                                format!("{:.2}", weight_avg),
                                &self.weight_input,
                            ))
                            .child(data_input(
                                "AVG SLEEP",
                                Self::num_to_time(sleep_avg),
                                &self.sleep_input,
                            ))
                            .child(data_input(
                                "AVG AWAKE",
                                Self::num_to_time(awake_avg),
                                &self.awake_input,
                            ))
                            .child(data_input(
                                "AVG SPEND",
                                format!("{:.2}", spend_avg),
                                &self.spend_input,
                            )),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(head("SETTING"))
                            .child(input(&self.length_input))
                            .child(input(&self.offset_input)),
                    ),
            )
            .child(
                div()
                    .id("charts")
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .overflow_y_scroll()
                    .child(chart(
                        AreaChart::new(weight.clone())
                            .name("WEIGHT")
                            .x_axis(false)
                            .x(|d| d.0.clone())
                            .y(|d| d.1)
                            .y_domain(w_min, w_max),
                    ))
                    .child(chart(
                        CandlestickChart::new(sleep.clone())
                            .x_axis(false)
                            .x(|d| d.0.clone())
                            .open(|d| d.1)
                            .high(|d| d.1.max(d.3))
                            .low(|d| d.1.min(d.3))
                            .close(|d| d.3)
                            .tick_margin(2)
                            .grid(false)
                            .tooltip_content(|d, _, _| {
                                let f = |l: &str, v: f64| format!("{l}  {}", App::num_to_time(v));

                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(f("AWAKE   ", d.3))
                                    .child(f("SLEEP   ", d.1))
                                    .child(f("DURATION", d.2))
                            }),
                    ))
                    .child(chart(
                        AreaChart::new(spend)
                            .name("SPEND")
                            .x_axis(false)
                            .x(|d| d.0.clone())
                            .y(|d| (d.1 * 100.).round() / 100.),
                    )),
            )
    }
}

fn head(text: &'static str) -> impl IntoElement {
    div()
        .w_full()
        .p(px(16.))
        .text_center()
        .text_size(px(24.))
        .font_weight(FontWeight::BOLD)
        .child(text)
}

fn input(state: &Entity<InputState>) -> impl IntoElement {
    div()
        .w(relative(0.8))
        .p(px(5.))
        .border_1()
        .border_dashed()
        .border_color(rgb(0x888888))
        .rounded(px(5.))
        .flex()
        .items_center()
        .justify_center()
        .child(
            Input::new(state)
                .appearance(false)
                .w_full()
                .text_center()
                .text_size(px(16.)),
        )
}

fn data_input(
    label: &'static str,
    average: String,
    state: &Entity<InputState>,
) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(
            div()
                .w_full()
                .text_center()
                .text_size(px(13.))
                .child(format!("{label} {average}")),
        )
        .child(input(state))
}

fn chart(chart: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(div().w_full().flex_1().min_h_0().child(chart))
}
