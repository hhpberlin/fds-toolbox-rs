use druid::Widget;

pub trait Tab<T>
    where Self: Sized
{
    fn title(&self) -> String;
    fn build_widget(&mut self) -> Box<dyn Widget<(Self, T)>>;
}