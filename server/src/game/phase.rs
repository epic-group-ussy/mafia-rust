use std::{time::Duration, io::Seek};

use serde::{Serialize, Deserialize};

use crate::network::packet::{ToClientPacket, YourButtons};

use super::{settings::PhaseTimeSettings, Game, player::{Player, PlayerIndex, self}, chat::{ChatGroup, ChatMessage}, game, verdict::Verdict, grave::Grave, role::{Role, RoleData}, role_list::Faction};


#[derive(Clone, Copy, PartialEq, Debug, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PhaseType {
    Morning,
    Discussion,
    Voting,
    Testimony,
    Judgement,
    Evening,
    Night,
}

pub struct PhaseStateMachine {
    pub time_remaining: Duration,
    pub current_state: PhaseType,
    pub day_number: u8, // Hopefully nobody is having more than 256 days anyway
}

impl PhaseStateMachine {
    pub fn new(times: PhaseTimeSettings) -> Self {
        let current_state = PhaseType::Evening;

        Self {
            time_remaining: current_state.get_length(&times),
            day_number: 1,
            current_state,
        }
    }
}

impl PhaseType {
    pub const fn get_length(&self, times: &PhaseTimeSettings) -> Duration {
        match self {
            PhaseType::Morning => times.morning,
            PhaseType::Discussion => times.discussion,
            PhaseType::Voting => times.voting,
            PhaseType::Testimony => times.testimony,
            PhaseType::Judgement => times.judgement,
            PhaseType::Evening => times.evening,
            PhaseType::Night => times.night,
        }
    }

    pub fn start(game: &mut Game) {
        // Match phase type and do stuff
        match game.phase_machine.current_state {
            PhaseType::Morning => {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PhaseChange { phase_type: PhaseType::Morning, day_number: game.phase_machine.day_number });

                //generate & add graves
                for player_index in 0..game.players.len(){
                    if game.get_unchecked_player(player_index as PlayerIndex).night_variables.died {
                        //generate grave
                        let new_grave = Grave::from_player_night(game, player_index as PlayerIndex);
                        game.send_packet_to_all(ToClientPacket::AddGrave{grave: new_grave.clone()});
                        game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PlayerDied { grave: new_grave });
                    }
                }
                //convert roles
            },
            PhaseType::Discussion => {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PhaseChange { phase_type: PhaseType::Discussion, day_number: game.phase_machine.day_number });
                
            },
            PhaseType::Voting => {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PhaseChange { phase_type: PhaseType::Voting, day_number: game.phase_machine.day_number });

                let required_votes = (game.players.iter().filter(|p|*p.alive()).collect::<Vec<&Player>>().len()/2)+1;
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::TrialInformation { required_votes, trials_left: game.trials_left });
                

                let packet = ToClientPacket::new_player_votes(game);
                game.send_packet_to_all(packet);
            },
            PhaseType::Testimony => {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PhaseChange { phase_type: PhaseType::Testimony, day_number: game.phase_machine.day_number });
                
                //TODO should be impossible for there to be no player on trial therefore unwrap
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PlayerOnTrial { player_index: game.player_on_trial.unwrap() });
                game.send_packet_to_all(ToClientPacket::PlayerOnTrial { player_index: game.player_on_trial.unwrap() });
            },
            PhaseType::Judgement => {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PhaseChange { phase_type: PhaseType::Judgement, day_number: game.phase_machine.day_number });

            },
            PhaseType::Evening => {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PhaseChange { phase_type: PhaseType::Evening, day_number: game.phase_machine.day_number });
                
            },
            PhaseType::Night => {
                //ensure mafia can kill
                //search for mafia godfather or mafioso
                let mut main_mafia_killing_exists = false;
                for player in game.players.iter(){
                    if player.role() == Role::Mafioso { 
                        main_mafia_killing_exists = true;
                        break;
                    }
                }
                //TODO for now just convert the first person we see to mafioso
                //later set an order for roles
                //ambusher should be converted first
                if !main_mafia_killing_exists{
                    for player_index in 0..(game.players.len() as PlayerIndex){

                        if game.get_unchecked_player(player_index).role().faction_alignment().faction() == Faction::Mafia {
                            Player::set_role(game, player_index, RoleData::Mafioso);
                            break;
                        }
                    }
                }

                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::PhaseChange { phase_type: PhaseType::Night, day_number: game.phase_machine.day_number });
            },
        }

        //every phase
        for player in game.players.iter(){
            player.send_packet(ToClientPacket::YourButtons{
                buttons: YourButtons::from(game, player.index().clone()) 
            });
        }
        game.send_packet_to_all(ToClientPacket::Phase { 
            phase: game.current_phase(), 
            day_number: game.phase_machine.day_number, 
            seconds_left: game.phase_machine.time_remaining.as_secs() 
        });
    }

    ///returns the next phase
    pub fn end(game: &mut Game) -> PhaseType {
        // Match phase type and do stuff
        match game.phase_machine.current_state {
            PhaseType::Morning => {
                Self::Discussion
            },
            PhaseType::Discussion => {
                Self::Voting 
            },
            PhaseType::Voting => {                
                Self::Night
            },
            PhaseType::Testimony => {
                Self::Judgement
            },
            PhaseType::Judgement => {
                
                let mut innocent = 0;   let mut guilty = 0;
                let mut messages = Vec::new();
                for player in game.players.iter(){
                    match *player.verdict(){
                        Verdict::Innocent => innocent += 1,
                        Verdict::Abstain => {},
                        Verdict::Guilty => guilty += 1,
                    }
                    messages.push(ChatMessage::JudgementVerdict { voter_player_index: player.index().clone(), verdict: player.verdict().clone() });
                }
                game.add_messages_to_chat_group(ChatGroup::All, messages);
                game.add_message_to_chat_group(ChatGroup::All, ChatMessage::TrialVerdict { player_on_trial: game.player_on_trial.unwrap(), innocent, guilty });
                
                Self::Evening
            },
            PhaseType::Evening => {
                Self::Night
            },
            PhaseType::Night => {

                //MAIN NIGHT CODE

                //get visits
                for player_index in 0..game.players.len(){
                    let player = game.get_unchecked_mut_player(player_index as PlayerIndex);

                    let targets: Vec<PlayerIndex> = player.chosen_targets().clone();
                    let role = player.role();
                    let visits = role.convert_targets_to_visits(player.index().clone(), targets, game);
                    game.get_unchecked_mut_player(player_index as PlayerIndex).night_variables.visits = visits;
                }

                //Night actions -- main loop
                for priority in 0..12{
                    for player_index in 0..game.players.len(){
                        game.get_unchecked_mut_player(player_index as PlayerIndex).role().do_night_action(player_index as PlayerIndex, priority, game);
                    }
                }

                //queue night messages
                for player in game.players.iter_mut(){
                    player.add_chat_messages(player.night_variables.night_messages.clone());
                    player.send_chat_messages();
                }


                game.phase_machine.day_number+=1;
                Self::Morning
            },
        }
    }

    pub fn is_day(&self) -> bool {
        return Self::Night != *self;
    }

}