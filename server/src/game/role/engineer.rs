use serde::Serialize;

use crate::game::chat::ChatMessageVariant;
use crate::game::grave::GraveReference;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::player_group::PlayerGroup;
use crate::game::role_list::Faction;
use crate::game::visit::Visit;

use crate::game::Game;
use super::{Priority, Role, RoleState, RoleStateImpl};

#[derive(Default, Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Engineer {
    pub trap: Trap
}
#[derive(Default, Clone, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Trap {
    #[default]
    Dismantled,
    Ready,
    #[serde(rename_all = "camelCase")]
    Set{target: PlayerReference, should_unset: bool}
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
pub(super) const DEFENSE: u8 = 0;

impl RoleStateImpl for Engineer {
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        match priority {
            Priority::Heal => {
                //upgrade state

                if !actor_ref.night_blocked(game) {
                    match self.trap {
                        Trap::Dismantled => {
                            actor_ref.set_role_state(game, RoleState::Engineer(Engineer {trap: Trap::Ready}));
                        },
                        Trap::Ready => {
                            if let Some(visit) = actor_ref.night_visits(game).first(){
                                actor_ref.set_role_state(game, RoleState::Engineer(Engineer {trap: Trap::Set{target: visit.target, should_unset: false}}));
                            }
                        },
                        Trap::Set { should_unset: true, .. } => {
                            actor_ref.set_role_state(game, RoleState::Engineer(Engineer {trap: Trap::Ready}));
                        },
                        _ => {}
                    }
                }
    
                if let RoleState::Engineer(Engineer{trap: Trap::Set{target, ..}}) = actor_ref.role_state(game).clone(){
                    target.increase_defense_to(game, 2);
                }
            }
            Priority::Kill => {
                if let Trap::Set { target, .. } = self.trap {
                    for attacker in PlayerReference::all_players(game) {
                        if 
                            attacker.night_visits(game).iter().any(|visit| visit.target == target && visit.attack) &&
                            attacker != actor_ref
                        {
                            attacker.try_night_kill(actor_ref, game, crate::game::grave::GraveKiller::Role(Role::Engineer), 2, false);
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
                        actor_ref.set_role_state(game, RoleState::Engineer(Engineer {trap: Trap::Dismantled}));
                    }
                }
            }
            _ => {}
        }
    }
    fn can_select(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        (match self.trap {
            Trap::Dismantled => false,
            Trap::Ready => actor_ref != target_ref,
            Trap::Set { .. } => false,
        }) &&
        !actor_ref.night_jailed(game) &&
        actor_ref.selection(game).is_empty() &&
        actor_ref.alive(game) &&
        target_ref.alive(game)
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
    fn get_won_game(self, game: &Game, actor_ref: PlayerReference) -> bool {
        crate::game::role::common_role::get_won_game(game, actor_ref)
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {
                if actor_ref.alive(game) {
                    actor_ref.add_private_chat_message(game, ChatMessageVariant::TrapState { state: self.trap.state() });
                }
            }
            _ => {}
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