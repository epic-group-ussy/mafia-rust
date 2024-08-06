use std::collections::HashMap;

use mafia_server::{game::{chat::ChatMessageVariant, phase::PhaseState, player::{PlayerIndex, PlayerReference}, role::{Role, RoleState}, tag::Tag, verdict::Verdict, Game}, packet::ToServerPacket};
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

    pub fn set_night_selection(&self, selection: Vec<TestPlayer>)->bool {
        self.0.set_selection(
            game!(self), 
            selection.into_iter().map(|t|t.0).collect()
        )
    }

    pub fn set_night_selection_single(&self, selection: TestPlayer)->bool {
        self.0.set_selection(game!(self), vec![selection.0])
    }

    pub fn vote_for_player(&self, target: Option<TestPlayer>) {
        let &PhaseState::Nomination { .. } = game!(self).current_phase() else {return};

        let player_voted_ref = match PlayerReference::index_option_to_ref(game!(self), &target.map(|f|f.0.index())){
            Ok(player_voted_ref) => player_voted_ref,
            Err(_) => return,
        };

        self.0.set_chosen_vote(game!(self), player_voted_ref, true);

        game!(self).count_votes_and_start_trial();
    }
    pub fn set_verdict(&self, verdict: Verdict) {
        self.0.set_verdict(game!(self), verdict);
    }

    pub fn send_message(&self, message: &str) {
        game!(self).on_client_message(self.0.index(), 
            ToServerPacket::SendMessage { text: message.to_string() }
        );
    }

    pub fn day_target(&self, target: TestPlayer)->bool{
        let out = self.0.can_day_target(game!(self), target.0);
        game!(self).on_client_message(self.0.index(), 
            ToServerPacket::DayTarget { player_index: target.index() }
        );
        out
    }

    pub fn alive(&self) -> bool {
        self.0.alive(game!(self))
    }

    pub fn was_roleblocked(&self) -> bool {
        self.0.night_roleblocked(game!(self))
    }

    pub fn get_messages(&self) -> Vec<ChatMessageVariant> {
        self.0.deref(game!(self)).chat_messages.iter().map(|m|{
            m.variant.clone()
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

    pub fn get_player_tags(&self) -> &HashMap<PlayerReference, Vec1<Tag>> {
        self.0.player_tags(game!(self))
    }

    pub fn get_won_game(&self) -> bool {
        self.0.get_won_game(game!(self))
    }
}