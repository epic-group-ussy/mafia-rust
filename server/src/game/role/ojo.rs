use serde::{Deserialize, Serialize};

use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::visit::Visit;
use crate::game::Game;
use super::{Priority, RoleStateImpl, Role};


#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Ojo{
    pub chosen_action: OjoAction,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum OjoAction {
    Kill{role: Role},
    See{role: Role},
    #[default]
    None
}

pub(super) const FACTION: Faction = Faction::Neutral;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;

impl RoleStateImpl for Ojo {
    fn defense(&self, _game: &Game, _actor_ref: PlayerReference) -> u8 {1}

    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        
        match priority {
            Priority::TopPriority => {
                if !actor_ref.alive(game) {return;}

                let (chosen_role, attack) = match self.chosen_action {
                    OjoAction::Kill{role} => (role, game.day_number() > 1),
                    OjoAction::See{role} => (role, false),
                    OjoAction::None => return,
                    
                };

                actor_ref.set_night_visits(game, 
                    PlayerReference::all_players(game)
                    .filter_map(|player|
                        if 
                            player.alive(game) && 
                            player != actor_ref &&
                            player.role(game) == chosen_role
                        {
                            Some(Visit {
                                target: player,
                                attack
                            })
                        } else {
                            None
                        }
                    )
                    .collect()
                );
            }
            Priority::Kill => {
                if game.day_number() == 1 {return;}
                if let OjoAction::Kill{..} = self.chosen_action {
                    for player in 
                        actor_ref.night_visits(game)
                            .iter()
                            .map(|visit| visit.target)
                            .collect::<Vec<PlayerReference>>()
                    {
                        player.try_night_kill(actor_ref, game, GraveKiller::Role(Role::Ojo), 2, true);
                    }
                }
            },
            Priority::Investigative => {
                if let OjoAction::See{..} = self.chosen_action {
                    let players = actor_ref.night_visits(game)
                    .iter()
                    .map(|visit| visit.target)
                    .collect::<Vec<PlayerReference>>();
                
                    actor_ref.push_night_message(game, 
                        ChatMessageVariant::OjoResult{players: PlayerReference::ref_vec_to_index(&players)}
                    );
                }
            },
            _ => {}
        }
    }
    fn can_night_target(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }
    fn do_day_action(self, _game: &mut Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) {
    }
    fn can_day_target(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }
    fn convert_targets_to_visits(self, _game: &Game, _actor_ref: PlayerReference, _target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        vec![]
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
    fn on_phase_start(self, _game: &mut Game, _actor_ref: PlayerReference, _phase: PhaseType){
    }
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
    }
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference){
    }
}