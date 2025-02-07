use crate::{
    game::{
        ability_input::{AbilityInput, ControllerID, ControllerParametersMap, PlayerListSelection}, chat::{ChatGroup, ChatMessageVariant}, modifiers::{ModifierType, Modifiers}, player::PlayerReference, Game
    }, packet::ToClientPacket, vec_set
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
                PlayerReference::all_players(game).filter(|p|p.alive(game)).collect(),
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
    pub fn on_validated_ability_input_received(game: &mut Game, player: PlayerReference, input: AbilityInput){
        if let Some(PlayerListSelection(voted)) = input.get_player_list_selection_if_id(ControllerID::Nominate{ player }){

            game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::Voted{
                voter: player.index(), 
                votee: voted.first().map(|p|p.index())
            });

            game.count_nomination_and_start_trial(
                !Modifiers::modifier_is_enabled(game, ModifierType::ScheduledNominations)
            );

            let packet = ToClientPacket::new_player_votes(game);
            game.send_packet_to_all(packet);
        }
    }
}