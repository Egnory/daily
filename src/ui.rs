use super::io::DataFile;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};
use gpui_kit::{
	component::{
		ActiveTheme,
		calendar::{Calendar, CalendarEvent, CalendarState, Date, Matcher},
		chart::{AreaChart, CandlestickChart},
		input::{Input, InputEvent, InputState},
		time_field::{TimeField, TimeFieldEvent, TimeFieldState, TimePrecision},
	},
	*,
};

const RATIO: f32 = 8.;
const LARGE: Pixels = px(RATIO * 3.);
const MIDLE: Pixels = px(RATIO * 2.);
const SMALL: Pixels = px(RATIO);

macro_rules! time_subscription {
	($cx:ident, $window:ident, $app:ident, $input:ident, $field:ident) => {
		$cx.subscribe_in(&$app.$input, $window, |this, _, event, _, cx| {
			let TimeFieldEvent::Change(time) = event;
			this.data.dates.get_mut(&this.date).unwrap().$field = (time.hour() * 60 + time.minute()) as f64;
			this.data.save();
			cx.notify();
		})
	};
}

macro_rules! input {
	($cx:ident, $state:expr) => {
		div().w(relative(0.8)).p(SMALL).shadow_md().rounded(SMALL).flex().items_center().justify_center().child(Input::new($state).appearance(false).w_full().text_center().text_size(MIDLE)).bg($cx.theme().background)
	};
}

macro_rules! time_input {
	($cx:ident, $state:expr) => {
		div().w(relative(0.8)).p(SMALL).shadow_md().rounded(SMALL).flex().items_center().justify_center().child(TimeField::new($state)).bg($cx.theme().background)
	};
}

macro_rules! data_input {
	($label:expr, $average:expr, $input:expr) => {
		div().w_full().flex().flex_col().items_center().gap_2().child(div().w_full().text_center().text_size(MIDLE).child(format!("{} {}", $label, $average))).child($input)
	};
}

macro_rules! head {
	($text:expr) => {
		div().w_full().p(MIDLE).text_center().text_size(LARGE).font_weight(FontWeight::BOLD).child($text)
	};
}

macro_rules! chart {
	($cx:ident, $chart:expr) => {
		div().w_full().flex_1().min_h_0().flex().flex_col().child(div().w_full().flex_1().min_h_0().child($chart.x_axis(false).grid(false))).bg($cx.theme().background)
	};
}

macro_rules! area {
	($cx:ident, $data:expr, $name:expr, $y:expr) => {
		AreaChart::new($data).name($name).stroke($cx.theme().chart_1).fill($cx.theme().chart_1).x(|d| d.0.clone()).y($y)
	};
}

pub struct App {
	data: DataFile,
	date: String,
	offset: usize,

