fn view<'a, Message: Copy + 'a>(
    children: impl Iterator<Item = (impl Fn(bool) -> pure::Element<'a, Message>, bool)>,
) -> pure::Element<'a, Message> {
    children
        .fold(pure::row(), |row, (child, selected)| {
            row.push(child(selected))
        })
        .into()
}
