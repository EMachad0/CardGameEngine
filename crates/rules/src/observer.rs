use crate::{Event, Game, PlayerId, View};

pub struct Views<'g>(&'g Game);

impl Views<'_> {
    pub fn of(&self, viewer: PlayerId) -> View {
        self.0.view(viewer)
    }
}

pub trait Observer {
    fn event(&mut self, e: &Event);
    fn checkpoint(&mut self, views: Views<'_>);
}

impl Observer for () {
    fn event(&mut self, _e: &Event) {}

    fn checkpoint(&mut self, _views: Views<'_>) {}
}
