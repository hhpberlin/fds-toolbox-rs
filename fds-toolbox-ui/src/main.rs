use druid::{AppLauncher, WindowDesc, Widget, PlatformError, MenuDesc, platform_menus, LocalizedString};
use druid::widget::Label;
use modular_ui_druid::build_ui;

mod panes;

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
        menu = menu.append(platform_menus::mac::application::default());
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