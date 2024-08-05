use crate::game::{phase::PhaseType, player::PlayerReference, player_group::PlayerGroup, role::{godfather::Godfather, Role, RoleState}, role_list::Faction, Game};
use rand::prelude::SliceRandom;

#[derive(Default, Clone)]
pub struct Mafia;
impl Mafia{
    pub fn on_phase_start(game: &mut Game, _phase: PhaseType){
        //This depends on role_state.on_phase_start being called before this
        Mafia::ensure_mafia_can_kill(game);
    }
    pub fn on_game_start(game: &mut Game) {
        //This depends on role_state.on_game_start being called before this
        Mafia::ensure_mafia_can_kill(game);
    }
    pub fn on_any_death(game: &mut Game, dead_player: PlayerReference){
        //This depends on role_state.on_any_death being called before this
        if dead_player.role(game).faction() == Faction::Mafia {
            Mafia::ensure_mafia_can_kill(game);
        }
    }
    pub fn on_role_switch(game: &mut Game, old: Role, new: Role) {
        if old.faction() == Faction::Mafia || new.faction() == Faction::Mafia {
            Mafia::ensure_mafia_can_kill(game);
        }

        for member in Mafia::get_members(game) {
            member.reveal_role(game, PlayerGroup::Mafia)
        }
    }
    pub fn get_members(game: &Game)->Vec<PlayerReference>{
        PlayerReference::all_players(game).filter(
            |p| p.role(game).faction() == Faction::Mafia
        ).collect()
    }
    pub fn get_living_members(&self, game: &Game)->Vec<PlayerReference>{
        PlayerReference::all_players(game).filter(
            |p| p.role(game).faction() == Faction::Mafia && p.alive(game)
        ).collect()
    }

    fn ensure_mafia_can_kill(game: &mut Game){

        for player_ref in PlayerReference::all_players(game){
            if (player_ref.role(game) == Role::Godfather || player_ref.role(game) == Role::Mafioso) && player_ref.alive(game) { 
                return;
            }
        }

        //if no mafia killing exists, the code can reach here
        let list_of_living_mafia = PlayerReference::all_players(game)
            .filter(|p| 
                p.role(game).faction() == Faction::Mafia && p.alive(game)
            )
            .collect::<Vec<PlayerReference>>();
        
        //choose random mafia to be godfather
        let random_mafia = list_of_living_mafia.choose(&mut rand::thread_rng());

        if let Some(random_mafia) = random_mafia{
            random_mafia.set_role(game, RoleState::Godfather(Godfather::default()));
        }
    }
}