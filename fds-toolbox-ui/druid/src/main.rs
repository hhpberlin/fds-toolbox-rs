mod plot_2d;
mod state;
mod tab;

use druid::widget::{Button, Flex, Label};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = 0_u32;
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .configure_env(|env, _| {
            // env.get_all().for_each(|(k, v)| {
            //     println!("{}: {:?}", k, v);
            // });

            // env.set(
            //     theme::WINDOW_BACKGROUND_COLOR,
            //     Color::rgb8(0x2e, 0x34, 0x36),
            // );
            // env.set(theme::BUTTON_BORDER_RADIUS, 5);
            // env.set(theme::BUTTON_BORDER_WIDTH, 0);
            // env.set(theme::BUTTON_DARK, Color::rgb8(0x4c, 0x56, 0x5a));
            // env.set(theme::BUTTON_LIGHT, Color::rgb8(0xfc, 0x56, 0x5a));
            // env.set(theme::BUTTON_DARK, 0);
        })
        .launch(data)
}

fn ui_builder() -> impl Widget<u32> {
    // The label text will be computed dynamically based on the current locale and count
    let text =
        LocalizedString::new("hello-counter").with_arg("count", |data: &u32, _env| (*data).into());
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("increment")
        .on_click(|_ctx, data, _env| *data += 1)
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)
}
