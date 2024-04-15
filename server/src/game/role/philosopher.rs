use serde::Serialize;

use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::end_game_condition::EndGameCondition;
use crate::game::visit::Visit;
use crate::game::Game;

use super::{Priority, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Philosopher;

pub(super) const FACTION: Faction = Faction::Town;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;

impl RoleStateImpl for Philosopher {
    fn defense(&self, _game: &Game, _actor_ref: PlayerReference) -> u8 {0}
    


    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if priority != Priority::Investigative {return;}

        let Some(first_visit) = actor_ref.night_visits(game).get(0) else {return;};
        let Some(second_visit) = actor_ref.night_visits(game).get(1) else {return;};

            
        if first_visit.target.night_jailed(game) || second_visit.target.night_jailed(game){
            actor_ref.push_night_message(game, ChatMessageVariant::TargetJailed);
            return
        }

        let message = ChatMessageVariant::SeerResult{
            enemies: Philosopher::players_are_enemies(game, first_visit.target, second_visit.target)
        };
        
        actor_ref.push_night_message(game, message);
    }
    fn do_day_action(self, _game: &mut Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) {}
    fn can_night_target(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        actor_ref != target_ref &&
        !actor_ref.night_jailed(game) &&
        actor_ref.alive(game) &&
        target_ref.alive(game) &&
        (
            actor_ref.chosen_targets(game).is_empty() || 
            actor_ref.chosen_targets(game).len() == 1 && *actor_ref.chosen_targets(game).get(0).unwrap() != target_ref
        )
    }
    fn can_day_target(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }
    fn convert_targets_to_visits(self, _game: &Game, _actor_ref: PlayerReference, target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        if target_refs.len() == 2 {
            vec![
                Visit{ target: target_refs[0], attack:false },
                Visit{ target: target_refs[1], attack:false }
            ]
        } else {
            Vec::new()
        }
    }
    fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> Vec<ChatGroup> {
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, vec![])
    }
    fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> Vec<ChatGroup> {
        crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref)
    }
    fn get_won_game(self, game: &Game, actor_ref: PlayerReference) -> bool {
        crate::game::role::common_role::get_won_game(game, actor_ref)
    }
    fn on_phase_start(self, _game: &mut Game, _actor_ref: PlayerReference, _phase: PhaseType) {}
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference) {
        
    }
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
    }
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
}
impl Philosopher{
    pub fn players_are_enemies(game: &Game, a: PlayerReference, b: PlayerReference) -> bool {
        if a.has_suspicious_aura(game) || b.has_suspicious_aura(game){
            true
        }else if a.has_innocent_aura(game) || b.has_innocent_aura(game){
            false
        }else{
            !EndGameCondition::can_win_together(
                a.role(game).end_game_condition(), 
                b.role(game).end_game_condition()
            )
        }
    }
}