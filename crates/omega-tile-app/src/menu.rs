use crate::commands;
use crate::AppData;
use druid::{KeyCode, MenuDesc, MenuItem, RawMods};

pub(crate) fn make_menu(app: &AppData) -> MenuDesc<AppData> {
    MenuDesc::empty().append(file_menu(app))
}

fn file_menu(_app: &AppData) -> MenuDesc<AppData> {
    MenuDesc::new(L!("menu-file-menu")).append(open()).append_separator().append(exit())
}

macro_rules! register_menu_items {
    ($($name:ident => ($sel:literal, $cmd:expr $(, $mods:ident, $keycode:ident)? )),*) => {
        $(
        fn $name() -> MenuItem<AppData> {
            MenuItem::new(L!($sel), $cmd)
                $( .hotkey(RawMods::$mods, KeyCode::$keycode) )?
        })*
    }
}

register_menu_items! {
    // files
    open => ("menu-file-open", commands::file_open_command(), Ctrl, KeyO),
    exit => ("menu-file-exit", commands::FILE_EXIT_ACTION, Alt, F4)
}
