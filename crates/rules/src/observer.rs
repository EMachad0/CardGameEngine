use crate::{Event, Game, PlayerId, View};

pub struct Views<'g>(&'g Game);

impl<'g> Views<'g> {
    pub(crate) fn new(game: &'g Game) -> Self {
        Self(game)
    }
}

impl Views<'_> {
    pub fn of(&self, viewer: PlayerId) -> View {
        self.0.view(viewer)
    }
}

pub trait Observer {
    fn event(&mut self, event: &Event);
    fn checkpoint(&mut self, views: Views<'_>);
}

impl Observer for () {
    fn event(&mut self, _event: &Event) {}

    fn checkpoint(&mut self, _views: Views<'_>) {}
}
