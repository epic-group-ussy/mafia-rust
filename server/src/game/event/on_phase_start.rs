use crate::game::{phase::PhaseType, player::PlayerReference, Game};

#[must_use = "Event must be invoked"]
pub struct OnPhaseStart{
    phase: PhaseType
}
impl OnPhaseStart{
    pub fn new(phase: PhaseType) -> Self{
        Self{ phase }
    }
    pub fn invoke(self, game: &mut Game){
        for player_ref in PlayerReference::all_players(game){
            player_ref.on_phase_start(game, self.phase);
        }

        game.mafia().clone().on_phase_start(game, self.phase);
        game.cult().clone().on_phase_start(game, self.phase);

        game.on_phase_start(self.phase);
    }
}