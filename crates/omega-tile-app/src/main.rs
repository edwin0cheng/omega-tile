macro_rules! L {
    ($str:literal) => {
        $crate::LocalizedString::new($str)
    };
}

mod commands;
mod generate;
mod menu;
mod progress;
mod widgets;

use druid::widget::{Button, Either, Flex, Label, WidgetExt};
use druid::{
    AppDelegate, AppLauncher, Application, Data, DelegateCtx, Env, Event, EventCtx, Lens,
    LocalizedString, Widget, WindowDesc, WindowId,
};

use omega_tile::ts::image;
use std::collections::HashSet;
use std::sync::Arc;

fn main() {
    let app_state = AppState { img: None, path: None, in_progress: None };

    let main_window = WindowDesc::new(ui_builder)
        .title(L!("omega-tile-app-name"))
        .menu(menu::make_menu(&app_state))
        .window_size((800.0, 600.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate::default())
        .configure_env(|_env| {})
        // .use_simple_logger()
        .launch(app_state)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppState> {
    let name = |data: &AppState, _: &Env| {
        data.path.as_ref().map(|it| it.to_string_lossy()).unwrap_or("".into()).to_string()
    };

    let controls = Button::sized(
        L!("Generate"),
        |ctx: &mut EventCtx, _, _| ctx.submit_command(commands::GENERATE_TILES, ctx.window_id()),
        200.0,
        30.0,
    );

    let main_content = Flex::column()
        .with_child(Label::new(name).center(), 0.0)
        .with_child(widgets::Image::new().lens(AppState::img).center(), 0.0)
        .with_child(
            Either::new(
                |data: &AppState, _| data.in_progress.is_some(),
                progress::Progress::new().lens(AppState::in_progress),
                controls,
            )
            .padding(5.0)
            .center(),
            0.0,
        )
        .center();

    Flex::column().with_child(
        Either::new(
            |app: &AppState, _| app.img.is_some(),
            main_content,
            Label::new(L!("omega-tile-app-name")).center(),
        ),
        1.0,
    )
}

#[derive(Data, Clone)]
struct ImageData(Arc<image::DynamicImage>);

#[derive(Data, Clone, Lens)]
struct AppState {
    img: Option<ImageData>,
    path: Option<Arc<std::path::PathBuf>>,
    in_progress: Option<Arc<generate::Handle>>,
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

impl AppState {
    fn do_open_image(&mut self, path: &std::path::Path) -> Result<(), Error> {
        let img = image::open(path)?;
        self.img = Some(ImageData(Arc::new(image::DynamicImage::ImageRgba8(to_rgba(img)))));
        self.path = Some(Arc::new(path.to_path_buf()));
        Ok(())
    }

    fn do_generate(&mut self, output_path: &std::path::Path) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or_else(|| anyhow::anyhow!("Current path is empty"))?;

        let options = generate::GenerateOptions {
            variation: omega_tile::WTileVariation::V16,
            size: 256,
            seed: 102,
        };

        self.in_progress = Some(Arc::new(generate::generate(&path, output_path, &options)));
        Ok(())
    }
}

impl Delegate {
    fn handle_command(
        &mut self,
        data: &mut AppState,
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

impl AppDelegate<AppState> for Delegate {
    fn event(
        &mut self,
        event: Event,
        data: &mut AppState,
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
        _data: &mut AppState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        self.windows.insert(id);
    }

    fn window_removed(
        &mut self,
        id: WindowId,
        _data: &mut AppState,
        _env: &Env,
        _ctx: &mut DelegateCtx,
    ) {
        self.windows.remove(&id);

        // FIXME: Use commands::QUIT_APP
        // It do not works right now, maybe a druid bug
        Application::quit();
    }
}
