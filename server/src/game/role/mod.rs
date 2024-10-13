#![allow(clippy::single_match)]
#![allow(clippy::get_first)]

use std::collections::HashSet;

use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::chat::ChatGroup;
use crate::game::phase::PhaseType;
use crate::game::attack_power::DefensePower;

use serde::{Serialize, Deserialize};

use super::{event::before_role_switch::BeforeRoleSwitch, grave::GraveReference, win_condition::WinCondition};

pub trait GetClientRoleState<CRS> {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> CRS;
}
//Automatically implement this for the case where RoleState = ClientRoleState
impl<T> GetClientRoleState<T> for T {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> T {
        self
    }
}

pub trait RoleStateImpl: Clone + std::fmt::Debug + Default + GetClientRoleState<<Self as RoleStateImpl>::ClientRoleState> {
    type ClientRoleState: Clone + std::fmt::Debug + Serialize;
    fn do_night_action(self, _game: &mut Game, _actor_ref: PlayerReference, _priority: Priority) {}
    fn do_day_action(self, _game: &mut Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) {}

    fn can_select(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }
    fn can_day_target(self, _game: &Game, _actor_ref: PlayerReference, _target_ref: PlayerReference) -> bool {
        false
    }

    fn convert_selection_to_visits(self, _game: &Game, _actor_ref: PlayerReference, _target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        vec![]
    }

    fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, vec![])
    }
    fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref)
    }
    fn default_win_condition(self) -> WinCondition where RoleState: From<Self>{
        let role_state: RoleState = self.into();
        crate::game::role::common_role::default_win_condition(role_state.role())
    }

    fn on_phase_start(self, _game: &mut Game, _actor_ref: PlayerReference, _phase: PhaseType) {}
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn before_role_switch(self, _game: &mut Game, _actor_ref: PlayerReference, _event: BeforeRoleSwitch) {}
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference) {}
    fn on_grave_added(self, _game: &mut Game, _actor_ref: PlayerReference, _grave: GraveReference) {}
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn on_game_start(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn before_initial_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference) {}
}

// Creates the Role enum
macros::roles! {
    Jailor : jailor,
    Villager : villager,

    Detective : detective,
    Lookout : lookout,
    Spy : spy,
    Tracker : tracker,
    Philosopher : philosopher,
    Psychic : psychic,
    Auditor : auditor,
    Snoop : snoop,
    Gossip : gossip,
    FlowerGirl : flower_girl,

    Doctor : doctor,
    Bodyguard : bodyguard,
    Cop : cop,
    Bouncer : bouncer,
    Engineer : engineer,
    Armorsmith : armorsmith,
    Steward : steward,

    Vigilante : vigilante,
    Veteran : veteran,
    Marksman: marksman,
    Deputy : deputy,
    RabbleRouser : rabble_rouser,

    Escort : escort,
    Medium : medium,
    Retributionist : retributionist,
    Journalist : journalist,
    Mayor : mayor,
    Transporter : transporter,

    // Mafia
    Godfather : godfather,
    Eros: eros,
    Counterfeiter : counterfeiter,
    Imposter : imposter,
    Retrainer : retrainer,
    Recruiter : recruiter,
    Mafioso : mafioso,
    MafiaKillingWildcard : mafia_killing_wildcard,

    MadeMan : made_man,
    Consort : consort,
    
    Hypnotist : hypnotist,
    Blackmailer : blackmailer,
    Informant: informant,
    Witch : witch,
    Necromancer : necromancer,
    Mortician : mortician,
    Framer : framer,
    Forger : forger,
    Cupid : cupid,
    MafiaSupportWildcard: mafia_support_wildcard,

    // Neutral
    Jester : jester,
    Revolutionary : revolutionary,
    Politician : politician,

    Minion : minion,
    Scarecrow : scarecrow,
    Doomsayer : doomsayer,
    Death : death,
    L : l,

    Arsonist : arsonist,
    Werewolf : werewolf,
    Ojo : ojo,
    Puppeteer: puppeteer,
    Pyrolisk : pyrolisk,
    Kira : kira,
    FiendsWildcard : fiends_wildcard,

    Wildcard : wild_card,
    TrueWildcard : true_wildcard,
    Martyr : martyr,

    Apostle : apostle,
    Disciple : disciple,
    Zealot : zealot
}

macros::priorities! {
    TopPriority,
    Ward,

    Transporter,

    Possess,
    Roleblock,

    Armorsmith,

    Deception,

    Bodyguard,

    Heal,
    Kill,
    Poison,
    Investigative,

    Cupid,
    SpyBug,

    StealMessages,

    Convert,

    FinalPriority
}

mod common_role;

