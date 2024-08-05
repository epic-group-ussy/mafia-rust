
use serde::Serialize;

use crate::game::chat::ChatMessageVariant;
use crate::game::resolution_state::ResolutionState;
use crate::game::grave::{Grave, GraveDeathCause, GraveInformation, GraveKiller, GraveReference};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::player_group::PlayerGroup;
use crate::game::role_list::Faction;
use crate::game::visit::Visit;

use crate::game::Game;
use super::{Priority, RoleStateImpl, Role, RoleState};




#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deputy {
    bullets_remaining: u8,
}
impl Default for Deputy {
    fn default() -> Self {
        Self { bullets_remaining: 1 }
    }
}

pub(super) const FACTION: Faction = Faction::Town;
pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: u8 = 0;

impl RoleStateImpl for Deputy {
    fn do_night_action(self, _game: &mut Game, _actor_ref: PlayerReference, _priority: Priority) {

    }
    fn do_day_action(self, game: &mut Game, actor_ref: PlayerReference, target_ref: PlayerReference) {

        target_ref.add_private_chat_message(game, ChatMessageVariant::DeputyShotYou);
        if target_ref.defense(game) >= 1 {
            target_ref.add_private_chat_message(game, ChatMessageVariant::YouSurvivedAttack);
            actor_ref.add_private_chat_message(game, ChatMessageVariant::SomeoneSurvivedYourAttack);

        }else{
            game.add_message(PlayerGroup::All, ChatMessageVariant::DeputyKilled{shot_index: target_ref.index()});
            
            
            let mut grave = Grave::from_player_lynch(game, target_ref);
            if let GraveInformation::Normal{death_cause, ..} = &mut grave.information {
                *death_cause = GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Deputy)]);
            }
            target_ref.die(game, grave);
            

            if ResolutionState::requires_only_this_resolution_state(game, target_ref, ResolutionState::Town) {
                actor_ref.die(game, Grave::from_player_leave_town(game, actor_ref));
            }
        }

        actor_ref.set_role_state(game, RoleState::Deputy(Deputy{bullets_remaining:self.bullets_remaining-1}));
    }
    fn can_select(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }
    fn can_day_target(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        game.current_phase().is_day() &&
        game.phase_machine.day_number > 1 &&
        self.bullets_remaining > 0 &&
        actor_ref != target_ref &&
        target_ref.alive(game) && actor_ref.alive(game) &&
        (PhaseType::Discussion == game.current_phase().phase() || PhaseType::Nomination == game.current_phase().phase())
    }
    fn convert_selection_to_visits(self, _game: &Game, _actor_ref: PlayerReference, _target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        vec![]
    }
    fn get_current_send_chat_groups(self,  game: &Game, actor_ref: PlayerReference) -> Vec<PlayerGroup> {
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, vec![])
    }
    fn get_current_receive_chat_groups(self,  game: &Game, actor_ref: PlayerReference) -> Vec<PlayerGroup> {
        crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref)
    }
    fn get_won_game(self, game: &Game, actor_ref: PlayerReference) -> bool {
        crate::game::role::common_role::get_won_game(game, actor_ref)
    }
    fn on_phase_start(self, _game: &mut Game, _actor_ref: PlayerReference, _phase: PhaseType) {

    }
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference) {
        
    }
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
    }
    fn on_grave_added(self, _game: &mut Game, _actor_ref: PlayerReference, _grave_ref: GraveReference){
    }
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
}