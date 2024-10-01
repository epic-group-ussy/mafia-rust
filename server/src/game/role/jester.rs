
use rand::seq::SliceRandom;
use serde::Serialize;

use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::phase::{PhaseType, PhaseState};
use crate::game::player::PlayerReference;
use crate::game::role_list::Faction;
use crate::game::verdict::Verdict;

use crate::game::Game;
use super::{GetClientRoleState, Priority, RoleStateImpl};

#[derive(Clone, Debug, Default)]
pub struct Jester {
    lynched_yesterday: bool,
    won: bool,
    night_selection: <Self as RoleStateImpl>::RoleActionChoice
}

#[derive(Clone, Serialize, Debug)]
pub struct ClientRoleState;

pub(super) const FACTION: Faction = Faction::Neutral;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Jester {
    type ClientRoleState = ClientRoleState;
    type RoleActionChoice = super::common_role::RoleActionChoiceOnePlayer;
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if priority != Priority::TopPriority {return;}
        if actor_ref.alive(game) {return;}
        if !self.lynched_yesterday {return}
        
        let all_killable_players: Vec<PlayerReference> = PlayerReference::all_players(game)
            .filter(|player_ref|{
                player_ref.alive(game) &&
                *player_ref != actor_ref &&
                player_ref.verdict(game) != Verdict::Innocent
            }).collect();
    
        let player = match self.night_selection.player {
            Some(v) => v,
            None => {
                let Some(target_ref) = all_killable_players.choose(&mut rand::thread_rng()) else {return};
                *target_ref
            },
        };
        player.try_night_kill_single_attacker(actor_ref, game, 
            crate::game::grave::GraveKiller::Role(super::Role::Jester), AttackPower::ProtectionPiercing, true
        );
    }
    fn on_role_action(mut self, game: &mut Game, actor_ref: PlayerReference, action_choice: Self::RoleActionChoice) {

        let Some(target_ref) = action_choice.player else {
            self.night_selection = action_choice;
            actor_ref.set_role_state(game, self);
            return;
        };

        if !(
            actor_ref != target_ref &&
            !actor_ref.alive(game) &&
            target_ref.alive(game) &&
            target_ref.verdict(game) != Verdict::Innocent &&
            self.lynched_yesterday
        ){
            return;
        }

        self.night_selection = action_choice;
        actor_ref.set_role_state(game, self);
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        match game.current_phase() {
            &PhaseState::FinalWords { player_on_trial } => {
                if player_on_trial == actor_ref {
                    actor_ref.set_role_state(game, Jester { 
                        lynched_yesterday: true,
                        won: true,
                        night_selection: <Self as RoleStateImpl>::RoleActionChoice::default()
                    });
                }
            }
            PhaseState::Obituary => {
                actor_ref.set_role_state(game, Jester { 
                    lynched_yesterday: false,
                    won: self.won,
                    night_selection: <Self as RoleStateImpl>::RoleActionChoice::default()
                });
            }
            _ => {}
        }
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
        if 
            actor_ref == dead_player_ref && 
            game.current_phase().phase() == PhaseType::FinalWords
        {
            game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::JesterWon);
        }
    }
}
impl GetClientRoleState<ClientRoleState> for Jester {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl Jester {
    pub fn won(&self) -> bool {
        self.won
    }
}