use druid::{Command, FileDialogOptions, FileSpec, Selector};
const IMAGE_FILE_TYPE: FileSpec = FileSpec::new("Images", &["bmp", "png", "gif", "jpg", "jpeg"]);

pub(crate) const FILE_EXIT_ACTION: Selector = Selector::new("menu-exit-action");
pub(crate) const GENERATE_TILES: Selector = Selector::new("generate-tiles-action");

pub(crate) const TRIGGER_PROGRESS: Selector = Selector::new("trigger-progress-action");

pub(crate) fn file_open_command() -> Command {
    Command::new(
        druid::commands::SHOW_OPEN_PANEL,
        FileDialogOptions::new().allowed_types(vec![IMAGE_FILE_TYPE]),
    )
}

pub(crate) fn generate_tiles_command() -> Command {
    Command::new(
        druid::commands::SHOW_SAVE_PANEL,
        FileDialogOptions::new().allowed_types(vec![IMAGE_FILE_TYPE]),
    )
}
