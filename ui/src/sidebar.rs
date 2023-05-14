use iced::Element;

use crate::app::Message;

fn view(
    simulations: impl Iterator<Item = SimulationIdx>,
selected: impl Fn(Simulations) -> bool,

) -> Element<'_, Message> {

}

enum Series0 {
    Device { sim: SimulationIdx, idx: usize },
    Hrr { sim: SimulationIdx, idx: usize },
    // Slice { sim: SimulationIdx, idx: usize, x: usize, y: usize },
