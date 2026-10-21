use crate::action::Action;
use crate::cards::object::ObjectId;
use crate::game::cards_ref::CardsRef;
use crate::game::play::Playable;
use crate::game::player::Player;
use crate::game::player_input::{Awaiting, NothingAwaited, PickOption, PlayerInput, TargetingSlot};
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::stats::cost;
use crate::game::targeting_walk::{can_start_targeting, may_stop_targeting, next_candidates};
use crate::target::TargetId;
use crate::zones::InHand;

/// An action legal for one player, with what the check learned about the game. What it learned
/// holds until the game changes.
pub(super) enum LegalAction {
    /// `targeting` is the player's targeting of the card, if they played it from there.
    Play {
        card: Playable,
        targeting: Option<TargetingSlot>,
    },
    Pick(PickOption),
    StartTargeting {
        object_id: ObjectId,
        nothing_awaited: NothingAwaited,
    },
    Target {
        targeting: TargetingSlot,
        target_id: TargetId,
        object_id: ObjectId,
    },
    Cancel {
        targeting: TargetingSlot,
        object_id: ObjectId,
    },
    EndTurn,
}

impl LegalAction {
    /// What a player sends for this action.
    pub(super) fn action(&self) -> Action {
        match *self {
            Self::Play { ref card, .. } => Action::Play {
                object_id: card.object_id(),
            },
            Self::Pick(ref option) => Action::Pick {
                object_id: option.object_id(),
            },
            Self::StartTargeting { object_id, .. } => Action::StartTargeting { object_id },
            Self::Target {
                target_id,
                object_id,
                ..
            } => Action::Target {
                target_id,
                object_id,
            },
            Self::Cancel { object_id, .. } => Action::Cancel { object_id },
            Self::EndTurn => Action::EndTurn,
        }
    }
}

/// The one definition of legality.
pub(super) fn legal_actions(
    cards: CardsRef,
    standings: &Standings,
    player_input: &PlayerInput,
    player: &Player,
) -> Vec<LegalAction> {
    let player_id = player.id;
    if standings.outcome().is_some() {
        return Vec::new();
    }

    let mut actions = Vec::new();
    match player_input.awaiting(player_id) {
        None => {
            if cards.turn_order.get_active_player_id() != player_id {
                return Vec::new();
            }
            let Some(nothing_awaited) = player_input.nothing_awaited() else {
                return Vec::new();
            };

            for card in cards.zones.hand_cards(player_id) {
                let object_id = card.object_id();
                let Some(playable) = playable(cards, player, card) else {
                    continue;
                };

                match cards.card_def(object_id).targets.is_empty() {
                    true => {
                        actions.push(LegalAction::Play {
                            card: playable,
                            targeting: None,
                        });
                    }
                    false => {
                        if can_start_targeting(cards, object_id) {
                            actions.push(LegalAction::StartTargeting {
                                object_id,
                                nothing_awaited,
                            });
                        }
                    }
                }
            }

            actions.push(LegalAction::EndTurn);
        }
        Some(Awaiting::Pick(pick)) => {
            actions.extend(pick.options().map(LegalAction::Pick));
        }
        Some(Awaiting::Targets(awaited)) => {
            let targeting = awaited.targeting();
            let object_id = targeting.object_id();
            actions.extend(next_candidates(cards, targeting).into_iter().map(
                |(target_id, object_id)| LegalAction::Target {
                    targeting: awaited.slot(),
                    target_id,
                    object_id,
                },
            ));
            if may_stop_targeting(cards, targeting)
                && let Some(card) = cards
                    .zones
                    .find_in_hand(player_id, object_id)
                    .and_then(|card| playable(cards, player, card))
            {
                actions.push(LegalAction::Play {
                    card,
                    targeting: Some(awaited.slot()),
                });
            }
            actions.push(LegalAction::Cancel {
                targeting: awaited.slot(),
                object_id,
            });
        }
    }
    actions
}

/// The card, if its player can pay its cost now.
fn playable(cards: CardsRef, player: &Player, card: InHand) -> Option<Playable> {
    let payment = player.mana.payment(cost(cards, card))?;
    Some(Playable::new(card.slot(), payment))
}
