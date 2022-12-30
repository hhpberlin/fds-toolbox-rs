struct ListElem<'a> {
    name: &'a str,
    selected: bool,
}

fn view_list<'a>(
    // mut series: RefMut<'a, HashMap<SimulationIdx<SliceSeriesIdx>, ListElem>>,
    // model: &'a Simulations,
    elements: impl IntoIterator<Item = ListElem<'a>>,
) -> Element<'a, Message> {
    let mut sidebar = Column::new();

    for elem in elements
    {
        // TODO: This does not work with multiple simulations
        let global_idx = SimulationIdx(0, TimeSeriesIdx::Device(idx));

        sidebar = sidebar
            .push(row![
                container(checkbox(elem.name),
                    elem.selected,
                    move |checked| {
                        if checked {
                            Message::Add(global_idx)
                        } else {
                            Message::Remove(global_idx)
                        }
                    },
                )
                .width(Length::Shrink),
                horizontal_space(Length::Fill),
                container(array_stats_vis(device.values.stats))
                    .width(Length::Units(100))
                    .height(Length::Units(20)),
            ])
            .max_width(400);
    }

    scrollable(sidebar).into()
}