
use serde::Serialize;

use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::visit::Visit;

use crate::game::Game;
use super::{Priority, RoleState, RoleStateImpl};

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Doctor {
    self_heals_remaining: u8,
    target_healed_ref: Option<PlayerReference>
}

impl Default for Doctor {
    fn default() -> Self {
        Self { 
            self_heals_remaining: 1,
            target_healed_ref: None
        }
    }
}

pub(super) const FACTION: Faction = Faction::Town;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;

impl RoleStateImpl for Doctor {
    fn defense(&self, _game: &Game, _actor_ref: PlayerReference) -> u8 {0}
    


    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        match priority {
            Priority::TopPriority => {
                actor_ref.set_role_state(game, RoleState::Doctor(
                    Doctor {
                        self_heals_remaining: self.self_heals_remaining, 
                        target_healed_ref: None
                    }
                ));
            }
            Priority::Heal => {
                let Some(visit) = actor_ref.night_visits(game).first() else {return};
                let target_ref = visit.target;
                if target_ref.night_jailed(game){
                    actor_ref.push_night_message(game, ChatMessageVariant::TargetJailed);
                    return
                }

                target_ref.increase_defense_to(game, 2);

                if actor_ref == target_ref{
                    actor_ref.set_role_state(game, RoleState::Doctor(Doctor{
                        self_heals_remaining: self.self_heals_remaining.saturating_sub(1), 
                        target_healed_ref: Some(target_ref)
                    }));
                }else{
                    actor_ref.set_role_state(game, RoleState::Doctor(Doctor{
                        target_healed_ref: Some(target_ref),
                        ..self
                    }));
                }

            }
            Priority::Investigative => {
                if let Some(target_healed_ref) = self.target_healed_ref {
                    if target_healed_ref.night_attacked(game){
                        
                        actor_ref.push_night_message(game, ChatMessageVariant::TargetWasAttacked);
                        target_healed_ref.push_night_message(game, ChatMessageVariant::YouWereProtected);
                    }
                }
            }
            _ => {}
        }
    }
    fn can_night_target(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        ((actor_ref == target_ref && self.self_heals_remaining > 0) || actor_ref != target_ref) &&
        !actor_ref.night_jailed(game) &&
        actor_ref.chosen_targets(game).is_empty() &&
        actor_ref.alive(game) &&
        target_ref.alive(game)
    }
    fn do_day_action(self, _game: &mut Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) {
        
    }
    fn can_day_target(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }
    fn convert_targets_to_visits(self, game: &Game, actor_ref: PlayerReference, target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        crate::game::role::common_role::convert_targets_to_visits(game, actor_ref, target_refs, false)
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
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.set_role_state(game, RoleState::Doctor(Doctor {self_heals_remaining: self.self_heals_remaining, target_healed_ref: None}));
    }
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference){
        
    }
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
    }
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
}