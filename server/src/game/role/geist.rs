use serde::Serialize;

use crate::game::chat::ChatGroup;

use crate::game::components::night_visits::NightVisits;
use crate::game::grave::Grave;

use crate::game::attack_power::AttackPower;
use crate::game::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::GameConclusion;
use crate::game::Game;

use super::{common_role, ControllerID, ControllerParametersMap, Priority, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Geist {
    won: bool,
}

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Geist {
    type ClientRoleState = Geist;
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        let transporter_visits = actor_ref.untagged_night_visits_cloned(game).clone();
        let Some(first_visit) = transporter_visits.get(0) else {return};
        if priority == Priority::Geist {
            
            first_visit.target.push_night_message(game, ChatMessageVariant::Transported);

            for player_ref in PlayerReference::all_players(game){
                if player_ref == actor_ref {continue;}
                if player_ref.role(game) == Role::Geist {continue;}
                if player_ref.role(game) == Role::Warper {continue;}
                if player_ref.role(game) == Role::Transporter {continue;}

                let new_visits = player_ref.all_night_visits_cloned(game).clone().into_iter().map(|mut v|{
                    if v.target == first_visit.target {
                        v.target = actor_ref;
                    }else if v.target == actor_ref{
                        v.target = first_visit.target;
                    }
                    v
                }).collect();
                player_ref.set_night_visits(game, new_visits);
            
            }
        }
        if priority == Priority::Heal {
            actor_ref.increase_defense_to(game, DefensePower::Protection);
            first_visit.target.increase_defense_to(game, DefensePower::Protection);
            
        }

        if priority == Priority::Kill{
            let mut all_attackers: Vec<PlayerReference> = vec![];
            for visit in NightVisits::all_visits(game).into_iter().cloned().collect::<Vec<_>>() {
                if 
                    visit.attack &&
                    visit.target == actor_ref &&
                    visit.visitor != actor_ref
                {
                    all_attackers.push(visit.visitor);
                }
            }
            if all_attackers.len() > 0 {
                for attacker_ref in all_attackers {
                    if attacker_ref.win_condition(game).friends_with_resolution_state(GameConclusion::Town) { 
                        attacker_ref.try_night_kill_single_attacker(actor_ref, game, GraveKiller::Role(Role::Geist), AttackPower::ProtectionPiercing, true);
                    }              
                    actor_ref.set_role_state(game, Geist { won: true});     
                }
            }
        }

        if priority == Priority::Investigative {
            if let Some(target_healed_ref) = Some(first_visit.target) {
                if target_healed_ref.night_attacked(game){
                    actor_ref.push_night_message(game, ChatMessageVariant::TargetWasAttacked);
                    target_healed_ref.push_night_message(game, ChatMessageVariant::YouWereProtected);
                }
            }
        }
    }

    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        match phase {
            PhaseType::Obituary => {
                if self.won {
                    actor_ref.die(game, Grave::from_player_leave_town(game, actor_ref));
                    game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::GeistWon);
                }
            }
            _ => {}
        }
    }


    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        crate::game::role::common_role::controller_parameters_map_player_list_night_typical(
            game,
            actor_ref,
            false,
            false,
            ControllerID::role(actor_ref, Role::Geist, 0)
        )
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference, _target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Geist, 0),
            false
        )
    }

}

impl Geist {
    pub fn won(&self) -> bool {
        return self.won;
    }
}