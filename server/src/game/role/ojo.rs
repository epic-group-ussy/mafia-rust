use serde::{Deserialize, Serialize};

use crate::game::attack_power::DefensePower;
use crate::game::{attack_power::AttackPower, grave::GraveKiller};
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

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum OjoAction {
    Kill{role: Role},
    See{role: Role},
    #[default]
    None
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleActionChoice{
    action: OjoAction
}

pub(super) const FACTION: Faction = Faction::Fiends;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armor;

impl RoleStateImpl for Ojo {
    type ClientRoleState = Ojo;
    type RoleActionChoice = RoleActionChoice;
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
                            player != actor_ref &&
                            player.role(game) == chosen_role &&
                            player.alive(game)
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
                        player.try_night_kill_single_attacker(actor_ref, game, GraveKiller::Role(Role::Ojo), AttackPower::ArmorPiercing, true);
                    }
                }
            },
            Priority::Investigative => {
                if let OjoAction::See{..} = self.chosen_action {
                    let i_visited = actor_ref.night_visits(game)
                        .iter()
                        .map(|visit| visit.target)
                        .collect::<Vec<PlayerReference>>();

                    let visited_me = actor_ref.all_visitors(game);

                    for player in PlayerReference::all_players(game) {
                        if i_visited.contains(&player) || visited_me.contains(&player) {
                            actor_ref.insert_role_label(game, player);
                        }
                    }
                }
            },
            _ => {}
        }
    }
    fn on_role_action(mut self, game: &mut Game, actor_ref: PlayerReference, action_choice: Self::RoleActionChoice) {
        if game.current_phase().phase() != crate::game::phase::PhaseType::Night {return};

        self.chosen_action = match action_choice.action {
            OjoAction::Kill { .. } => {
                if game.day_number() == 1 {return;}
                action_choice.action
            },
            _ => action_choice.action
        };
        
        actor_ref.set_role_state(game, self);
    }
}