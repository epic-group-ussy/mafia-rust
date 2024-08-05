use serde::Serialize;

use crate::game::chat::ChatMessageVariant;
use crate::game::grave::{Grave, GraveDeathCause, GraveInformation, GraveKiller, GraveReference};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::player_group::PlayerGroup;
use crate::game::role_list::Faction;

use crate::game::visit::Visit;
use crate::game::Game;
use super::{Priority, RoleStateImpl, RoleState, Role};


#[derive(Debug, Clone, Default, Serialize)]
pub struct Death{
    souls: u8,
    won: bool,
}
const NEEDED_SOULS: u8 = 6;
pub(super) const FACTION: Faction = Faction::Neutral;
pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: u8 = 0;

impl RoleStateImpl for Death {
    fn do_night_action(mut self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if priority == Priority::Heal && self.souls >= NEEDED_SOULS{
            actor_ref.set_night_upgraded_defense(game, Some(3))
        }

        if priority != Priority::Investigative {return;}
        if !actor_ref.alive(game) {return;}

        let mut souls_to_gain = 1;

        if !actor_ref.night_jailed(game) {
            if let Some(visit) = actor_ref.night_visits(game).first(){
                let target_ref = visit.target;
                if target_ref.night_died(game) {
                    souls_to_gain = 2
                }
            }
        }

        self.souls += souls_to_gain;
        if self.souls >= NEEDED_SOULS {
            game.add_message(PlayerGroup::All, ChatMessageVariant::DeathCollectedSouls);
        }
        actor_ref.set_role_state(game, RoleState::Death(self));
    }
    fn can_select(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        crate::game::role::common_role::can_night_select(game, actor_ref, target_ref)
    }
    fn do_day_action(self, _game: &mut Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) {
        
    }
    fn can_day_target(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference, target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        crate::game::role::common_role::convert_selection_to_visits(game, actor_ref, target_refs, false)
    }
    fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> Vec<PlayerGroup> {
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, vec![])
    }
    fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> Vec<PlayerGroup> {
        crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref)
    }
    fn get_won_game(self, _game: &Game, _actor_ref: PlayerReference) -> bool {
        self.won
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {
                if self.souls >= NEEDED_SOULS {
                    for player in PlayerReference::all_players(game){
                        if !player.alive(game){continue;}
                        if player.defense(game) >= 3 {
                            player.add_private_chat_message(game, ChatMessageVariant::YouSurvivedAttack);
                            actor_ref.add_private_chat_message(game, ChatMessageVariant::SomeoneSurvivedYourAttack);
                
                        }else{
                            let mut grave = Grave::from_player_lynch(game, player);
                            if let GraveInformation::Normal{ death_cause, .. } = &mut grave.information {
                                *death_cause = GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Death)]);
                            }
                            player.die(game, grave);
                            actor_ref.set_role_state(game, RoleState::Death(Death{won: true, souls: self.souls}));
                        }
                    }
                }
            },
            _=>{}
        }
        
    }
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference){
        
    }
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
    }
    fn on_grave_added(self, _game: &mut Game, _actor_ref: PlayerReference, _grave_ref: GraveReference){
    }
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
}
