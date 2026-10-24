use crate::game::state::Resolving;
use crate::observer::Observer;

pub(in crate::game) fn resolve_effect_stack(mut game: Resolving<'_, impl Observer>) {
    while !game.any_awaiting() && !game.is_over() {
        continue_resolving_effect(&mut game);

        if !game.is_resolving() {
            game.check_state();
            game.start_next();
        }

        if game.is_idle() {
            break;
        }
    }
}

fn continue_resolving_effect(game: &mut Resolving<'_, impl Observer>) {
    while !game.any_awaiting() && game.resolve_next_effect() {}
    game.finish_if_done();
}
