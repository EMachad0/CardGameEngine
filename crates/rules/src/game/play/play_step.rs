//! The play step, and what a play reads before its first write.

use crate::cards::object::ObjectId;
use crate::effect::ParsedEffect;
use crate::event::Event;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::PendingEffects;
use crate::game::play::playability::WasPlayable;
use crate::game::player::Players;
use crate::game::player_input::awaiting::{PlayerInput, WasAwaitingTargeting};
use crate::game::reporting::Report;
use crate::game::stats::cost;
use crate::ids::PlayerId;
use crate::target::Bound;
use crate::zones::WasInHand;

pub(in crate::game) struct Play {
    player_id: PlayerId,
    object_id: ObjectId,
}

/// A card about to be played, with what its steps write, all read before the first write.
pub(in crate::game) enum PreparedPlay {
    Minion { card: WasInHand, play: PlayStep },
    Spell { card: WasInHand, play: PlayStep },
}

/// What a play step writes: the cost, the targeting it closes and the bound play effects.
pub(in crate::game) struct PlayStep {
    player_id: PlayerId,
    object_id: ObjectId,
    awaiting_targeting: Option<WasAwaitingTargeting>,
    cost: u8,
    play_effects: Vec<ParsedEffect<Bound>>,
}

impl From<Play> for Event {
    fn from(play: Play) -> Self {
        Event::Played {
            player_id: play.player_id,
            object_id: play.object_id,
        }
    }
}

pub(in crate::game) fn prepare(
    cards: CardsRef,
    player_input: &PlayerInput,
    playable: WasPlayable,
) -> PreparedPlay {
    let (card, awaiting_targeting) = playable.into_parts();
    let cost = cost(cards, cards.zones.still_in_hand(&card));
    let targeting = awaiting_targeting
        .as_ref()
        .map(|awaiting_targeting| player_input.still_targeting(awaiting_targeting));
    let card_def = cards.card_def(card.object_id());
    let play_effects = card_def.bind_play_effects(|target_id| {
        targeting
            .expect("a card that declares targets should be played from its targeting")
            .targeted_for(target_id)
            .collect()
    });
    let play = PlayStep {
        player_id: card.player_id(),
        object_id: card.object_id(),
        awaiting_targeting,
        cost,
        play_effects,
    };
    if card_def.has_board_presence() {
        PreparedPlay::Minion { card, play }
    } else {
        PreparedPlay::Spell { card, play }
    }
}

/// If the card is played from the player's targeting, closes it.
pub(in crate::game) fn play(
    step: PlayStep,
    players: &mut Players,
    player_input: &mut PlayerInput,
    pending: &mut PendingEffects,
) -> Report {
    let PlayStep {
        player_id,
        object_id,
        awaiting_targeting,
        cost,
        play_effects,
    } = step;
    players.mana_mut(player_id).pay(cost);
    if let Some(awaiting_targeting) = awaiting_targeting {
        player_input.commit_targeting(awaiting_targeting);
    }
    if !play_effects.is_empty() {
        pending.push(object_id, play_effects);
    }

    Report::Played(Play {
        player_id,
        object_id,
    })
}
