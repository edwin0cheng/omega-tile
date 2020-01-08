use crate::generate::HandleResult;
use crate::HandleData;
use druid::widget;
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, Point, Rect, Size, UpdateCtx,
    Widget, WidgetPod,
};
use std::sync::Arc;

pub struct Progress {
    inner: WidgetPod<String, Box<dyn Widget<String>>>,
}

type DataType = Option<HandleData>;

fn in_progress(data: &DataType) -> Option<String> {
    let data = data.as_ref()?;
    let res = match data {
        HandleData::InProgress(it) => it.get(),
        HandleData::Finish(_) => return None,
    };

    match res {
        HandleResult::Ok(s) => Some(s),
        _ => None,
    }
}

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

        if let Some(HandleData::InProgress(hdata)) = data {
            match hdata.get() {
                HandleResult::Ok(mut s) => {
                    self.inner.event(ctx, event, &mut s, env);
                    ctx.request_anim_frame();
                }
                a @ HandleResult::Fail(_) | a @ HandleResult::Success(_) => {
                    *data = Some(HandleData::Finish(Arc::new(a)));
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
        if let Some(s) = in_progress(data) {
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
        if let Some(s) = in_progress(data) {
            let size = self.inner.layout(ctx, bc, &s, env);
            self.inner.set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));
            size
        } else {
            Size::ZERO
        }
    }
    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &DataType, env: &Env) {
        if let Some(s) = in_progress(data) {
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
