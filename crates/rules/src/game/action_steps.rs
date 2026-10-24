use crate::game::legal_actions::LegalAction;
use crate::game::state::{Acting, ActionStep};
use crate::observer::Observer;

/// A played minion enters the board in a step after its play, so it isn't on the board when its
/// own play is matched.
pub(super) fn take_action(mut game: Acting<'_, impl Observer>, legal_action: LegalAction) {
    match legal_action {
        LegalAction::Play(playable) => {
            let (player_id, object_id) = (playable.player_id(), playable.object_id());
            let enters_board = game.cards().card_def(object_id).has_board_presence();
            game.take_step(ActionStep::Play(playable));
            if enters_board {
                game.take_step(ActionStep::Summon {
                    player_id,
                    object_id,
                });
            }
        }
        LegalAction::Pick(awaiting_picking) => {
            game.take_step(ActionStep::Pick(awaiting_picking));
        }
        LegalAction::EndTurn => {
            game.take_step(ActionStep::EndTurn);
        }
        LegalAction::StartTargeting {
            object_id,
            awaiting_nothing,
        } => {
            game.take_step(ActionStep::StartTargeting {
                object_id,
                awaiting_nothing,
            });
        }
        LegalAction::Target {
            awaiting_targeting,
            target_id,
            object_id,
        } => {
            game.take_step(ActionStep::Target {
                awaiting_targeting,
                target_id,
                object_id,
            });
        }
        LegalAction::Cancel(awaiting_targeting) => {
            game.take_step(ActionStep::Cancel(awaiting_targeting));
        }
    }
}
