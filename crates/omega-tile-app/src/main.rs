#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

macro_rules! L {
    ($str:literal) => {
        $crate::LocalizedString::new($str)
    };
}

mod commands;
mod enums;
mod generate;
mod menu;
mod progress;
mod widgets;

use druid::widget::{Button, Either, Flex, Label, WidgetExt};
use druid::{
    lens, AppDelegate, AppLauncher, Application, Data, DelegateCtx, Env, Event, EventCtx, Lens,
    LensExt, LocalizedString, Widget, WindowDesc, WindowId,
};

use enums::Enum;
use generate::HandleResult;
use omega_tile::ts::image;
use std::collections::HashSet;
use std::sync::Arc;

fn main() {
    let app = AppData { make: None };

    let main_window = WindowDesc::new(ui_builder)
        .title(L!("omega-tile-app-name"))
        .menu(menu::make_menu(&app))
        .window_size((800.0, 600.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate::default())
        .configure_env(|_env| {})
        // .use_simple_logger()
        .launch(app)
        .expect("launch failed");
}

fn ui_make_builder() -> impl Widget<Make> {
    let controls = Button::sized(
        L!("Generate"),
        |ctx: &mut EventCtx, _, _| ctx.submit_command(commands::GENERATE_TILES, ctx.window_id()),
        200.0,
        30.0,
    );

    let finish_label = Label::new(|data: &Make, _: &Env| match &data.in_progress {
        None | Some(HandleData::InProgress(_)) => String::new(),
        Some(HandleData::Finish(status)) => match status.as_ref() {
            HandleResult::Ok(_) => String::new(),
            HandleResult::Fail(_) => "Generating Fail.".to_string(),
            HandleResult::Success(path) => format!("Done. ({})", path.to_string_lossy()),
        },
    });

    Flex::column()
        .with_child(
            Label::new(|data: &Make, _: &Env| data.path.to_string_lossy().to_string()).center(),
            0.0,
        )
        .with_child(widgets::Image::new().lens(Make::img).center(), 0.0)
        .with_child(
            Enum::new(ProgressMode::Ready, |data: &Make, _| match data.in_progress.as_ref() {
                None => ProgressMode::Ready,
                Some(it) => match it {
                    HandleData::InProgress(_) => ProgressMode::InProgress,
                    HandleData::Finish(_) => ProgressMode::Finish,
                },
            })
            .with_branch(ProgressMode::Ready, controls)
            .with_branch(
                ProgressMode::InProgress,
                progress::Progress::new().lens(Make::in_progress),
            )
            .with_branch(ProgressMode::Finish, finish_label)
            .padding(5.0)
            .center(),
            0.0,
        )
        .center()
}

fn ui_builder() -> impl Widget<AppData> {
    let maker = ui_make_builder();

    Flex::column().with_child(
        Either::new(
            |app: &AppData, _| app.make.is_some(),
            maker.lens(lens::Id.map(
                |app: &AppData| app.make.as_ref().unwrap().clone(),
                |app: &mut AppData, make: Make| {
                    if let Some(it) = app.make.as_mut() {
                        *it = make;
                    }
                },
            )),
            Label::new(L!("omega-tile-app-name")).center(),
        ),
        1.0,
    )
}

#[derive(Data, Clone)]
struct ImageData(Arc<image::DynamicImage>);

#[derive(Data, Clone)]
enum HandleData {
    InProgress(Arc<generate::Handle>),
    Finish(Arc<HandleResult<String>>),
}

#[derive(Data, Clone, Eq, PartialEq, Hash)]
enum ProgressMode {
    Ready,
    InProgress,
    Finish,
}

#[derive(Data, Clone, Lens)]
struct Make {
    img: ImageData,
    path: Arc<std::path::PathBuf>,
    in_progress: Option<HandleData>,
}

#[derive(Data, Clone, Lens)]
struct AppData {
    make: Option<Make>,
}

type Error = anyhow::Error;

#[derive(Debug, Default)]
struct Delegate {
    windows: HashSet<WindowId>,
}

fn to_rgba(img: image::DynamicImage) -> image::RgbaImage {
    match img {
        image::DynamicImage::ImageRgba8(img) => img,
        _ => img.to_rgba(),
    }
}

impl AppData {
    fn do_open_image(&mut self, path: &std::path::Path) -> Result<(), Error> {
        let img = image::open(path)?;
        let make = Make {
            img: ImageData(Arc::new(image::DynamicImage::ImageRgba8(to_rgba(img)))),
            path: Arc::new(path.to_path_buf()),
            in_progress: None,
        };
        self.make = Some(make);
        Ok(())
    }

    fn do_generate(&mut self, output_path: &std::path::Path) -> Result<(), Error> {
        let make = self.make.as_mut().ok_or_else(|| anyhow::anyhow!("Not in edit mode"))?;

        let options = generate::GenerateOptions {
            variation: omega_tile::WTileVariation::V16,
            size: 256,
            seed: 102,
        };

        make.in_progress = Some(HandleData::InProgress(Arc::new(generate::generate(
            &make.path,
            output_path,
            &options,
        ))));
        Ok(())
    }
}

impl Delegate {
    fn handle_command(
        &mut self,
        data: &mut AppData,
        ctx: &mut DelegateCtx,
        cmd: &druid::Command,
    ) -> Result<(), Error> {
        match &cmd.selector {
            &commands::FILE_EXIT_ACTION => {
                ctx.submit_command(druid::commands::CLOSE_WINDOW.into(), None);
            }
            &druid::commands::OPEN_FILE => {
                let info = cmd
                    .get_object::<druid::FileInfo>()
                    .ok_or_else(|| anyhow::anyhow!("api violation"))?;
                data.do_open_image(info.path())?;
            }
            &druid::commands::SAVE_FILE => {
                let info = cmd
                    .get_object::<druid::FileInfo>()
                    .ok_or_else(|| anyhow::anyhow!("api violation"))?;

                data.do_generate(info.path())?;

                // Submit an event to start progress anim-frame event start
                for id in &self.windows {
                    ctx.submit_command(commands::TRIGGER_PROGRESS.into(), Some(*id));
                }
            }
            &commands::GENERATE_TILES => {
                ctx.submit_command(commands::generate_tiles_command(), None);
            }
            _ => (),
        }

        Ok(())
    }
}

impl AppDelegate<AppData> for Delegate {
    fn event(
        &mut self,
        event: Event,
        data: &mut AppData,
        _env: &Env,
        ctx: &mut DelegateCtx,
    ) -> Option<Event> {
        match event {
            Event::Command(ref cmd) => {
                if let Err(err) = self.handle_command(data, ctx, cmd) {
                    eprintln!("Error => {}", err.to_string());
                }
            }

            _ => (),
        };

        Some(event)
    }

    fn window_added(
        &mut self,
        id: WindowId,
        _data: &mut AppData,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        self.windows.insert(id);
    }

    fn window_removed(
        &mut self,
        id: WindowId,
        _data: &mut AppData,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        self.windows.remove(&id);

        // FIXME: Use commands::QUIT_APP
        // It do not works right now, maybe a druid bug
        Application::quit();
    }
}
