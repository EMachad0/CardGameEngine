use crate::action::Action;
use crate::game::cards::Cards;
use crate::game::player::Player;
use crate::game::player_input::{Awaiting, PlayerInput};
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::stats::cost;
use crate::game::targeting_walk::{can_start_targeting, may_stop_targeting, next_candidates};

/// The one definition of legality.
pub(super) fn legal_actions(
    cards: Cards<'_>,
    standings: &Standings,
    player_input: &PlayerInput,
    player: &Player,
) -> Vec<Action> {
    let player_id = player.id;
    if standings.outcome().is_some() {
        return Vec::new();
    }

    let mut actions = Vec::new();
    match player_input.awaiting(player_id) {
        None => {
            if cards.turn_order.get_active_player_id() != player_id || player_input.any_awaiting() {
                return Vec::new();
            }

            for card in cards.zones.hand_cards(player_id) {
                let object_id = card.object_id();
                if !player.mana.can_pay(cost(cards, card)) {
                    continue;
                }

                match cards.card_def(object_id).targets.is_empty() {
                    true => {
                        actions.push(Action::Play { object_id });
                    }
                    false => {
                        if can_start_targeting(cards, object_id) {
                            actions.push(Action::StartTargeting { object_id });
                        }
                    }
                }
            }

            actions.push(Action::EndTurn);
        }
        Some(Awaiting::Pick(pick)) => {
            actions.extend(
                pick.options()
                    .iter()
                    .cloned()
                    .map(|object_id| Action::Pick { object_id }),
            );
        }
        Some(Awaiting::Targets(targeting)) => {
            let object_id = targeting.object_id();
            actions.extend(next_candidates(cards, targeting).into_iter().map(
                |(target_id, object_id)| Action::Target {
                    target_id,
                    object_id,
                },
            ));
            if may_stop_targeting(cards, targeting) {
                actions.push(Action::Play { object_id });
            }
            actions.push(Action::Cancel { object_id });
        }
    }
    actions
}
