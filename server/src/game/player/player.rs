use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    game::{
        Game, chat::{ChatMessage, night_message::NightInformation}, 
        visit::Visit, verdict::Verdict,
        phase::{
            PhaseType
        }, 
        role::{
            Role, RoleData
        }, grave::GraveKiller, 
    }, 
    network::packet::{ToClientPacket, YourButtons}
};

use super::{player_voting_variables::PlayerVotingVariables, player_night_variables::PlayerNightVariables, PlayerIndex};


pub struct Player {
    pub(super) name: String,
    pub(super) index: PlayerIndex,
    pub(super) role_data: RoleData,
    pub(super) alive: bool,
    pub(super) will: String,
    pub(super) notes: String,

    pub(super) role_labels: HashMap<PlayerIndex, Role>,   //when you can see someone elses role in the playerlist, dead players and teammates, mayor

    pub(super) sender: UnboundedSender<ToClientPacket>,

    pub(super) chat_messages: Vec<ChatMessage>,
    pub(super) queued_chat_messages: Vec<ChatMessage>, 

    pub night_variables: PlayerNightVariables,
    pub(super) voting_variables: PlayerVotingVariables,
}

impl Player {
    pub fn new(index: PlayerIndex, name: String, sender: UnboundedSender<ToClientPacket>, role: Role) -> Self {
        let p = Self {
            name,
            index,
            role_data: role.default_data(),
            alive: true,
            will: "".to_string(),
            notes: "".to_string(),

            role_labels: HashMap::new(),

            sender,
            chat_messages: Vec::new(),

            queued_chat_messages: Vec::new(),

            night_variables: PlayerNightVariables::new(),
            voting_variables: PlayerVotingVariables::new(),
        };
        p.send_packet(ToClientPacket::YourPlayerIndex { player_index: p.index().clone() });
        p
    }


    //Night helper functions

    ///returns true if they were roleblocked by you
    pub fn roleblock(&mut self)->bool{
        if self.role().roleblockable() {
            self.night_variables.roleblocked = true;
            self.night_variables.night_messages.push(ChatMessage::NightInformation { night_information: NightInformation::RoleBlocked { immune: false }});
            return true;
        }else{
            self.night_variables.night_messages.push(ChatMessage::NightInformation { night_information: NightInformation::RoleBlocked { immune: true }});
            return false;
        }
    }
    ///returns true if attack overpowered defense and/or they are now dead.
    pub fn try_night_kill(game: &mut Game, player_index: PlayerIndex, grave_killer: GraveKiller, attack: u8)->bool{
        let player = game.get_unchecked_mut_player(player_index);
        if !player.alive {return true;}

        player.night_variables.attacked = true;

        if player.night_variables.defense >= attack {
            player.add_chat_message(ChatMessage::NightInformation { night_information: NightInformation::YouSurvivedAttack });
            return false;
        }
        
        //die
        player.night_variables.night_messages.push(ChatMessage::NightInformation { night_information: NightInformation::YouDied });
        player.night_variables.died = true;
        player.alive = false;
        player.night_variables.grave_killers.push(grave_killer);

        true
    }
    /// swap this persons role, sending them the role chat message, and associated changes
    pub fn set_role(game: &mut Game, player_index: PlayerIndex, new_role_data: RoleData){

        game.get_unchecked_mut_player(player_index).set_role_data(new_role_data);
        game.get_unchecked_mut_player(player_index).role().on_role_creation(player_index, game);
        game.get_unchecked_mut_player(player_index).add_chat_message(ChatMessage::RoleAssignment { role: new_role_data.role()});
    }


    pub fn add_chat_message(&mut self, message: ChatMessage) {
        self.chat_messages.push(message.clone());
        self.queued_chat_messages.push(message);
    }
    pub fn add_chat_messages(&mut self, messages: Vec<ChatMessage>){
        for message in messages.into_iter(){
            self.add_chat_message(message);
        }
    }
    

    pub fn tick(&mut self){
        self.send_chat_messages();
        // self.send_available_buttons();
    }
    pub fn reset_phase_start(game: &mut Game, player_index: PlayerIndex, phase: PhaseType){
        match phase {
            PhaseType::Morning => {},
            PhaseType::Discussion => {},
            PhaseType::Voting => {
                Player::reset_voting_variables(game, player_index);
                let player = game.get_unchecked_mut_player(player_index);
                player.send_packet(ToClientPacket::YourVoting { player_index: player.voting_variables.chosen_vote.clone() });
                player.send_packet(ToClientPacket::YourJudgement { verdict: player.voting_variables.verdict.clone() });
            },
            PhaseType::Testimony => {},
            PhaseType::Judgement => {},
            PhaseType::Evening => {},
            PhaseType::Night => {
                let new_night_variables =  PlayerNightVariables::reset(game, player_index);

                let player = game.get_unchecked_mut_player(player_index);

                player.night_variables =new_night_variables;
                player.send_packet(ToClientPacket::YourTarget { player_indices: player.chosen_targets().clone() });
            }
        }
    }

    
    pub fn send_packet(&self, packet: ToClientPacket){
        self.sender.send(packet);
    }
    fn requeue_chat_messages(&mut self){
        for msg in self.chat_messages.iter(){
            self.queued_chat_messages.push(msg.clone());
        }
    }
    pub fn send_chat_messages(&mut self){
        
        if self.queued_chat_messages.len() == 0 {
            return;
        }
        
        let mut chat_messages_out = vec![];

        //get the first 5
        for _ in 0..5{
            let msg_option = self.queued_chat_messages.get(0);
            if let Some(msg) = msg_option{
                chat_messages_out.push(msg.clone());
                self.queued_chat_messages.remove(0);
            }else{ break; }
        }
        
        self.send_packet(ToClientPacket::AddChatMessages { chat_messages: chat_messages_out });
        

        self.send_chat_messages();
    }
    fn send_available_buttons(&mut self, game: &mut Game){

        //TODO maybe find a way to check to see if we should send this like i do in chat messages
        self.send_packet(ToClientPacket::YourButtons { buttons: game.players.iter().map(|player|{
            YourButtons{
                vote: false,
                target: self.role().can_night_target(self.index, player.index, game),
                day_target: self.role().can_day_target(self.index, player.index, game),
            }
        }).collect()});
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

