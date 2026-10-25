use crate::game::effect_stack::Advance;
use crate::game::state::Resolving;
use crate::observer::Observer;

pub(in crate::game) fn resolve_effect_stack(mut game: Resolving<'_, impl Observer>) {
    while !game.any_targeting() && !game.is_over() {
        match game.advance() {
            Advance::Effect(next) => game.resolve_effect(next),
            Advance::FinishPicking(complete) => game.finish_picking(complete),
            Advance::AwaitPicking => break,
            Advance::Done => {
                game.check_state();
                game.start_next();
                if game.is_idle() {
                    break;
                }
            }
        }
    }
}
