
use serde::Serialize;

use crate::game::chat::ChatMessageVariant;
use crate::game::chat::RecipientLike;
use crate::game::player_group::PlayerGroup;
use crate::game::grave::GraveInformation;
use crate::game::grave::GraveReference;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::tag::Tag;
use crate::game::visit::Visit;

use crate::game::Game;
use super::{Priority, RoleState, RoleStateImpl};


#[derive(Default, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mortician {
    obscured_players: Vec<PlayerReference>
}

pub(super) const FACTION: Faction = Faction::Mafia;
pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: u8 = 0;

const MAX_CREMATIONS: u8 = 3;

impl RoleStateImpl for Mortician {
    fn do_night_action(mut self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if actor_ref.night_jailed(game) {return}

        if self.obscured_players.len() as u8 >= MAX_CREMATIONS {return}

        match priority {
            Priority::Deception=>{
                let Some(visit) = actor_ref.night_visits(game).first() else{return};

                let target_ref = visit.target;
                
                if !self.obscured_players.contains(&target_ref){
                    self.obscured_players.push(target_ref);
                    actor_ref.set_role_state(game, RoleState::Mortician(self));
                    actor_ref.push_player_tag(game, target_ref, Tag::MorticianTagged);
                }
            },
            _ => {}
        }
    }
    fn can_select(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        actor_ref != target_ref &&
        !actor_ref.night_jailed(game) &&
        actor_ref.selection(game).is_empty() &&
        actor_ref.alive(game) &&
        target_ref.alive(game) &&
        (self.obscured_players.len() as u8) < MAX_CREMATIONS && 
        !self.obscured_players.contains(&target_ref)
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
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, vec![PlayerGroup::Mafia])
    }
    fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> Vec<PlayerGroup> {
        crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref)
    }
    fn get_won_game(self, game: &Game, actor_ref: PlayerReference) -> bool {
        crate::game::role::common_role::get_won_game(game, actor_ref)
    }
    fn on_phase_start(self, _game: &mut Game, _actor_ref: PlayerReference, _phase: PhaseType){
    }
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
    }
    fn on_grave_added(self, game: &mut Game, actor_ref: PlayerReference, grave_ref: GraveReference){
        if actor_ref.alive(game) && self.obscured_players.contains(&grave_ref.deref(game).player) {
            actor_ref.add_chat_message(game, ChatMessageVariant::PlayerRoleAndAlibi{
                player: grave_ref.deref(game).player,
                role: grave_ref.deref(game).player.role(game),
                will: grave_ref.deref(game).player.will(game).to_string(),
            });

            grave_ref.deref_mut(game).information = GraveInformation::Obscured;
        }
    }
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
}
