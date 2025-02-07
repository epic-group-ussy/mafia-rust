use std::time::Duration;

use crate::{
    client_connection::ClientConnection, 
    game::{
        chat::ChatMessageVariant, components::insider_group::InsiderGroupID,
        phase::PhaseState, Game, GameOverReason
    },
    lobby::GAME_DISCONNECT_TIMER_SECS,
    packet::ToClientPacket, websocket_connections::connection::ClientSender
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
            game.add_message_to_chat_group(
                crate::game::chat::ChatGroup::All, 
                ChatMessageVariant::PlayerQuit{player_index: self.index(), game_over: game.game_is_over()}
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
    }
    pub fn send_join_game_data(&self, game: &mut Game){
        // General
        self.send_packets(game, vec![
            ToClientPacket::GamePlayers{ 
                players: PlayerReference::all_players(game).map(|p|p.name(game).clone()).collect()
            },
            ToClientPacket::EnabledRoles { roles: game.settings.enabled_roles.clone().into_iter().collect() },
            ToClientPacket::RoleList {role_list: game.settings.role_list.clone()},
            ToClientPacket::EnabledModifiers {
                modifiers: game.settings.enabled_modifiers.clone().into_iter().collect()
            },
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
        InsiderGroupID::send_player_insider_groups(game, *self);
        InsiderGroupID::send_fellow_insiders(game, *self);

        self.send_packets(game, vec![
            ToClientPacket::YourSendChatGroups {
                send_chat_groups: self.get_current_send_chat_groups(game).into_iter().collect()
            },
            ToClientPacket::YourPlayerIndex { 
                player_index: self.index() 
            },
            ToClientPacket::YourRoleState {
                role_state: self.role_state(game).clone().get_client_role_state(game, *self)
            },
            ToClientPacket::YourRoleLabels { 
                role_labels: PlayerReference::ref_vec_map_to_index(self.role_label_map(game)) 
            },
            ToClientPacket::YourPlayerTags { 
                player_tags: PlayerReference::ref_vec_map_to_index(self.player_tags(game).clone())
            },
            ToClientPacket::YourJudgement{
                verdict: self.verdict(game)
            },
            ToClientPacket::YourAllowedControllers { 
                save: game.saved_controllers.controllers_allowed_to_player(*self).all_controllers().clone(),
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
            ToClientPacket::Phase { 
                phase: game.current_phase().clone(),
                day_number: game.phase_machine.day_number 
            },
            ToClientPacket::PhaseTimeLeft { seconds_left: game.phase_machine.time_remaining.as_secs() },
            ToClientPacket::GameInitializationComplete
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
}

