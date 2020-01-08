use druid::kurbo::{Point, Rect, Size};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget, WidgetPod,
};

use std::collections::HashMap;
use std::hash::Hash;

/// A widget that switches between two possible child views.
pub struct Enum<T: Data, U: Eq + Hash> {
    closure: Box<dyn Fn(&T, &Env) -> U>,
    branches: HashMap<U, WidgetPod<T, Box<dyn Widget<T>>>>,
    current: U,
}

impl<T: Data, U: Eq + Hash> Enum<T, U> {
    pub fn new(init: U, closure: impl Fn(&T, &Env) -> U + 'static) -> Enum<T, U> {
        Enum { closure: Box::new(closure), branches: HashMap::new(), current: init }
    }

    fn add_branch(&mut self, variant: U, w: impl Widget<T> + 'static) {
        self.branches.insert(variant, WidgetPod::new(w).boxed());
    }

    pub fn with_branch(mut self, variant: U, w: impl Widget<T> + 'static) -> Self {
        self.add_branch(variant, w);
        self
    }
}

impl<T: Data, U: Eq + Hash> Widget<T> for Enum<T, U> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(branch) = self.branches.get_mut(&self.current) {
            branch.event(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: Option<&T>, data: &T, env: &Env) {
        let current = (self.closure)(data, env);
        if current != self.current {
            self.current = current;
            ctx.invalidate();
            // TODO: more event flow to request here.
        }

        if let Some(branch) = self.branches.get_mut(&self.current) {
            branch.update(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        if let Some(branch) = self.branches.get_mut(&self.current) {
            let size = branch.layout(layout_ctx, bc, data, env);
            branch.set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));
            size
        } else {
            Size::ZERO
        }
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &T, env: &Env) {
        if let Some(branch) = self.branches.get_mut(&self.current) {
            branch.paint(paint_ctx, data, env);
        }
    }
}
