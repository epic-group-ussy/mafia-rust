use serde::Serialize;

use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::resolution_state::ResolutionState;
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::visit::Visit;
use crate::game::Game;

use super::{Priority, RoleStateImpl};

pub(super) const FACTION: Faction = Faction::Town;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Detective;

impl RoleStateImpl for Detective {
    type ClientRoleState = Detective;
    type RoleActionChoice = super::common_role::RoleActionChoiceRole;
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if priority != Priority::Investigative {return;}

        if let Some(visit) = actor_ref.night_visits(game).first(){
            
            let message = ChatMessageVariant::SheriffResult {
                suspicious: Detective::player_is_suspicious(game, visit.target)
            };
            
            actor_ref.push_night_message(game, message);
        }
    }
    fn can_select(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        crate::game::role::common_role::can_night_select(game, actor_ref, target_ref)
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference, target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        crate::game::role::common_role::convert_selection_to_visits(game, actor_ref, target_refs, false)
    }
}

impl Detective {
    pub fn player_is_suspicious(game: &Game, player_ref: PlayerReference) -> bool {

        if player_ref.has_suspicious_aura(game){
            true
        }else if player_ref.has_innocent_aura(game){
            false
        }else{
            !player_ref.win_condition(game).can_win_when_resolution_state_reached(ResolutionState::Town)
        }
    }
}