	calendar: Entity<CalendarState>,
	weight_input: Entity<InputState>,
	sleep_input: Entity<TimeFieldState>,
	awake_input: Entity<TimeFieldState>,
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
		data.dates.keys().position(|key| key.as_str() == date).map(|i| data.dates.len() - i - 1).unwrap_or(0)
	}

	fn num_to_time(value: f64) -> String {
		let value = value as i64 + if value < 0. { 1440 } else { 0 };
		format!("{:02}:{:02}", value / 60, value % 60)
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
		let (weight, sleep, awake, spend) = (day.weight, day.sleep, day.awake, day.spend.iter().map(ToString::to_string).collect::<Vec<_>>().join(" "));

		let time = |value: f64| {
			let value = (value as i64).rem_euclid(1440) as u32;
			NaiveTime::from_hms_opt(value / 60, value % 60, 0).unwrap()
		};

		self.weight_input.update(cx, |s, cx| s.set_value(weight.to_string(), window, cx));
		self.sleep_input.update(cx, |s, cx| s.set_time(time(sleep), window, cx));
		self.awake_input.update(cx, |s, cx| s.set_time(time(awake), window, cx));
		self.spend_input.update(cx, |s, cx| s.set_value(spend, window, cx));
		self.length_input.update(cx, |s, cx| s.set_value(self.data.configs.length.to_string(), window, cx));
		self.offset_input.update(cx, |s, cx| s.set_value(self.offset.to_string(), window, cx));
	}

	pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
		let mut data = DataFile::load();
		let date = data.dates.keys().next_back().cloned().unwrap_or_else(Self::today);

		data.build(&date);

		let offset = Self::offset_for_date(&data, &date);
		data.configs.length = data.configs.length.clamp(1, data.dates.len().saturating_sub(offset).max(1));

		let calendar = cx.new(|cx| {
			let min_date = data.dates.keys().next().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()).unwrap_or_else(|| Local::now().naive_local().date());
			let max_date = Local::now().naive_local().date();

			let mut calendar = CalendarState::new(window, cx).disabled_matcher(Matcher::custom(move |date| *date < min_date || *date > max_date)).year_range((min_date.year(), max_date.year() + 1));

			if let Ok(d) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
				calendar.set_date(d, window, cx);
			}

			calendar
		});

		let weight_input = cx.new(|cx| InputState::new(window, cx).placeholder("WEIGHT"));
		let sleep_input = cx.new(|cx| TimeFieldState::new(window, cx).precision(TimePrecision::Minute));
		let awake_input = cx.new(|cx| TimeFieldState::new(window, cx).precision(TimePrecision::Minute));
		let spend_input = cx.new(|cx| InputState::new(window, cx).placeholder("SPEND"));
		let length_input = cx.new(|cx| InputState::new(window, cx).placeholder("LENGTH").default_value(&data.configs.length.to_string()));
		let offset_input = cx.new(|cx| InputState::new(window, cx).placeholder("OFFSET").default_value(&offset.to_string()));

		let mut app = Self { data, date, offset, calendar, weight_input, sleep_input, awake_input, spend_input, length_input, offset_input, _subscriptions: Vec::new() };

		app.sync_inputs(window, cx);

		let date_sub = cx.subscribe_in(&app.calendar, window, |this, _, event, window, cx| {
			if let CalendarEvent::Selected(Date::Single(Some(date))) = event {
				let date = date.format("%Y-%m-%d").to_string();

				if date.as_str() < "1970-01-01" || date.as_str() > Self::today().as_str() {
					return;
				}

				this.data.build(&date);
				this.date = date;
				this.offset = Self::offset_for_date(&this.data, &this.date);
				this.data.configs.length = this.data.configs.length.clamp(1, this.data.dates.len().saturating_sub(this.offset).max(1));

				this.sync_inputs(window, cx);
				this.data.save();
				cx.notify();
			}
		});

		let weight_sub = cx.subscribe_in(&app.weight_input, window, |this, state, event, _, cx| {
			if matches!(event, InputEvent::Change) {
				if let Ok(w) = state.read(cx).value().parse::<f64>() {
					if (0.0..1000.0).contains(&w) {
						this.data.dates.get_mut(&this.date).unwrap().weight = w;
						this.data.save();
						cx.notify();
					}
				}
			}
		});

		let sleep_sub = time_subscription!(cx, window, app, sleep_input, sleep);
		let awake_sub = time_subscription!(cx, window, app, awake_input, awake);

		let spend_sub = cx.subscribe_in(&app.spend_input, window, |this, state, event, _, cx| {
			if matches!(event, InputEvent::Change) {
				let v = state.read(cx).value().to_string();

				if v.ends_with('.') || v.ends_with(".0") {
					return;
				}

				this.data.dates.get_mut(&this.date).unwrap().spend = v.split_whitespace().filter_map(|s| s.parse::<f64>().ok()).filter(|n| *n >= 0.0).collect();

				this.data.save();
				cx.notify();
			}
		});

		let length_sub = cx.subscribe_in(&app.length_input, window, |this, state, event, window, cx| {
			if matches!(event, InputEvent::Change) {
				if let Ok(len) = state.read(cx).value().parse::<usize>() {
					this.data.configs.length = len.clamp(1, this.data.dates.len().saturating_sub(this.offset).max(1));

					let value = this.data.configs.length.to_string();
					this.length_input.update(cx, |s, cx| s.set_value(value, window, cx));

					this.data.save();
					cx.notify();
				}
			}
		});

		let offset_sub = cx.subscribe_in(&app.offset_input, window, |this, state, event, window, cx| {
			if matches!(event, InputEvent::Change) {
				if let Ok(offset) = state.read(cx).value().parse::<usize>() {
					this.offset = offset.min(this.data.dates.len().saturating_sub(1));
					this.data.configs.length = this.data.configs.length.clamp(1, this.data.dates.len().saturating_sub(this.offset).max(1));

					let (length, offset) = (this.data.configs.length.to_string(), this.offset.to_string());
					this.length_input.update(cx, |s, cx| s.set_value(length, window, cx));
					this.offset_input.update(cx, |s, cx| s.set_value(offset, window, cx));

					this.data.save();
					cx.notify();
				}
			}
		});

		app._subscriptions = vec![date_sub, weight_sub, sleep_sub, awake_sub, spend_sub, length_sub, offset_sub];
		app
	}
}

