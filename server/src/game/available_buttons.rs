use serde::Serialize;

use super::{player::PlayerReference, phase::PhaseType, Game};


#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvailableButtons{
    pub vote: bool,
}
impl AvailableButtons{
    pub fn from_player_target(game: &Game, actor_ref: PlayerReference, target_ref: PlayerReference)->Self{
        Self{
            vote: 
                actor_ref != target_ref &&
                game.current_phase().phase() == PhaseType::Nomination &&
                actor_ref.chosen_vote(game).is_none() && 
                !actor_ref.forfeit_vote(game) &&
                actor_ref.alive(game) && target_ref.alive(game),
        }
    }
    pub fn from_player(game: &Game, actor_ref: PlayerReference)->Vec<Self>{
        let mut out = Vec::new();

        for target_ref in PlayerReference::all_players(game){
            out.push(Self::from_player_target(game, actor_ref, target_ref));
        }
        out
    }
}

//Whats up my Diggity Dogs, Aum Sizzle as arrived to the party. I love you all. 
