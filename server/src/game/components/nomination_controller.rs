use crate::{
    game::{
        ability_input::ControllerParametersMap,
        player::PlayerReference,
        Game
    },
    vec_set
};

pub struct NominationController;

impl NominationController{
    pub fn controller_parameters_map(game: &mut Game)->ControllerParametersMap{
        PlayerReference::all_players(game)
            .map(|actor| Self::one_player_controller(game, actor))
            .fold(ControllerParametersMap::default(), |mut acc, controller| {
                acc.combine_overwrite(controller);
                acc
            })
    }
    fn one_player_controller(game: &mut Game, actor: PlayerReference)->ControllerParametersMap{
        ControllerParametersMap::new_controller_fast(
            game,
            crate::game::ability_input::ControllerID::Nominate { player: actor },
            crate::game::ability_input::AvailableAbilitySelection::new_player_list(
                PlayerReference::all_players(game).into_iter().collect(),
                false,
                Some(1)
            ),
            crate::game::ability_input::AbilitySelection::new_player_list(vec![]),
            !actor.alive(game) ||
            actor.forfeit_vote(game) ||
            game.current_phase().phase() != crate::game::phase::PhaseType::Nomination,
            Some(crate::game::phase::PhaseType::Nomination),
            false,
            vec_set!(actor)
        )
    }
}