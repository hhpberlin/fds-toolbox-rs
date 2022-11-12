use iced::{Element};
use iced::widget::Row;

fn view<'a, Message: Copy + 'a>(
    children: impl Iterator<Item = (impl Fn(bool) -> Element<'a, Message>, bool)>,
) -> Element<'a, Message> {
    children
        .fold(Row::new(), |row, (child, selected)| {
            row.push(child(selected))
        })
        .into()
}