impl Render for App {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let keys: Vec<_> = {
			let keys: Vec<String> = self.data.dates.keys().cloned().collect();
			let end = keys.len().saturating_sub(self.offset);
			keys[end.saturating_sub(self.data.configs.length)..end].to_vec()
		};

		let weight_avg = Self::avg(keys.iter().map(|d| self.data.dates[d].weight));
		let sleep_avg = Self::avg(keys.iter().map(|d| Self::check_sleep(self.data.dates[d].sleep)));
		let awake_avg = Self::avg(keys.iter().map(|d| self.data.dates[d].awake));
		let spend_avg = Self::avg(keys.iter().map(|d| self.data.dates[d].spend.iter().sum::<f64>()));

		let weight: Vec<_> = keys.iter().map(|d| (d.clone(), self.data.dates[d].weight)).collect();
		let sleep: Vec<_> = keys
			.iter()
			.map(|d| {
				let day = &self.data.dates[d];
				let sleep = Self::check_sleep(day.sleep);
				(d.clone(), sleep, day.awake - sleep, day.awake)
			})
			.collect();
		let spend: Vec<_> = keys.iter().map(|d| (d.clone(), self.data.dates[d].spend.iter().sum::<f64>())).collect();

		let w_max = weight.iter().map(|d| d.1).fold(f64::NEG_INFINITY, f64::max) + 0.5;
		let w_min = weight.iter().map(|d| d.1).fold(f64::INFINITY, f64::min) - 0.5;

		div()
			.flex()
			.flex_row()
			.gap_3()
			.p(MIDLE)
			.size_full()
			.bg(cx.theme().background)
			.font_family("Maple Mono NF")
			.child(
				div()
					.py(px(RATIO * 8.))
					.w(px(RATIO * 40.))
					.h_full()
					.flex()
					.flex_col()
					.items_center()
					.justify_between()
					.child(div().w_full().flex().flex_col().items_center().child(head!("TODAY IS")).w(relative(0.8)).child(Calendar::new(&self.calendar).w_full().border_0().bg(cx.theme().background).shadow_md()))
					.child(
						div()
							.w_full()
							.flex()
							.flex_col()
							.items_center()
							.gap_2()
							.child(head!("WRITE DAILY"))
							.child(data_input!("AVG WEIGHT", format!("{:.2}", weight_avg), input!(cx, &self.weight_input)))
							.child(data_input!("AVG SLEEP", Self::num_to_time(sleep_avg), time_input!(cx, &self.sleep_input)))
							.child(data_input!("AVG AWAKE", Self::num_to_time(awake_avg), time_input!(cx, &self.awake_input)))
							.child(data_input!("AVG SPEND", format!("{:.2}", spend_avg), input!(cx, &self.spend_input))),
					)
					.child(div().w_full().flex().flex_col().items_center().gap_2().child(head!("SETTING")).child(input!(cx, &self.length_input)).child(input!(cx, &self.offset_input))),
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
					.child(chart!(cx, area!(cx, weight.clone(), "WEIGHT", |d| d.1).y_domain(w_min, w_max)))
					.child(chart!(
						cx,
						CandlestickChart::new(sleep.clone()).bullish(cx.theme().chart_1).x(|d| d.0.clone()).open(|d| d.1).high(|d| d.1.max(d.3)).low(|d| d.1.min(d.3)).close(|d| d.3).tick_margin(2).tooltip_content(|d, _, _| {
							let f = |l: &str, v: f64| format!("{l}  {}", App::num_to_time(v));
							div().flex().flex_col().gap_1().child(f("AWAKE   ", d.3)).child(f("SLEEP   ", d.1)).child(f("DURATION", d.2))
						})
					))
					.child(chart!(cx, area!(cx, spend, "SPEND", |d| (d.1 * 100.).round() / 100.))),
			)
	}
}