mod macros {
    macro_rules! roles {
        (
            $($name:ident : $file:ident),*
        ) => {
            $(pub mod $file;)*
            $(use crate::game::role::$file::$name;)*

            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
            #[serde(rename_all = "camelCase")]
            pub enum Role {
                $($name),*
            }
            impl Role {
                pub fn values() -> Vec<Role> {
                    return vec![$(Role::$name),*];
                }
                pub fn default_state(&self) -> RoleState {
                    match self {
                        $(Self::$name => RoleState::$name($file::$name::default())),*
                    }
                }
                pub fn maximum_count(&self) -> Option<u8> {
                    match self {
                        $(Self::$name => $file::MAXIMUM_COUNT),*
                    }
                }
                pub fn faction(&self) -> crate::game::role_list::Faction {
                    match self {
                        $(Self::$name => $file::FACTION),*
                    }
                }
                pub fn defense(&self) -> DefensePower {
                    match self {
                        $(Self::$name => $file::DEFENSE),*
                    }
                }
            }

            #[derive(Clone, Debug, Serialize)]
            #[serde(tag = "type", rename_all = "camelCase")]
            pub enum ClientRoleStateEnum {
                $($name(<$name as RoleStateImpl>::ClientRoleState)),*
            }

            // This does not need to implement Deserialize or PartialEq!
            // Use Role for those things!
            #[derive(Clone, Debug)]
            pub enum RoleState {
                $($name($file::$name)),*
            }
            impl RoleState {
                pub fn role(&self) -> Role {
                    match self {
                        $(Self::$name(_) => Role::$name),*
                    }
                }
                
                pub fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority){
                    match self {
                        $(Self::$name(role_struct) => role_struct.do_night_action(game, actor_ref, priority)),*
                    }
                }
                pub fn do_day_action(self, game: &mut Game, actor_ref: PlayerReference, target_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.do_day_action(game, actor_ref, target_ref)),*
                    }
                }
                pub fn can_select(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool{
                    match self {
                        $(Self::$name(role_struct) => role_struct.can_select(game, actor_ref, target_ref)),*
                    }
                }
                pub fn can_day_target(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool{
                    match self {
                        $(Self::$name(role_struct) => role_struct.can_day_target(game, actor_ref, target_ref)),*
                    }
                }
                pub fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference, target_refs: Vec<PlayerReference>) -> Vec<Visit>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.convert_selection_to_visits(game, actor_ref, target_refs)),*
                    }
                }
                pub fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.get_current_send_chat_groups(game, actor_ref)),*
                    }
                }
                pub fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.get_current_receive_chat_groups(game, actor_ref)),*
                    }
                }
                pub fn default_win_condition(self) -> WinCondition{
                    match self {
                        $(Self::$name(role_struct) => role_struct.default_win_condition()),*
                    }
                }
                pub fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_phase_start(game, actor_ref, phase)),*
                    }
                }
                pub fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_role_creation(game, actor_ref)),*
                    }
                }
                pub fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, event: BeforeRoleSwitch){
                    match self {
                        $(Self::$name(role_struct) => role_struct.before_role_switch(game, actor_ref, event)),*
                    }
                }
                pub fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_any_death(game, actor_ref, dead_player_ref)),*
                    }
                }
                pub fn on_grave_added(self, game: &mut Game, actor_ref: PlayerReference, grave: GraveReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_grave_added(game, actor_ref, grave)),*
                    }
                }
                pub fn on_game_start(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_game_start(game, actor_ref)),*
                    }
                }
                pub fn on_game_ending(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_game_ending(game, actor_ref)),*
                    }
                }
                pub fn before_initial_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.before_initial_role_creation(game, actor_ref)),*
                    }
                }
                pub fn get_client_role_state(self, game: &Game, actor_ref: PlayerReference) -> ClientRoleStateEnum {
                    match self {
                        $(Self::$name(role_struct) => ClientRoleStateEnum::$name(role_struct.get_client_role_state(game, actor_ref))),*
                    }
                }
            }
            $(
                impl From<$file::$name> for RoleState where $name: RoleStateImpl {
                    fn from(role_struct: $file::$name) -> Self {
                        RoleState::$name(role_struct)
                    }
                }
            )*
        }
    }

    macro_rules! priorities {
        (
            $($name:ident),*
        )=>{
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub enum Priority {
                $($name,)*
            }
            impl Priority {
                pub fn values() -> Vec<Self> {
                    return vec![$(Self::$name),*];
                }
            }
        }
    }

    pub(super) use {roles, priorities};
}
#[allow(clippy::match_like_matches_macro)]
impl Role{
    pub fn possession_immune(&self)->bool{
        match self {
            Role::FlowerGirl
            | Role::Bouncer
            | Role::Veteran
            | Role::Transporter | Role::Retributionist
            | Role::Minion | Role::Doomsayer | Role::Scarecrow
            | Role::Witch | Role::Necromancer
            | Role::Ojo => true,
            _ => false,
        }
    }
    pub fn roleblock_immune(&self)->bool{
        match self {
            Role::Bouncer |
            Role::Veteran | 
            Role::Transporter | Role::Escort | Role::Retributionist | 
            Role::Jester | Role::Minion | Role::Scarecrow |
            Role::Hypnotist | Role::Consort | Role::Witch | Role::Necromancer => true,
            _ => false,
        }
    }
    pub fn wardblock_immune(&self)->bool{
        match self {
            Role::Jailor
            | Role::Bouncer
            | Role::Scarecrow => true,
            _ => false
        }
    }
    pub fn has_innocent_aura(&self, game: &Game)->bool{
        match self {
            Role::Godfather | Role::Eros => true,
            Role::Werewolf => {
                game.day_number() == 1 || game.day_number() == 3
            },
            _ => false,
        }
    }
    pub fn has_suspicious_aura(&self, _game: &Game)->bool{
        match self {
            _ => false,
        }
    }
}
pub fn same_evil_team(game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
    (actor_ref.role(game).faction() == super::role_list::Faction::Mafia && target_ref.role(game).faction() == super::role_list::Faction::Mafia) ||
    (actor_ref.role(game).faction() == super::role_list::Faction::Cult && target_ref.role(game).faction() == super::role_list::Faction::Cult)
}