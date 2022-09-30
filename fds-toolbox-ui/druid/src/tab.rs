use druid::Widget;

pub trait Tab<T>
where
    Self: Sized,
{
    type Data;

    fn title(&self) -> String;
    fn build_widget(&mut self) -> Box<dyn Widget<(Self::Data, T)>>;
}
