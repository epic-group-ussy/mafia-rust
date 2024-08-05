use std::time::Duration;

use crate::{
    client_connection::ClientConnection, game::{available_buttons::AvailableButtons, chat::ChatMessageVariant, phase::PhaseState, Game, GameOverReason}, lobby::GAME_DISCONNECT_TIMER_SECS, packet::ToClientPacket, websocket_connections::connection::ClientSender
};

use super::PlayerReference;

impl PlayerReference{
    pub fn connect(&self, game: &mut Game, sender: ClientSender){
        self.deref_mut(game).connection = ClientConnection::Connected(sender);
        self.send_join_game_data(game);
    }
    pub fn lose_connection(&self, game: &mut Game){
        self.deref_mut(game).connection = ClientConnection::CouldReconnect { disconnect_timer: Duration::from_secs(GAME_DISCONNECT_TIMER_SECS) };
    }
    pub fn quit(&self, game: &mut Game) {
        self.deref_mut(game).connection = ClientConnection::Disconnected;
        if self.alive(game) {
            game.add_message(
                crate::game::player_group::PlayerGroup::All, 
                ChatMessageVariant::PlayerQuit{player_index: self.index()}
            );
        }
    }

    pub fn connection<'a>(&self, game: &'a Game) -> &'a ClientConnection {
        &self.deref(game).connection
    }
    pub fn is_connected(&self, game: &Game) -> bool {
        matches!(self.deref(game).connection, ClientConnection::Connected(_))
    }
    pub fn could_reconnect(&self, game: &Game) -> bool {
        matches!(self.deref(game).connection, ClientConnection::CouldReconnect {..})
    }
    pub fn is_disconnected(&self, game: &Game) -> bool {
        matches!(self.deref(game).connection, ClientConnection::Disconnected)
    }

    pub fn send_packet(&self, game: &Game, packet: ToClientPacket){
        self.deref(game).connection.send_packet(packet);
    }
    pub fn send_packets(&self, game: &Game, packets: Vec<ToClientPacket>){
        for packet in packets{
            self.send_packet(game, packet);
        }
    }
    pub fn send_repeating_data(&self, game: &mut Game){
        self.send_chat_messages(game);
        self.send_available_buttons(game);
    }
    pub fn send_join_game_data(&self, game: &mut Game){
        // General
        self.send_packets(game, vec![
            ToClientPacket::GamePlayers{ 
                players: PlayerReference::all_players(game).map(|p|p.name(game).clone()).collect()
            },
            ToClientPacket::ExcludedRoles { roles: game.settings.excluded_roles.clone().into_iter().collect() },
            ToClientPacket::RoleList {role_list: game.settings.role_list.clone()},
            ToClientPacket::PlayerAlive{
                alive: PlayerReference::all_players(game).map(|p|p.alive(game)).collect()
            }
        ]);

        if !game.ticking {
            self.send_packet(game, ToClientPacket::GameOver { reason: GameOverReason::Draw })
        }

        if let PhaseState::Testimony { player_on_trial, .. }
            | PhaseState::Judgement { player_on_trial, .. }
            | PhaseState::FinalWords { player_on_trial } = game.current_phase() {
            self.send_packet(game, ToClientPacket::PlayerOnTrial{
                player_index: player_on_trial.index()
            });
        }
        let votes_packet = ToClientPacket::new_player_votes(game);
        self.send_packet(game, votes_packet);
        for grave in game.graves.iter(){
            self.send_packet(game, ToClientPacket::AddGrave { grave: grave.clone() });
        }

        // Player specific
        self.requeue_chat_messages(game);
        self.send_chat_messages(game);
        self.send_available_buttons(game);

        self.send_packets(game, vec![
            ToClientPacket::YourPlayerIndex { 
                player_index: self.index() 
            },
            ToClientPacket::YourRoleState {
                role_state: self.role_state(game).clone()
            },
            ToClientPacket::YourRoleLabels { 
                role_labels: PlayerReference::ref_map_to_index(self.role_label_map(game)) 
            },
            ToClientPacket::YourPlayerTags { 
                player_tags: PlayerReference::ref_map_to_index(self.player_tags(game).clone())
            },
            ToClientPacket::YourSelection{
                player_indices: PlayerReference::ref_vec_to_index(self.selection(game))
            },
            ToClientPacket::YourJudgement{
                verdict: self.verdict(game)
            },
            ToClientPacket::YourVoting{ 
                player_index: PlayerReference::ref_option_to_index(&self.chosen_vote(game))
            },
            ToClientPacket::YourWill{
                will: self.will(game).clone()
            },
            ToClientPacket::YourNotes{
                notes: self.notes(game).clone()
            },
            ToClientPacket::YourCrossedOutOutlines{
                crossed_out_outlines: self.crossed_out_outlines(game).clone()
            },
            ToClientPacket::YourButtons{
                buttons: AvailableButtons::from_player(game, *self)
            },
            ToClientPacket::Phase { 
                phase: game.current_phase().clone(),
                day_number: game.phase_machine.day_number 
            },
            ToClientPacket::PhaseTimeLeft { seconds_left: game.phase_machine.time_remaining.as_secs() }
        ]);
    }



    pub fn send_chat_messages(&self, game: &mut Game){
        
        if self.deref(game).queued_chat_messages.is_empty() {
            return;
        }
        
        let mut chat_messages_out = vec![];

        // Send in chunks
        for _ in 0..5 {
            let msg_option = self.deref(game).queued_chat_messages.first();
            if let Some(msg) = msg_option{
                chat_messages_out.push(msg.clone());
                self.deref_mut(game).queued_chat_messages.remove(0);
            }else{ break; }
        }
        
        self.send_packet(game, ToClientPacket::AddChatMessages { chat_messages: chat_messages_out });
        

        self.send_chat_messages(game);
    }
    #[allow(unused)]
    fn requeue_chat_messages(&self, game: &mut Game){
        self.deref_mut(game).queued_chat_messages = self.deref(game).chat_messages.clone();
    }   

    fn send_available_buttons(&self, game: &mut Game){
        let new_buttons = AvailableButtons::from_player(game, *self);
        if new_buttons == self.deref(game).last_sent_buttons{
            return;
        }
        
        self.send_packet(game, ToClientPacket::YourButtons { buttons: new_buttons.clone() });
        self.deref_mut(game).last_sent_buttons = new_buttons
    }

}

