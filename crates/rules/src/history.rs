use crate::{
    cards::object::{ObjectBag, ObjectId},
    event::Event,
    ids::PlayerId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryKind {
    CardPlayed,
    CharacterDied,
    CharacterEntered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistoryEntry {
    player_id: PlayerId,
    kind: HistoryKind,
    object_id: ObjectId,
    turn: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerFilter {
    Active,
    All,
    Owner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnFilter {
    Current,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryQuery {
    pub kind: HistoryQueryKind,
    pub scope: PlayerFilter,
    pub turn: TurnFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryQueryKind {
    SpellsPlayed,
    MinionsDied,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct History {
    entries: Vec<HistoryEntry>,
}

impl History {
    /// Keeps a play, a board entry or a death. A death counts for the dead character's owner.
    pub(crate) fn record(&mut self, event: &Event, turn: u32, objects: &ObjectBag) {
        let (player_id, kind, object_id) = match *event {
            Event::Played {
                player_id,
                object_id,
            } => (player_id, HistoryKind::CardPlayed, object_id),
            Event::BoardEntered {
                player_id,
                object_id,
            } => (player_id, HistoryKind::CharacterEntered, object_id),
            Event::Died { object_id } => (
                objects.get(object_id).player_id,
                HistoryKind::CharacterDied,
                object_id,
            ),
            Event::TurnStarted { .. }
            | Event::TurnEnded { .. }
            | Event::Drew { .. }
            | Event::Damaged { .. }
            | Event::Revealed { .. }
            | Event::Picked { .. }
            | Event::Buried { .. }
            | Event::AddedToHand { .. }
            | Event::FatigueDamaged { .. }
            | Event::GameEnded { .. } => return,
        };
        self.entries.push(HistoryEntry {
            player_id,
            kind,
            object_id,
            turn,
        });
    }

    /// The cards `player_id` played in `turn`, oldest first. `None` matches any.
    pub(crate) fn played(
        &self,
        player_id: Option<PlayerId>,
        turn: Option<u32>,
    ) -> impl Iterator<Item = ObjectId> {
        self.of_kind(HistoryKind::CardPlayed, player_id, turn)
    }

    /// The characters of `player_id` that died in `turn`, oldest first. `None` matches any.
    pub(crate) fn died(
        &self,
        player_id: Option<PlayerId>,
        turn: Option<u32>,
    ) -> impl Iterator<Item = ObjectId> {
        self.of_kind(HistoryKind::CharacterDied, player_id, turn)
    }

    fn of_kind(
        &self,
        kind: HistoryKind,
        player_id: Option<PlayerId>,
        turn: Option<u32>,
    ) -> impl Iterator<Item = ObjectId> {
        self.entries
            .iter()
            .filter(move |entry| {
                entry.kind == kind
                    && player_id.is_none_or(|p| entry.player_id == p)
                    && turn.is_none_or(|t| entry.turn == t)
            })
            .map(|entry| entry.object_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::cards::binder::sets::base::static_card_definition::{RECRUIT, SPARK};

    const P0: PlayerId = PlayerId::new(0);
    const P1: PlayerId = PlayerId::new(1);

    fn played(player_id: PlayerId, object_id: ObjectId) -> Event {
        Event::Played {
            player_id,
            object_id,
        }
    }

    #[test]
    fn a_played_card_is_listed_for_the_player_who_played_it() {
        let mut objects = ObjectBag::default();
        let spark = objects.insert(SPARK, P0);
        let mut history = History::default();

        history.record(&played(P0, spark), 1, &objects);

        assert_eq!(history.played(Some(P0), None).collect::<Vec<_>>(), [spark]);
    }

    #[test]
    fn a_card_another_player_played_is_not_listed_for_a_player() {
        let mut objects = ObjectBag::default();
        let spark = objects.insert(SPARK, P0);
        let mut history = History::default();

        history.record(&played(P0, spark), 1, &objects);

        assert_eq!(history.played(Some(P1), None).count(), 0);
    }

    #[test]
    fn any_player_lists_the_cards_every_player_played() {
        let mut objects = ObjectBag::default();
        let first = objects.insert(SPARK, P0);
        let second = objects.insert(SPARK, P1);
        let mut history = History::default();

        history.record(&played(P0, first), 1, &objects);
        history.record(&played(P1, second), 2, &objects);

        assert_eq!(
            history.played(None, None).collect::<Vec<_>>(),
            [first, second]
        );
    }

    #[test]
    fn a_turn_lists_only_the_cards_played_in_it() {
        let mut objects = ObjectBag::default();
        let first = objects.insert(SPARK, P0);
        let second = objects.insert(SPARK, P0);
        let mut history = History::default();

        history.record(&played(P0, first), 1, &objects);
        history.record(&played(P0, second), 3, &objects);

        assert_eq!(history.played(None, Some(3)).collect::<Vec<_>>(), [second]);
    }

    #[test]
    fn a_dead_character_is_listed_for_its_owner() {
        let mut objects = ObjectBag::default();
        let recruit = objects.insert(RECRUIT, P1);
        let mut history = History::default();

        history.record(&Event::Died { object_id: recruit }, 2, &objects);

        assert_eq!(history.died(Some(P1), None).collect::<Vec<_>>(), [recruit]);
    }

    #[test]
    fn a_turn_lists_only_the_characters_that_died_in_it() {
        let mut objects = ObjectBag::default();
        let first = objects.insert(RECRUIT, P0);
        let second = objects.insert(RECRUIT, P0);
        let mut history = History::default();

        history.record(&Event::Died { object_id: first }, 1, &objects);
        history.record(&Event::Died { object_id: second }, 3, &objects);

        assert_eq!(history.died(None, Some(3)).collect::<Vec<_>>(), [second]);
    }

    #[test]
    fn a_dead_character_is_not_listed_for_another_player() {
        let mut objects = ObjectBag::default();
        let recruit = objects.insert(RECRUIT, P1);
        let mut history = History::default();

        history.record(&Event::Died { object_id: recruit }, 2, &objects);

        assert_eq!(history.died(Some(P0), None).count(), 0);
    }

    #[test]
    fn entering_the_board_or_dying_is_not_a_play() {
        let mut objects = ObjectBag::default();
        let recruit = objects.insert(RECRUIT, P0);
        let mut history = History::default();

        history.record(
            &Event::BoardEntered {
                player_id: P0,
                object_id: recruit,
            },
            1,
            &objects,
        );
        history.record(&Event::Died { object_id: recruit }, 1, &objects);

        assert_eq!(history.played(None, None).count(), 0);
    }

    #[test]
    fn a_play_or_a_draw_is_not_a_death() {
        let mut objects = ObjectBag::default();
        let recruit = objects.insert(RECRUIT, P0);
        let mut history = History::default();

        history.record(
            &Event::Drew {
                player_id: P0,
                object_id: recruit,
            },
            1,
            &objects,
        );
        history.record(&played(P0, recruit), 1, &objects);

        assert_eq!(history.died(None, None).count(), 0);
    }
}
