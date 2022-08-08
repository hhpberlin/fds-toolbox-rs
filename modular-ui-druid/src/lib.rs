use druid::{Widget, Env, WidgetExt, Lens, Data, PlatformError, WindowDesc, AppLauncher, MenuDesc, platform_menus, LocalizedString};
use druid::widget::{Label, TextBox, Flex, Align, Split, Tabs};

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;

#[derive(Clone, Data, Lens)]
pub struct HelloState {
    name: String,
}

impl Default for HelloState {
    fn default() -> Self {
        HelloState {
            name: "World".to_string(),
        }
    }
}

pub fn build_ui() -> impl Widget<HelloState> {
    // a label that will determine its text based on the current app data.
    let label = Label::new(|data: &HelloState, _env: &Env| format!("Hello {}!", data.name));
    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(HelloState::name);

    // arrange the two widgets vertically, with some padding
    // let layout = Flex::column()
    //     .with_child(label)
    //     .with_spacer(VERTICAL_WIDGET_SPACING)
    //     .with_child(textbox);

    let layout = Split::rows(label, 
        Tabs::new()
        .with_tab("Text", textbox)
        .with_tab("Label", Label::new("Mogus"))
        
        ).draggable(true);

    // center the two widgets in the available space
    Align::centered(layout)
}

pub fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui).menu(app_menu());

    AppLauncher::with_window(main_window).launch(<_>::default())?;

    Ok(())
}

// This menu seems to be required for copy-paste to work sometimes.
// Known issue: https://github.com/linebender/druid/issues/1030

pub(crate) fn app_menu<T: druid::Data>() -> MenuDesc<T> {
    let mut menu = MenuDesc::empty();
    // #[cfg(target_os = "macos")]
    {
        // menu = menu.append(platform_menus::mac::application::default());
        menu = menu.append(edit_menu());
    }

    menu
}

fn edit_menu<T: druid::Data>() -> MenuDesc<T> {
    MenuDesc::new(LocalizedString::new("common-menu-edit-menu"))
        .append(platform_menus::common::undo())
        .append(platform_menus::common::redo())
        .append_separator()
        .append(platform_menus::common::cut().disabled())
        .append(platform_menus::common::copy())
        .append(platform_menus::common::paste())
}