use crate::generate::Handle;
use druid::widget;
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, Point, Rect, Size, UpdateCtx,
    Widget, WidgetPod,
};
use std::sync::Arc;

pub struct Progress {
    inner: WidgetPod<String, Box<dyn Widget<String>>>,
}

type DataType = Option<Arc<Handle>>;

impl Widget<DataType> for Progress {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DataType, env: &Env) {
        match event {
            Event::AnimFrame(_) => {
                if data.is_some() {
                    ctx.invalidate();
                }
            }
            _ => (),
        };

        if let Some(hdata) = data {
            match hdata.get() {
                Some(mut s) => {
                    self.inner.event(ctx, event, &mut s, env);
                    ctx.request_anim_frame();
                }
                None => {
                    *data = None;
                    ctx.invalidate();
                }
            }
        }
    }
    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: Option<&DataType>,
        data: &DataType,
        env: &Env,
    ) {
        if let Some(s) = data.as_ref().and_then(|it| it.get()) {
            self.inner.update(ctx, &s, env);
            ctx.invalidate();
        }
    }
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &DataType,
        env: &Env,
    ) -> Size {
        if let Some(s) = data.as_ref().and_then(|it| it.get()) {
            let size = self.inner.layout(ctx, bc, &s, env);
            self.inner.set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));
            size
        } else {
            Size::ZERO
        }
    }
    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &DataType, env: &Env) {
        if let Some(s) = data.as_ref().and_then(|it| it.get()) {
            self.inner.paint_with_offset(paint_ctx, &s, env);
        }
    }
}

impl Progress {
    pub fn new() -> Progress {
        let f = |data: &String, _: &Env| data.clone();
        let label = widget::Label::<String>::new(f);
        Progress { inner: WidgetPod::new(label).boxed() }
    }
}
