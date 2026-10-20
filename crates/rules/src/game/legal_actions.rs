use crate::action::Action;
use crate::game::player_input::Awaiting;
use crate::game::state::Game;
use crate::ids::PlayerId;

impl Game {
    /// The one definition of legality.
    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        if self.outcome().is_some() {
            return Vec::new();
        }

        let cards = self.cards();
        let mut actions = Vec::new();
        match self.player_input.awaiting(player_id) {
            None => {
                if self.turn_order.get_active_player_id() != player_id
                    || self.player_input.any_awaiting()
                {
                    return Vec::new();
                }

                let mana = &self.get_player(player_id).mana;
                for card in self.zones.hand_cards(player_id) {
                    let object_id = card.object_id();
                    if !mana.can_pay(cards.cost(card)) {
                        continue;
                    }

                    match cards.card_def(object_id).targets.is_empty() {
                        true => {
                            actions.push(Action::Play { object_id });
                        }
                        false => {
                            if cards.can_start_targeting(object_id) {
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
                actions.extend(cards.next_candidates(targeting).into_iter().map(
                    |(target_id, object_id)| Action::Target {
                        target_id,
                        object_id,
                    },
                ));
                if cards.may_stop_targeting(targeting) {
                    actions.push(Action::Play { object_id });
                }
                actions.push(Action::Cancel { object_id });
            }
        }
        actions
    }
}
