use rand::seq::SliceRandom;

use crate::game::{phase::PhaseType, player::PlayerReference, role::{Role, RoleState}, role_list::{Faction, RoleSet}, Game};


const DEFAULT_MAFIA_KILLING_ROLE: Role = Role::Godfather;

#[derive(Clone)]
pub struct Mafia;
impl Game{
    pub fn mafia(&self)->&Mafia{
        &self.mafia
    }
    pub fn set_mafia(&mut self, mafia: Mafia){
        self.mafia = mafia;
    }
}
impl Mafia{
    pub fn on_phase_start(_game: &mut Game, _phase: PhaseType){
    }
    pub fn on_game_start(game: &mut Game) {
        Mafia::give_mafia_killing_role(game, DEFAULT_MAFIA_KILLING_ROLE.default_state());
    }


    /// - This must go after rolestate on any death
    /// - Godfathers backup should become godfather if godfather dies as part of the godfathers ability
    pub fn on_any_death(game: &mut Game, dead_player: PlayerReference){
        if RoleSet::MafiaKilling.get_roles().contains(&dead_player.role(game)) {
            Mafia::give_mafia_killing_role(game, dead_player.role_state(game).clone());
        }
    }
    pub fn on_role_switch(game: &mut Game, old: RoleState, _new: RoleState) {
        if RoleSet::MafiaKilling.get_roles().contains(&old.role()) {
            Mafia::give_mafia_killing_role(game, old);
        }

        for a in Mafia::get_members(game) {
            for b in Mafia::get_members(game) {
                a.insert_role_label(game, b);
            }
        }
    }
    pub fn get_members(game: &Game)->Vec<PlayerReference>{
        PlayerReference::all_players(game).filter(
            |p| p.role(game).faction() == Faction::Mafia
        ).collect()
    }
    pub fn get_living_members(game: &Game)->Vec<PlayerReference>{
        PlayerReference::all_players(game).filter(
            |p| p.role(game).faction() == Faction::Mafia && p.alive(game)
        ).collect()
    }


    pub fn give_mafia_killing_role(
        game: &mut Game,
        role: RoleState
    ){
        //if they already have a mafia killing then return
        if Mafia::get_living_members(game).iter().any(|p|
            RoleSet::MafiaKilling.get_roles().contains(&p.role(game))
        ) {return;}

        //choose random mafia to be mafia killing
        let all_living_mafia = Mafia::get_living_members(game);
        let random_mafia = all_living_mafia.choose(&mut rand::thread_rng());
        
        if let Some(random_mafia) = random_mafia {
            random_mafia.set_role_and_wincon(game, role);
        }
    }
}