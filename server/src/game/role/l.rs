
use rand::seq::{IteratorRandom, SliceRandom};
use serde::Serialize;

use crate::game::attack_power::DefensePower;
use crate::game::chat::ChatMessageVariant;
use crate::game::event::before_role_switch::BeforeRoleSwitch;
use crate::game::grave::{Grave, GraveDeathCause, GraveInformation, GraveKiller, GravePhase};
use crate::game::player::PlayerReference;
use crate::game::role::RoleState;
use crate::game::role_list::{Faction, RoleSet};

use crate::game::visit::Visit;
use crate::game::Game;
use super::jester::Jester;
use super::{GetClientRoleState, Priority, Role, RoleStateImpl};


#[derive(Clone, Debug, Default)]
pub struct L {
    target: LTarget,
}

#[derive(Clone, Serialize, Debug)]
pub struct ClientRoleState;

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
pub enum LTarget{
    Target(PlayerReference),
    Won,
}
impl LTarget {
    fn get_target(&self)->Option<PlayerReference>{
        if let Self::Target(p) = self {
            Some(*p)
        }else{
            None
        }
    }
}
impl Default for LTarget {
    fn default() -> Self {
        Self::Won
    }
}

pub(super) const FACTION: Faction = Faction::Neutral;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for L {
    type ClientRoleState = ClientRoleState;
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if self.won() || !actor_ref.alive(game) || priority != Priority::TopPriority {
            return;
        }

        let Some(visit) = actor_ref.night_visits(game).first() else { return };

        if let LTarget::Target(target) = self.target {
            if target != visit.target { return }
            
            actor_ref.set_role_state(game, L { target: LTarget::Won });

            actor_ref.die(game, Grave {
                player: actor_ref,
                died_phase: GravePhase::Night,
                day_number: game.day_number(),
                information: GraveInformation::Normal {
                    role: Role::L,
                    will: "Obscured".to_string(), // Lol
                    death_cause: GraveDeathCause::Killers(vec![
                        GraveKiller::Suicide
                    ]),
                    death_notes: vec![]
                },
            });

            let mut living_players: Vec<PlayerReference> = PlayerReference::all_players(game)
                .filter(|p| p.alive(game) && *p != actor_ref)
                .collect();

            living_players.shuffle(&mut rand::thread_rng());
            living_players.drain(0..(living_players.len() / 2));

            target.push_night_message(game, ChatMessageVariant::LGuessedYou);
            for player in living_players {
                target.insert_role_label(game, player);
            }
        }
    }
    fn can_select(self, game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        actor_ref != target_ref &&
        !actor_ref.night_jailed(game) &&
        actor_ref.selection(game).is_empty() &&
        actor_ref.alive(game) &&
        !self.won() &&
        game.day_number() > 1
    }
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
        let target = if let Some(player) = find_player_assigned_role(game, Role::Kira) {
            player
        } else if let Some(player) = find_player_assigned_role(game, RoleSet::MafiaKilling) {
            player
        } else if let Some(player) = PlayerReference::all_players(game)
            .filter(|p| *p != actor_ref)
            .choose(&mut rand::thread_rng())
        {
            player
        } else { // Actually impossible
            actor_ref.set_role_and_wincon(game, RoleState::Jester(Jester::default()));
            return;
        };

        actor_ref.add_private_chat_message(game, ChatMessageVariant::LTargetRole{ role: target.role(game) });
        actor_ref.set_role_state(game, RoleState::L(L { target: LTarget::Target(target) }));
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, event: BeforeRoleSwitch) {
        if self.target.get_target().is_some_and(|target| target == event.player()) {
            actor_ref.add_private_chat_message(game, ChatMessageVariant::LTargetRole { role: event.new_role().role() });
        } 
    }
    fn convert_selection_to_visits(self,  game: &Game, actor_ref: PlayerReference, target_refs: Vec<PlayerReference>) -> Vec<Visit> {
        crate::game::role::common_role::convert_selection_to_visits(game, actor_ref, target_refs, true)
    }
}

fn find_player_assigned_role(game: &Game, roles: impl Into<RoleOrRoleSet> + Clone) -> Option<PlayerReference> {
    game.roles_originally_generated.iter()
        .find(|(role, _)| 
            match &roles.clone().into() {
                RoleOrRoleSet::Role(desired_role) => *role == *desired_role,
                RoleOrRoleSet::RoleSet(role_set) => role_set.get_roles().contains(role)
            }
        )
        .map(|(_, player)| *player)
}

impl GetClientRoleState<ClientRoleState> for L {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl L {
    pub fn won(&self)->bool{
        self.target == LTarget::Won
    }
}

enum RoleOrRoleSet {
    Role(Role),
    RoleSet(RoleSet)
}

impl From<Role> for RoleOrRoleSet {
    fn from(value: Role) -> Self {
        RoleOrRoleSet::Role(value)
    }
}

impl From<RoleSet> for RoleOrRoleSet {
    fn from(value: RoleSet) -> Self {
        RoleOrRoleSet::RoleSet(value)
    }
}