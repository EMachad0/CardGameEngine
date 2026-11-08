use crate::action::Action;
use crate::cards::object::ObjectId;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::{EffectStack, WasAwaitingPicking};
use crate::game::play::playability::{Playability, WasPlayable};
use crate::game::player::Player;
use crate::game::player_input::awaiting::{
    Awaiting, PlayerInput, WasAwaitingNothing, WasAwaitingTargeting, any_awaiting, awaiting,
};
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::targeting_walk::{can_start_targeting, may_stop_targeting, next_candidates};
use crate::target::TargetId;

/// An action legal for one player when the check ran, with what the check learned about the game.
pub(super) enum LegalAction {
    Play(WasPlayable),
    Pick(WasAwaitingPicking),
    StartTargeting {
        object_id: ObjectId,
        awaiting_nothing: WasAwaitingNothing,
    },
    Target {
        awaiting_targeting: WasAwaitingTargeting,
        target_id: TargetId,
        object_id: ObjectId,
    },
    Cancel(WasAwaitingTargeting),
    EndTurn,
}

/// What a player sends for the legal action.
impl From<&LegalAction> for Action {
    fn from(legal: &LegalAction) -> Self {
        match *legal {
            LegalAction::Play(ref playable) => Action::Play {
                object_id: playable.object_id(),
            },
            LegalAction::Pick(ref awaiting_picking) => Action::Pick {
                object_id: awaiting_picking.object_id(),
            },
            LegalAction::StartTargeting { object_id, .. } => Action::StartTargeting { object_id },
            LegalAction::Target {
                target_id,
                object_id,
                ..
            } => Action::Target {
                target_id,
                object_id,
            },
            LegalAction::Cancel(ref awaiting_targeting) => Action::Cancel {
                object_id: awaiting_targeting.object_id(),
            },
            LegalAction::EndTurn => Action::EndTurn,
        }
    }
}

/// The one definition of legality.
pub(super) fn legal_actions(
    cards: CardsRef,
    standings: &Standings,
    player_input: &PlayerInput,
    effect_stack: &EffectStack,
    player: &Player,
) -> Vec<LegalAction> {
    let player_id = player.id();
    if standings.outcome().is_some() {
        return Vec::new();
    }

    let mut actions = Vec::new();
    match awaiting(cards, player_input, effect_stack, player_id) {
        Awaiting::Nothing(awaits_nothing) => {
            if cards.turn_order.get_active_player_id() != player_id
                || any_awaiting(player_input, effect_stack)
            {
                return Vec::new();
            }

            for card in cards.zones.hand_cards(player_id) {
                let object_id = card.object_id();
                match Playability::new(cards, &player.mana, card) {
                    Some(Playability::WithoutTargeting(playable)) => {
                        actions.push(LegalAction::Play(playable));
                    }
                    Some(Playability::FromTargeting(_))
                        if can_start_targeting(cards, object_id) =>
                    {
                        actions.push(LegalAction::StartTargeting {
                            object_id,
                            awaiting_nothing: WasAwaitingNothing::from(awaits_nothing),
                        });
                    }
                    Some(Playability::FromTargeting(_)) | None => {}
                }
            }

            actions.push(LegalAction::EndTurn);
        }
        Awaiting::Picking(awaits_picking) => {
            actions.extend(awaits_picking.options().map(LegalAction::Pick));
        }
        Awaiting::Targeting(awaits_targeting) => {
            let targeting = awaits_targeting.targeting();
            let object_id = targeting.object_id();
            actions.extend(next_candidates(cards, targeting).into_iter().map(
                |(target_id, object_id)| LegalAction::Target {
                    awaiting_targeting: WasAwaitingTargeting::from(awaits_targeting),
                    target_id,
                    object_id,
                },
            ));
            if may_stop_targeting(cards, targeting)
                && let Some(Playability::FromTargeting(playable)) = cards
                    .zones
                    .find_in_hand(player_id, object_id)
                    .and_then(|card| Playability::new(cards, &player.mana, card))
            {
                actions.push(LegalAction::Play(
                    playable.with_targeting(WasAwaitingTargeting::from(awaits_targeting)),
                ));
            }
            actions.push(LegalAction::Cancel(WasAwaitingTargeting::from(
                awaits_targeting,
            )));
        }
    }
    actions
}
