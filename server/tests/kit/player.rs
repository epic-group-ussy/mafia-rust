use mafia_server::{game::{ability_input::*, chat::ChatMessageVariant, phase::PhaseState, player::{PlayerIndex, PlayerReference}, role::{Role, RoleState}, tag::Tag, verdict::Verdict, Game}, packet::ToServerPacket, vec_map::VecMap};
use vec1::Vec1;

#[derive(Clone, Copy, Debug)]
pub struct TestPlayer(PlayerReference, *mut Game);

/// A macro to get the game from this TestPlayer.
/// ## Example:
/// ```
/// // In TestPlayer::can_day_target
/// assert!(self.0.can_day_target(game!(self), target.0));

/// game!(self).on_client_message(self.0.index(), 
///     ToServerPacket::DayTarget { player_index: target.index() }
/// );
/// ```
macro_rules! game {
    ($self:ident) => {unsafe {&mut *$self.1}}
}

impl TestPlayer {
    pub fn new(player: PlayerReference, game: &Game) -> Self {
        TestPlayer(player, game as *const Game as *mut Game)
    }

    pub fn index(&self) -> PlayerIndex {
        self.0.index()
    }

    pub fn player_ref(&self) -> PlayerReference {
        self.0
    }

    pub fn send_ability_input(&self, ability_input: AbilityInput) {
        game!(self).on_client_message(self.0.index(), 
            ToServerPacket::AbilityInput { ability_input }
        );
    }

    pub fn send_ability_input_unit_typical(&self)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                AbilitySelection::new_unit()
            )
        );
        true
    }

    pub fn send_ability_input_two_player_typical(&self, a: TestPlayer, b: TestPlayer)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                AbilitySelection::new_two_player_option(
                    Some((a.player_ref(), b.player_ref()))
                )
            )
        );
        true
    }

    pub fn send_ability_input_player_list_typical(&self, selection: impl Into<Vec<TestPlayer>>)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                AbilitySelection::new_player_list(selection.into().iter().map(TestPlayer::player_ref).collect())
            )
        );
        true
    }

    pub fn send_ability_input_boolean_typical(&self, selection: bool)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                AbilitySelection::new_boolean(selection)
            )
        );
        true
    }

    pub fn send_ability_input_player_list(&self, selection: impl Into<Vec<TestPlayer>>, id: RoleControllerID)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), id),
                AbilitySelection::new_player_list(selection.into().iter().map(|p| p.player_ref()).collect())
            )
        );
        true
    }

    pub fn vote_for_player(&self, target: impl Into<Option<TestPlayer>>) {
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::nominate(self.player_ref()),
                AbilitySelection::new_player_list(target.into().iter().map(|p| p.player_ref()).collect())
            )
        );
    }
    pub fn set_verdict(&self, verdict: Verdict) {
        self.0.set_verdict(game!(self), verdict);
    }

    pub fn send_message(&self, message: &str) {
        game!(self).on_client_message(self.0.index(), 
            ToServerPacket::SendChatMessage { text: message.to_string(), block: false }
        );
    }

    pub fn alive(&self) -> bool {
        self.0.alive(game!(self))
    }

    pub fn was_roleblocked(&self) -> bool {
        self.0.night_roleblocked(game!(self))
    }

    pub fn get_messages(&self) -> Vec<ChatMessageVariant> {
        self.0.chat_messages(game!(self)).iter().map(|m|{
            m.get_variant().clone()
        }).collect()
    }

    pub fn get_messages_after_last_message(&self, last_message: ChatMessageVariant) -> Vec<ChatMessageVariant> {
        let mut found = false;
        let mut out = Vec::new();
        for message in self.get_messages().iter() {
            if *message == last_message {
                found = true;
            }else if found {
                out.push(message.clone());
            }
        }
        out
    }
    pub fn get_messages_after_night(&self, day_number: u8) -> Vec<ChatMessageVariant> {
        self.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number }
        )
    }

    pub fn role(&self) -> Role {
        self.0.role(game!(self))
    }

    pub fn role_state(&self) -> &RoleState{
        self.0.role_state(game!(self))
    }

    pub fn set_role_state(&self, new_role_data: RoleState){
        self.0.set_role_state(game!(self), new_role_data);
    }

    pub fn get_player_tags(&self) -> &VecMap<PlayerReference, Vec1<Tag>> {
        self.0.player_tags(game!(self))
    }

    pub fn get_won_game(&self) -> bool {
        self.0.get_won_game(game!(self))
    }
}

impl From<TestPlayer> for Vec<TestPlayer> {
    fn from(value: TestPlayer) -> Self {
        vec![value]
    }
}