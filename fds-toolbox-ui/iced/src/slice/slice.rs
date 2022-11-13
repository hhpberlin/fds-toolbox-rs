use fds_toolbox_core::formats::slcf::SliceFile;
use iced::{Element, widget::{canvas::{Program, Geometry, Cursor}, Canvas}, Rectangle, Theme, Length};

#[derive(Debug)]
struct Slice<'a> {
    slice: &'a SliceFile,
    frame: usize,
}

impl Slice<'_> {
    // pub fn new(slice: &'_ SliceFile) -> Slice<'_> {
    //     Self { slice, frame: 0 }
    // }

    pub fn view<'a, Message: Copy + 'a>(&'a self) -> Element<'a, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> Program<Message> for Slice<'_> {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        todo!()
    }
    
}