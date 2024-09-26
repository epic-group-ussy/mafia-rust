use serde::{Deserialize, Serialize};

use crate::game::attack_power::AttackPower;
use crate::game::{attack_power::DefensePower, components::love_linked::LoveLinked};
use crate::game::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::visit::Visit;

use crate::game::Game;
use super::{same_evil_team, Priority, RoleStateImpl};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Eros{
    pub action: ErosAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ErosAction{
    #[default] LoveLink,
    Kill,
}

pub(super) const FACTION: Faction = Faction::Mafia;
pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::Armor;

impl RoleStateImpl for Eros {
    type ClientRoleState = Eros;
    type RoleActionChoice = super::common_role::CommonRoleActionChoice;
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        match (priority, self.action) {
            (Priority::Kill, ErosAction::Kill) => {
                if game.day_number() == 1 {return}
                if let Some(visit) = actor_ref.night_visits(game).first(){
                    let target_ref = visit.target;
            
                    target_ref.try_night_kill_single_attacker(
                        actor_ref, game, GraveKiller::Faction(Faction::Mafia), AttackPower::Basic, false
                    );
                }
            }
            (Priority::Cupid, ErosAction::LoveLink) => {
                let visits = actor_ref.night_visits(game);

                let Some(first_visit) = visits.get(0) else {return};
                let Some(second_visit) = visits.get(1) else {return};
                
                let player1 = first_visit.target;
                let player2 = second_visit.target;

                LoveLinked::add_love_link(game, player1, player2);
            },
            _ => ()
        }
    }
    fn can_select(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        let selected = actor_ref.selection(game);

        actor_ref != target_ref &&
        !actor_ref.night_jailed(game) &&
        actor_ref.alive(game) &&
        target_ref.alive(game) &&
        match self.action {
            ErosAction::LoveLink => {
                selected.len() < 2 &&
                selected.iter().all(|&p| p != target_ref)
            },
            ErosAction::Kill => {
                game.day_number() > 1 &&
                !same_evil_team(game, actor_ref, target_ref) &&
                selected.is_empty()
            },
        }
    }
    fn convert_selection_to_visits(self, _game: &Game, _actor_ref: PlayerReference, target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        match self.action {
            ErosAction::LoveLink => {
                if target_refs.len() == 2 {
                    vec![
                        Visit{ target: target_refs[0], attack: false },
                        Visit{ target: target_refs[1], attack: false }
                    ]
                } else {
                    Vec::new()
                }
            },
            ErosAction::Kill => {
                if !target_refs.is_empty() {
                    vec![
                        Visit{ target: target_refs[0], attack: true }
                    ]
                } else {
                    Vec::new()
                }
            }
        }
    }
}