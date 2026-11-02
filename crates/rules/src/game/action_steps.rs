use crate::game::legal_actions::LegalAction;
use crate::game::state::{Acting, ActionStep};
use crate::observer::Observer;

pub(super) fn take_action(mut game: Acting<'_, impl Observer>, legal_action: LegalAction) {
    match legal_action {
        LegalAction::Play(playable) => {
            game.play(playable);
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
