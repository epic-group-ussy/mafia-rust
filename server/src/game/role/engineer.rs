use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::grave::GraveKiller;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::visit::Visit;

use crate::game::Game;
use super::{GetClientRoleState, Priority, Role, RoleState, RoleStateImpl};

#[derive(Default, Clone, Debug)]
pub struct Engineer {
    pub trap: Trap,
    pub night_selection: <Self as RoleStateImpl>::RoleActionChoice,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState {
    trap: ClientTrapState
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum ClientTrapState {
    Dismantled,
    Ready,
    Set
}

#[derive(Default, Clone, Debug)]
pub enum Trap {
    #[default]
    Dismantled,
    Ready,
    Set{target: PlayerReference}
}

impl Trap {
    fn state(&self) -> TrapState {
        match self {
            Trap::Dismantled => TrapState::Dismantled,
            Trap::Ready => TrapState::Ready,
            Trap::Set{..} => TrapState::Set
        }
    }
}
#[derive(Default, Clone, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum TrapState {
    #[default]
    Dismantled,
    Ready,
    Set
}

//engineer prioritys
//tell player state

//Set trap / ready up / choose to unset and bring to ready
//protect, kill & investigate, dismantle


pub(super) const FACTION: Faction = Faction::Town;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Engineer {
    type ClientRoleState = ClientRoleState;
    type RoleActionChoice = super::common_role::RoleActionChoiceOnePlayer;
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        match priority {
            Priority::Heal => {
                //upgrade state

                if !actor_ref.night_blocked(game) {
                    match self.trap {
                        Trap::Dismantled => {
                            actor_ref.set_role_state(game, Engineer {trap: Trap::Ready, ..self});
                        },
                        Trap::Ready => {
                            if let Some(visit) = actor_ref.night_visits(game).first(){
                                actor_ref.set_role_state(game, Engineer {trap: Trap::Set{target: visit.target}, ..self});
                            }
                        },
                        Trap::Set { .. } if actor_ref.night_visits(game).first().is_some() => {
                            actor_ref.set_role_state(game, Engineer {trap: Trap::Ready, ..self});
                        },
                        _ => {}
                    }
                }
    
                if let RoleState::Engineer(Engineer{trap: Trap::Set{target, ..}, ..}) = actor_ref.role_state(game).clone(){
                    target.increase_defense_to(game, DefensePower::Protection);
                }
            }
            Priority::Kill => {
                if let Trap::Set { target, .. } = self.trap {
                    for attacker in PlayerReference::all_players(game) {
                        if 
                            attacker.night_visits(game).iter().any(|visit| visit.target == target && visit.attack) &&
                            attacker != actor_ref
                        {
                            attacker.try_night_kill_single_attacker(actor_ref, game, GraveKiller::Role(Role::Engineer), AttackPower::ArmorPiercing, false);
                        }
                    }
                }
            }
            Priority::Investigative => {
                if let Trap::Set { target, .. } = self.trap {

                    let mut should_dismantle = false;

                    if target.night_attacked(game){
                        actor_ref.push_night_message(game, ChatMessageVariant::TargetWasAttacked);
                        target.push_night_message(game, ChatMessageVariant::YouWereProtected);
                    }

                    for visitor in PlayerReference::all_players(game) {
                        if 
                            visitor.night_visits(game).iter().any(|visit|visit.target == target) &&
                            visitor != actor_ref
                        {
                            actor_ref.push_night_message(game, ChatMessageVariant::EngineerVisitorsRole { role: visitor.role(game) });
                            should_dismantle = true;
                        }
                    }

                    if should_dismantle {
                        actor_ref.set_role_state(game, Engineer {trap: Trap::Dismantled, ..self});
                    }
                }

                actor_ref.push_night_message(game, ChatMessageVariant::TrapStateEndOfNight { state: self.trap.state() });
            }
            _ => {}
        }
    }
    fn on_role_action(mut self, game: &mut Game, actor_ref: PlayerReference, action_choice: Self::RoleActionChoice) {
        if game.current_phase().phase() != crate::game::phase::PhaseType::Night {return};
        let Some(target_ref) = action_choice.player else {
            self.night_selection = action_choice;
            actor_ref.set_role_state(game, self);
            return;
        };

        if !(
            (match self.trap {
            Trap::Dismantled => false,
            Trap::Ready => actor_ref != target_ref,
            Trap::Set { .. } => actor_ref == target_ref,
            }) &&
            crate::game::role::common_role::default_action_choice_boolean_is_valid(game, actor_ref)
        ){
            return;
        }

        self.night_selection = action_choice;
        actor_ref.set_role_state(game, self);
    }
    fn create_visits(self, _game: &Game, _actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_action_choice_to_visits(self.night_selection.player, false)
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {
                actor_ref.add_private_chat_message(game, ChatMessageVariant::TrapState { state: self.trap.state().clone() });
            }
            _ => {}
        }

        crate::on_phase_start_reset_night_selection!(self, game, actor_ref, phase);
    }

}
impl GetClientRoleState<ClientRoleState> for Engineer {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            trap: match self.trap {
                Trap::Dismantled => ClientTrapState::Dismantled,
                Trap::Ready => ClientTrapState::Ready,
                Trap::Set {..} => ClientTrapState::Set,
            }
        }
    }
}