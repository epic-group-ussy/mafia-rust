use crate::{log, packet::ToServerPacket, strings::TidyableString};

use super::{
    chat::{ChatGroup, ChatMessageVariant, MessageSender},
    event::on_fast_forward::OnFastForward, modifiers::{ModifierType, Modifiers},
    phase::PhaseType,
    player::{PlayerIndex, PlayerReference},
    role::{
        mayor::Mayor, politician::Politician,
        Role, RoleState
    },
    spectator::spectator_pointer::{SpectatorIndex, SpectatorPointer}, Game
};




impl Game {
    pub fn on_spectator_message(&mut self, sender_index: SpectatorIndex, incoming_packet: ToServerPacket){
        let sender_pointer = SpectatorPointer::new(sender_index);

        #[allow(clippy::single_match)]
        match incoming_packet {
            ToServerPacket::VoteFastForwardPhase { fast_forward } => {
                if sender_pointer.host(self) && fast_forward && !self.phase_machine.time_remaining.is_zero(){
                    OnFastForward::invoke(self);
                }
            },
            _ => {
            }
        }
    }
    pub fn on_client_message(&mut self, sender_player_index: PlayerIndex, incoming_packet: ToServerPacket){

        let sender_player_ref = match PlayerReference::new(self, sender_player_index){
            Ok(sender_player_ref) => sender_player_ref,
            Err(_) => {
                log!(error "Game"; "Received message from invalid player index: {}", sender_player_index);
                return;
            }
        };

        'packet_match: {match incoming_packet {
            ToServerPacket::Judgement { verdict } => {
                if self.current_phase().phase() != PhaseType::Judgement {break 'packet_match;}
                
                sender_player_ref.set_verdict(self, verdict);
            },
            ToServerPacket::SendChatMessage { text, block } => {
                if text.replace(['\n', '\r'], "").trim().is_empty() {
                    break 'packet_match;
                }
                
                for chat_group in sender_player_ref.get_current_send_chat_groups(self){
                    let message_sender = match chat_group {
                        ChatGroup::Jail => {
                            if sender_player_ref.role(self) == Role::Jailor {
                                Some(MessageSender::Jailor)
                            }else{None}
                        },
                        ChatGroup::Kidnapped => {
                            if sender_player_ref.role(self) == Role::Kidnapper {
                                Some(MessageSender::Jailor)
                            }else{None}
                        },
                        ChatGroup::Dead => {
                            if sender_player_ref.alive(self) {
                                Some(MessageSender::LivingToDead{ player: sender_player_index })
                            }else{None}
                        },
                        ChatGroup::Interview => {
                            if sender_player_ref.role(self) == Role::Reporter {
                                Some(MessageSender::Reporter)
                            }else{None}
                        },
                        _ => {None}
                    };

                    let message_sender = message_sender.unwrap_or(MessageSender::Player { player: sender_player_index });


                    self.add_message_to_chat_group(
                        chat_group.clone(),
                        ChatMessageVariant::Normal{
                            message_sender,
                            text: text.trim_newline().trim_whitespace().truncate(600).truncate_lines(35), 
                            block
                        }
                    );
                }
            },
            ToServerPacket::SendWhisper { player_index: whispered_to_player_index, text } => {
                if Modifiers::modifier_is_enabled(self, ModifierType::NoWhispers) {
                    sender_player_ref.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                    break 'packet_match
                }

                let whisperee_ref = match PlayerReference::new(self, whispered_to_player_index){
                    Ok(whisperee_ref) => whisperee_ref,
                    Err(_) => {
                        sender_player_ref.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                        break 'packet_match
                    },
                };

                if !self.current_phase().is_day() || 
                    whisperee_ref.alive(self) != sender_player_ref.alive(self) ||
                    whisperee_ref == sender_player_ref || 
                    !sender_player_ref.get_current_send_chat_groups(self).contains(&ChatGroup::All) ||
                    text.replace(['\n', '\r'], "").trim().is_empty()
                {
                    sender_player_ref.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                    break 'packet_match;
                }

                if let RoleState::Mayor(Mayor{revealed: true}) = whisperee_ref.role_state(self) {
                    sender_player_ref.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                    break 'packet_match;
                }
                if let RoleState::Mayor(Mayor{revealed: true}) = sender_player_ref.role_state(self) {
                    sender_player_ref.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                    break 'packet_match;
                }
                if let RoleState::Politician(Politician{revealed: true, ..}) = whisperee_ref.role_state(self) {
                    sender_player_ref.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                    break 'packet_match;
                }
                if let RoleState::Politician(Politician{revealed: true, ..}) = sender_player_ref.role_state(self) {
                    sender_player_ref.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                    break 'packet_match;
                }


                self.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::BroadcastWhisper { whisperer: sender_player_index, whisperee: whispered_to_player_index });
                let message = ChatMessageVariant::Whisper { 
                    from_player_index: sender_player_index, 
                    to_player_index: whispered_to_player_index, 
                    text 
                };
        
                sender_player_ref.add_private_chat_message(self, message.clone());

                for player in PlayerReference::all_players(self){
                    if 
                        player.role(self) == Role::Informant ||
                        whisperee_ref == player
                    {
                        player.add_private_chat_message(self, message.clone());
                    }
                }
            },
            ToServerPacket::SaveWill { will } => {
                sender_player_ref.set_will(self, will);
            },
            ToServerPacket::SaveNotes { notes } => {
                sender_player_ref.set_notes(self, notes);
            },
            ToServerPacket::SaveCrossedOutOutlines { crossed_out_outlines } => {
                sender_player_ref.set_crossed_out_outlines(self, crossed_out_outlines);
            },
            ToServerPacket::SaveDeathNote { death_note } => {
                sender_player_ref.set_death_note(self, death_note);
            },
            ToServerPacket::AbilityInput { ability_input } => 
                ability_input.on_client_message(self, sender_player_ref),
            ToServerPacket::SetDoomsayerGuess { guesses } => {
                if let RoleState::Doomsayer(mut doomsayer) = sender_player_ref.role_state(self).clone(){
                    doomsayer.guesses = guesses;
                    sender_player_ref.set_role_state(self, RoleState::Doomsayer(doomsayer));
                }
            },
            ToServerPacket::SetConsortOptions { 
                roleblock, 
                you_were_roleblocked_message, 
                you_survived_attack_message, 
                you_were_protected_message, 
                you_were_transported_message, 
                you_were_possessed_message, 
                your_target_was_jailed_message 
            } => {
                if let RoleState::Hypnotist(mut hypnotist) = sender_player_ref.role_state(self).clone(){
                    hypnotist.roleblock = roleblock;

                    hypnotist.you_were_roleblocked_message = you_were_roleblocked_message;
                    hypnotist.you_survived_attack_message = you_survived_attack_message;
                    hypnotist.you_were_protected_message = you_were_protected_message;
                    hypnotist.you_were_transported_message = you_were_transported_message;
                    hypnotist.you_were_possessed_message = you_were_possessed_message;
                    hypnotist.your_target_was_jailed_message = your_target_was_jailed_message;

                    //There must be at least one message enabled, so if none are, enable roleblocked message
                    hypnotist.ensure_at_least_one_message();

                    sender_player_ref.set_role_state(self, RoleState::Hypnotist(hypnotist));
                }
            },
            ToServerPacket::VoteFastForwardPhase { fast_forward } => {
                sender_player_ref.set_fast_forward_vote(self, fast_forward);
            },
            _ => {
                log!(fatal "Game"; "Unimplemented ToServerPacket: {incoming_packet:?}");
                unreachable!();
            }
        }}
        
        for player_ref in PlayerReference::all_players(self){
            player_ref.send_repeating_data(self)
        }
        for spectator_ref in SpectatorPointer::all_spectators(self){
            spectator_ref.send_repeating_data(self)
        }

    }
}