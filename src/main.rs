use gpui_kit::{
    AppContext, Bounds, WindowBounds, WindowOptions, application, init, open_window, point, px,
    size,
};
mod app;

fn main() {
    application().run(|cx| {
        init(cx);
        open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: size(px(1280.), px(1440.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|ctx| app::App::new(window, ctx)),
        )
        .expect("Failed to open window");
    });
}
