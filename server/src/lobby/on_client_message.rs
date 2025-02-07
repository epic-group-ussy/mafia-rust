use std::{collections::VecDeque, time::{Duration, Instant}};

use crate::{game::{chat::{ChatMessage, ChatMessageVariant}, phase::PhaseType, player::{PlayerIndex, PlayerInitializeParameters}, spectator::{spectator_pointer::SpectatorIndex, SpectatorInitializeParameters}, Game}, lobby::game_client::{GameClient, GameClientLocation}, log, packet::{ToClientPacket, ToServerPacket}, strings::TidyableString, vec_map::VecMap, websocket_connections::connection::ClientSender};

use super::{lobby_client::{LobbyClient, LobbyClientID, LobbyClientType, Ready}, name_validation::{self, sanitize_server_name}, Lobby, LobbyState};

pub const MESSAGE_PER_SECOND_LIMIT: u64 = 1;
pub const MESSAGE_PER_SECOND_LIMIT_TIME: Duration = Duration::from_secs(10);

impl Lobby {
    pub fn on_client_message(&mut self, send: &ClientSender, lobby_client_id: LobbyClientID, incoming_packet: ToServerPacket){

        //RATE LIMITER
        match incoming_packet {
            ToServerPacket::Judgement { .. } |
            ToServerPacket::SendChatMessage { .. } |
            ToServerPacket::SendLobbyMessage { .. } |
            ToServerPacket::SendWhisper { .. } => {

                let last_message_times = match &mut self.lobby_state {
                    LobbyState::Game { clients, .. } => {
                        if let Some(game_player) = clients.get_mut(&lobby_client_id) {
                            &mut game_player.last_message_times
                        } else {
                            log!(error "LobbyState::Game"; "{} {:?}", "Message recieved from player not in game", incoming_packet);
                            return;
                        }
                    },
                    LobbyState::Lobby { clients, .. } => {
                        if let Some(lobby_client) = clients.get_mut(&lobby_client_id) {
                            &mut lobby_client.last_message_times
                        } else {
                            log!(error "LobbyState::Lobby"; "{} {:?}", "Message recieved from player not in lobby", incoming_packet);
                            return;
                        }
                    }
                    LobbyState::Closed => {
                        log!(error "LobbyState::Closed"; "{} {:?}", "Message recieved from player in closed lobby", incoming_packet);
                        return;
                    }
                };

                let now = Instant::now();
                while let Some(time) = last_message_times.front() {
                    if now.duration_since(*time) > MESSAGE_PER_SECOND_LIMIT_TIME {
                        last_message_times.pop_front();
                    } else {
                        break;
                    }
                }
                if last_message_times.len() >= (MESSAGE_PER_SECOND_LIMIT_TIME.as_secs() * MESSAGE_PER_SECOND_LIMIT) as usize {
                    send.send(ToClientPacket::RateLimitExceeded);
                    return;
                }
                last_message_times.push_back(now);
                
            },
            _ => {}
        }



        match incoming_packet {
            ToServerPacket::SendLobbyMessage { text } => {
                let LobbyState::Lobby { clients, .. } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "ToServerPacket::SendLobbyMessage can not be used outside of LobbyState::Lobby", lobby_client_id);
                    return
                };

                let text = text.trim_newline().trim_whitespace().truncate(100);
                if text.is_empty() {return}
                
                let name = if let Some(
                    LobbyClient { client_type: LobbyClientType::Player { name }, .. }
                ) = clients.get(&lobby_client_id) {
                    name.clone()
                } else {
                    return
                };

                self.send_to_all(ToClientPacket::AddChatMessages { chat_messages: vec![
                    ChatMessage::new_non_private(
                        ChatMessageVariant::LobbyMessage { sender: name, text }, 
                        crate::game::chat::ChatGroup::All
                    )
                ]});
            }
            ToServerPacket::SetSpectator { spectator } => {
                let LobbyState::Lobby { clients, settings } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "ToServerPacket::SetName can not be used outside of LobbyState::Lobby", lobby_client_id);
                    return
                };
                
                let new_name = name_validation::sanitize_name("".to_string(), clients);
                if let Some(player) = clients.get_mut(&lobby_client_id){
                    match &player.client_type {
                        LobbyClientType::Spectator => {
                            if !spectator {
                                player.client_type = LobbyClientType::Player { name: new_name}
                            }
                        },
                        LobbyClientType::Player { .. } => {
                            if spectator {
                                player.client_type = LobbyClientType::Spectator;
                            }
                        },
                    }
                }

                Lobby::set_rolelist_length(settings, clients);
                Self::send_players_lobby(clients);
                let role_list = settings.role_list.clone();
                self.send_to_all(ToClientPacket::RoleList { role_list } );
            }
            ToServerPacket::SetName{ name } => {
                let LobbyState::Lobby { clients, .. } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "ToServerPacket::SetName can not be used outside of LobbyState::Lobby", lobby_client_id);
                    return
                };

                let mut other_players = clients.clone();
                other_players.remove(&lobby_client_id);
                
                let new_name: String = name_validation::sanitize_name(name, &other_players);
                if let Some(player) = clients.get_mut(&lobby_client_id){
                    if let LobbyClientType::Player { name } = &mut player.client_type {
                        *name = new_name;
                    }
                }

                Self::send_players_lobby(clients);
            },
            ToServerPacket::ReadyUp{ ready } => {
                let LobbyState::Lobby { clients, .. } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "ToServerPacket::ReadyUp can not be used outside of LobbyState::Lobby", lobby_client_id);
                    return
                };

                if let Some(player) = clients.get_mut(&lobby_client_id){
                    if player.ready != Ready::Host {
                        player.ready = if ready { Ready::Ready } else { Ready::NotReady }
                    }
                }


                let mut ready = Vec::new();
                for client in clients.iter() {
                    if client.1.ready == Ready::Ready {
                        ready.push(client.0.clone());
                    }
                }
                Self::send_to_all(&self, ToClientPacket::PlayersReady { ready });
            },
            ToServerPacket::SetLobbyName{ name } => {
                let LobbyState::Lobby { .. } = self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "ToServerPacket::SetLobbyName can not be used outside of LobbyState::Lobby", lobby_client_id);
                    return
                };

                if !self.is_host(lobby_client_id) {return};

                let name = sanitize_server_name(name);
                let name = if name.is_empty() {
                    self.name = name_validation::DEFAULT_SERVER_NAME.to_string();
                    self.name.clone()
                } else {
                    self.name = name.clone();
                    self.name.clone()
                };
                
                self.send_to_all(ToClientPacket::LobbyName { name })
            },
            ToServerPacket::StartGame => {
                let LobbyState::Lobby { settings, clients } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "ToServerPacket::StartGame can not be used outside of LobbyState::Lobby", lobby_client_id);
                    return
                };
                if let Some(player) = clients.get(&lobby_client_id){
                    if !player.is_host() {return}
                }

                settings.role_list.simplify();
                let role_list = settings.role_list.clone();
                
                self.send_to_all(ToClientPacket::RoleList { role_list });

                let mut game_clients: VecMap<LobbyClientID, GameClient> = VecMap::new();
                let mut game_player_params = Vec::new();
                let mut game_spectator_params = Vec::new();


                let LobbyState::Lobby { settings, clients} = &mut self.lobby_state else {
                    unreachable!("LobbyState::Lobby was checked to be to LobbyState::Lobby in the previous line")
                };

                let mut next_player_index: PlayerIndex = 0;
                let mut next_spectator_index: SpectatorIndex = 0;

                for (lobby_client_id, lobby_client) in clients.clone().into_iter() {
                    
                    game_clients.insert(lobby_client_id, 
                        if let LobbyClientType::Spectator = lobby_client.client_type {
                            GameClient {
                                client_location: GameClientLocation::Spectator(next_spectator_index),
                                host: lobby_client.is_host(),
                                last_message_times: VecDeque::new(),
                            }
                        } else {
                            GameClient {
                                client_location: GameClientLocation::Player(next_player_index),
                                host: lobby_client.is_host(),
                                last_message_times: VecDeque::new(),
                            }
                        }
                    );
                    
                    match lobby_client.client_type {
                        LobbyClientType::Player { ref name } => {
                            game_player_params.push(PlayerInitializeParameters{
                                host: lobby_client.is_host(),
                                connection: lobby_client.connection,
                                name: name.clone(),
                            });
                            next_player_index += 1;
                        },
                        LobbyClientType::Spectator => {
                            game_spectator_params.push(SpectatorInitializeParameters{
                                host: lobby_client.is_host(),
                                connection: lobby_client.connection,
                            });
                            next_spectator_index += 1;
                        }
                    }
                }

                let game = match Game::new(settings.clone(), game_player_params, game_spectator_params){
                    Ok(game) => game,
                    Err(err) => {
                        send.send(ToClientPacket::RejectStart { reason: err });
                        log!(info "Lobby"; "Failed to start game: {:?}", err);
                        return
                    }
                };
                
                log!(info "Lobby"; "Game started with room code {}", self.room_code);

                self.lobby_state = LobbyState::Game{
                    game,
                    clients: game_clients,
                };
                let LobbyState::Game { game, clients: _player } = &mut self.lobby_state else {
                    unreachable!("LobbyState::Game was set to be to LobbyState::Game in the previous line");
                };

                Lobby::send_players_game(game);
                
                self.send_to_all(ToClientPacket::LobbyName { name: self.name.clone() })
            },
            ToServerPacket::SetPhaseTime{phase, time} => {
                let LobbyState::Lobby{ settings, clients  } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Attempted to change phase time outside of the lobby menu!", lobby_client_id);
                    return;
                };
                if let Some(player) = clients.get(&lobby_client_id){
                    if !player.is_host() {return}
                }

                match phase {
                    PhaseType::Briefing => { settings.phase_times.briefing = time; }
                    PhaseType::Obituary => { settings.phase_times.obituary = time; }
                    PhaseType::Discussion => { settings.phase_times.discussion = time; }
                    PhaseType::FinalWords => { settings.phase_times.final_words = time; }
                    PhaseType::Dusk => { settings.phase_times.dusk = time; }
                    PhaseType::Judgement => { settings.phase_times.judgement = time; }
                    PhaseType::Night => { settings.phase_times.night = time; }
                    PhaseType::Testimony => { settings.phase_times.testimony = time; }
                    PhaseType::Nomination => { settings.phase_times.nomination = time; }
                    PhaseType::Recess => { }
                };
                
                self.send_to_all(ToClientPacket::PhaseTime { phase, time });
            },
            ToServerPacket::SetPhaseTimes { phase_time_settings } => {
                let LobbyState::Lobby{ settings, clients } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Attempted to change phase time outside of the lobby menu!", lobby_client_id);
                    return;
                };
                if let Some(player) = clients.get(&lobby_client_id){
                    if !player.is_host() {return}
                }

                settings.phase_times = phase_time_settings.clone();

                self.send_to_all(ToClientPacket::PhaseTimes { phase_time_settings });
            }
            ToServerPacket::SetRoleList { role_list } => {
                let LobbyState::Lobby{ settings, clients } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Can't modify game settings outside of the lobby menu", lobby_client_id);
                    return;
                };
                if let Some(player) = clients.get(&lobby_client_id){
                    if !player.is_host() {return}
                }

                settings.role_list = role_list;
                Lobby::set_rolelist_length(settings, clients);
                
                let role_list = settings.role_list.clone();

                self.send_to_all(ToClientPacket::RoleList { role_list });
            }
            ToServerPacket::SetRoleOutline { index, role_outline } => {
                let LobbyState::Lobby{ settings, clients } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Can't modify game settings outside of the lobby menu", lobby_client_id);
                    return;
                };
                if let Some(player) = clients.get(&lobby_client_id){
                    if !player.is_host() {return}
                }

                if settings.role_list.0.len() <= index as usize {return}
                let Some(unset_outline) = settings.role_list.0.get_mut(index as usize) else {return};
                *unset_outline = role_outline.clone();
                
                self.send_to_all(ToClientPacket::RoleOutline { index, role_outline });
            }
            ToServerPacket::SimplifyRoleList => {
                let LobbyState::Lobby{ settings, clients } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Can't modify game settings outside of the lobby menu", lobby_client_id);
                    return;
                };
                if let Some(player) = clients.get(&lobby_client_id){
                    if !player.is_host() {return}
                }

                settings.role_list.simplify();
                let role_list = settings.role_list.clone();
                
                self.send_to_all(ToClientPacket::RoleList { role_list });
            }
            ToServerPacket::SetEnabledRoles {roles } => {
                let LobbyState::Lobby{ settings, .. } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Can't modify game settings outside of the lobby menu", lobby_client_id);
                    return;
                };


                settings.enabled_roles = roles.into_iter().collect();
                let roles = settings.enabled_roles.clone().into_iter().collect();
                self.send_to_all(ToClientPacket::EnabledRoles { roles });
            }
            ToServerPacket::SetEnabledModifiers {modifiers } => {
                let LobbyState::Lobby{ settings, .. } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Can't modify game settings outside of the lobby menu", lobby_client_id);
                    return;
                };

                settings.enabled_modifiers = modifiers.into_iter().collect();
                let modifiers = settings.enabled_modifiers.clone().into_iter().collect();
                self.send_to_all(ToClientPacket::EnabledModifiers { modifiers });
            }
            ToServerPacket::Leave => {
                self.remove_player(lobby_client_id);
            }
            ToServerPacket::BackToLobby => {
                let LobbyState::Game { game, clients } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {}", "Can't go back to lobby from while in lobby", lobby_client_id);
                    return;
                };
                if let Some(player) = clients.get(&lobby_client_id){
                    if !player.host {return;}
                }

                let mut new_clients = VecMap::new();
                for (lobby_client_id, game_client) in clients.clone().into_iter() {
                    new_clients.insert(lobby_client_id, LobbyClient::new_from_game_client(&game, game_client));
                }


                self.lobby_state = LobbyState::Lobby {
                    settings: game.settings.clone(),
                    clients: new_clients,
                };

                Self::send_to_all(&self, ToClientPacket::BackToLobby);

                match &self.lobby_state {
                    LobbyState::Lobby { clients, settings } => {
                        for (id, client) in clients.iter() {
                            client.send(ToClientPacket::YourId { player_id: id.clone() });
                            Self::send_settings(client, settings, self.name.clone());
                        }
                        Self::send_players_lobby(clients);
                    }
                    _ => unreachable!("LobbyState::Lobby was set to be to LobbyState::Lobby in the previous line")
                }
            }
            _ => {
                let LobbyState::Game { game, clients } = &mut self.lobby_state else {
                    log!(error "Lobby"; "{} {:?}", "ToServerPacket not implemented for lobby was sent during lobby: ", incoming_packet);
                    return;
                };

                if let Some(client) = clients.get(&lobby_client_id) {
                    match client.client_location {
                        GameClientLocation::Player(player_index) => {
                            game.on_client_message(player_index, incoming_packet)
                        }
                        GameClientLocation::Spectator(spectator_index) => {
                            game.on_spectator_message(spectator_index, incoming_packet)
                        }
                    }
                }
                
            }
        }
    }
}