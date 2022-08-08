use druid::Widget;

pub struct Pane<T> {
    pub title: String,
    pub content: Box<dyn Widget<T>>,
}

impl<T> Pane<T> {
    pub fn new(title: String, content: Box<dyn Widget<T>>) -> Self {
        Pane {
            title,
            content,
        }
    }
}

impl<T> Widget<T> for Pane<T> {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut T, env: &druid::Env) {
        todo!()
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &T, env: &druid::Env) {
        todo!()
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        todo!()
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &T, env: &druid::Env) -> druid::Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        todo!()
    }
}