use crate::ImageData;
use druid::piet::{ImageFormat, InterpolationMode};
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, Point, Rect, RenderContext, Size,
    UpdateCtx, Widget,
};
use image::GenericImageView;
use omega_tile::ts::image;

pub struct Image {}

impl Image {
    pub fn new() -> Self {
        Self {}
    }
}

type DataType = Option<ImageData>;

impl Widget<DataType> for Image {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut DataType, _env: &Env) {}

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: Option<&DataType>,
        _data: &DataType,
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        _bc: &BoxConstraints,
        data: &DataType,
        _env: &Env,
    ) -> Size {
        if let Some(data) = data {
            (data.0.width() as f64, data.0.height() as f64).into()
        } else {
            Size::ZERO
        }
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &DataType, _env: &Env) {
        let img = match data {
            Some(img) => img,
            None => return,
        };

        let size = (img.0.width() as usize, img.0.height() as usize);

        // FIXME: Draw image only in paint_ctx.region
        let image = paint_ctx
            .make_image(size.0, size.1, &img.0.as_rgba8().unwrap(), ImageFormat::RgbaSeparate)
            .unwrap();
        // The image is automatically scaled to fit the rect you pass to draw_image
        paint_ctx.draw_image(
            &image,
            Rect::from_origin_size(Point::ORIGIN, (size.0 as f64, size.1 as f64)),
            InterpolationMode::NearestNeighbor,
        );
    }
}
