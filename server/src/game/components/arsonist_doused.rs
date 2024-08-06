use std::collections::HashSet;

use crate::game::{chat::{Recipient, RecipientLike}, grave::GraveKiller, player::PlayerReference, role::Role, tag::Tag, Game};

impl Game {
    pub fn arsonist_doused(&self)->&ArsonistDoused{
        &self.arsonist_doused
    }
    pub fn set_arsonist_doused(&mut self, arsonist_doused: ArsonistDoused){
        self.arsonist_doused = arsonist_doused;
    }
}
#[derive(Default, Clone)]
pub struct ArsonistDoused {
    pub doused_players: HashSet<PlayerReference>,
}
impl ArsonistDoused {
    pub fn douse(game: &mut Game, player: PlayerReference) {
        let mut arsonist_doused = game.arsonist_doused().clone();

        if player.role(game) == Role::Arsonist {
            return
        }

        arsonist_doused.doused_players.insert(player);

        game.set_arsonist_doused(arsonist_doused);

        ArsonistDoused::tag_doused_players_for_arsonists(game);
    }
    pub fn clean_doused(game: &mut Game, player: PlayerReference) {
        let mut arsonist_doused = game.arsonist_doused().clone();

        arsonist_doused.doused_players.remove(&player);
        
        game.set_arsonist_doused(arsonist_doused);

        ArsonistDoused::tag_doused_players_for_arsonists(game);
    }
    pub fn ignite(game: &mut Game, igniter: PlayerReference) {
        let arso_doused = game.arsonist_doused();

        for player in arso_doused.doused_players.clone() {
            if player.role(game) == Role::Arsonist {continue;}
            if !player.alive(game) {continue;}
            player.try_night_kill(igniter, game, GraveKiller::Role(Role::Arsonist), 3, true);
        }
    }
    pub fn doused(&self, player: PlayerReference) -> bool {
        self.doused_players.contains(&player)
    }
    pub fn tag_doused_players_for_arsonists(game: &mut Game) {
        let arso_doused = game.arsonist_doused().clone();

        for arsonist in PlayerReference::all_players(game){
            if arsonist.role(game) != Role::Arsonist {continue;}

            for player in arso_doused.doused_players.clone() {
                if arsonist.player_has_tag(game, player, Tag::Doused) == 0{
                    arsonist.push_player_tag(game, player, Tag::Doused)
                }
            }
        }
    }
    pub fn has_suspicious_aura_douse(game: &Game, player: PlayerReference) -> bool {
        game.arsonist_doused().doused(player) &&
        PlayerReference::all_players(game).any(|player_ref|
            player_ref.alive(game) && player_ref.role(game) == Role::Arsonist
        )
    }
}