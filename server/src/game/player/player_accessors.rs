use std::collections::HashMap;

use crate::{
    game::{
        role::{RoleData, Role}, 
        Game, 
        verdict::Verdict, 
        chat::{
            ChatGroup, 
            ChatMessage, 
            night_message::NightInformation
        }, 
        visit::Visit, 
        grave::{GraveRole, GraveKiller}}, packet::ToClientPacket, 
    };
use super::PlayerReference;


impl PlayerReference{
    pub fn name<'a>(&self, game: &'a Game) -> &'a String {
        &self.deref(game).name
    }
    
    pub fn role(&self, game: &Game) -> Role {
        self.deref(game).role_data.role()
    }
    pub fn role_data<'a>(&self, game: &'a Game) -> &'a RoleData{
        &self.deref(game).role_data
    }
    pub fn set_role_data(&self, game: &mut Game, new_role_data: RoleData){
        
        if self.deref(game).role_data.role() == new_role_data.role() {
            self.send_packet(game, ToClientPacket::YourRole { role: self.deref(game).role_data.role() });
        }
        self.deref_mut(game).role_data = new_role_data;
        self.send_packet(game, ToClientPacket::YourRoleData { role_data: self.deref(game).role_data.clone() } );
    }

    pub fn alive(&self, game: &Game) -> bool{
        self.deref(game).alive
    }
    pub fn set_alive(&self, game: &mut Game, alive: bool){
        self.deref_mut(game).alive = alive;

        let mut alive_players = vec![];
        for player in PlayerReference::all_players(game){
            alive_players.push(player.deref(game).alive);
        }
        game.send_packet_to_all(ToClientPacket::PlayerAlive { alive: alive_players });
    }

    pub fn will<'a>(&self, game: &'a Game) -> &'a String {
        &self.deref(game).will
    }
    pub fn set_will(&self, game: &mut Game, will: String){
        self.deref_mut(game).will = will;
        self.send_packet(game, ToClientPacket::YourWill { will: self.deref(game).will.clone() });
    }
    
    pub fn notes<'a>(&self, game: &'a Game) -> &'a String {
        &self.deref(game).notes
    }
    pub fn set_notes(&self, game: &mut Game, notes: String){
        self.deref_mut(game).notes = notes;
        self.send_packet(game, ToClientPacket::YourNotes { notes: self.deref(game).notes.clone() })
    }
    
    pub fn role_labels<'a>(&self, game: &'a Game) -> &'a HashMap<PlayerReference, Role>{
        &self.deref(game).role_labels
    }  
    pub fn insert_role_label(&self, game: &mut Game, key: PlayerReference, value: Role){
        self.deref_mut(game).role_labels.insert(key, value);
        self.send_packet(game, ToClientPacket::YourRoleLabels { role_labels: PlayerReference::ref_map_to_index(self.deref(game).role_labels.clone()) });
    }

    pub fn add_chat_message(&self, game: &mut Game, message: ChatMessage) {
        self.deref_mut(game).chat_messages.push(message.clone());
        self.deref_mut(game).queued_chat_messages.push(message);
    }
    pub fn add_chat_messages(&self, game: &mut Game, messages: Vec<ChatMessage>){
        for message in messages.into_iter(){
            self.add_chat_message(game, message);
        }
    }

    //VOTING
    pub fn chosen_vote(&self, game: &Game) -> Option<PlayerReference>{
        self.deref(game).voting_variables.chosen_vote
    }
    /// returns true if players vote was changed and packet was sent
    /// ### checks
    /// - player is alive
    /// - player is not silenced
    /// - player is not voting itself
    /// - player is not voting a dead player
    pub fn set_chosen_vote(&self, game: &mut Game, chosen_vote: Option<PlayerReference>, send_chat_message: bool) -> bool{
        if chosen_vote == self.deref(game).voting_variables.chosen_vote ||
            !self.deref(game).alive || self.night_silenced(game) {
            self.deref_mut(game).voting_variables.chosen_vote = None;
            self.send_packet(game, ToClientPacket::YourVoting { 
                player_index: None
            });
            return false;
        }
        
        if let Some(chosen_vote) = chosen_vote {
            if chosen_vote == *self || !chosen_vote.deref(game).alive {
                self.deref_mut(game).voting_variables.chosen_vote = None;
                self.send_packet(game, ToClientPacket::YourVoting { 
                    player_index: None
                });
                return false;
            }
        }
        
        self.deref_mut(game).voting_variables.chosen_vote = chosen_vote;
        self.send_packet(game, ToClientPacket::YourVoting { 
            player_index: self.chosen_vote(game).as_ref().map(PlayerReference::index)
        });
        let player_votes_packet = ToClientPacket::new_player_votes(game);
        game.send_packet_to_all(player_votes_packet);
        
        if send_chat_message {
            game.add_message_to_chat_group(ChatGroup::All, ChatMessage::Voted{
                voter: self.index(), 
                votee: chosen_vote.as_ref().map(PlayerReference::index)
            });
        }
        
        true
    }

    
    pub fn verdict(&self, game: &Game) -> Verdict{
        self.deref(game).voting_variables.verdict
    }
    pub fn set_verdict(&self, game: &mut Game, verdict: Verdict, send_chat_message: bool){
        
        
        self.send_packet(game, ToClientPacket::YourJudgement { verdict });
        self.deref_mut(game).voting_variables.verdict = verdict;

        if send_chat_message {
            game.add_message_to_chat_group(
                ChatGroup::All, 
                ChatMessage::JudgementVote{ 
                    voter_player_index: self.index() 
                }
            );
        }
    }

    //NIGHT
    pub fn night_alive_tonight(&self, game: &Game) -> bool {
        self.deref(game).night_variables.alive_tonight
    }
    pub fn set_night_alive_tonight(&self, game: &mut Game, alive_tonight: bool){
        self.deref_mut(game).night_variables.alive_tonight = alive_tonight;
    }
    
    pub fn night_died(&self, game: &Game) -> bool {
        self.deref(game).night_variables.died
    }
    pub fn set_night_died(&self, game: &mut Game, died: bool){
        self.deref_mut(game).night_variables.died = died;
    }

    pub fn night_attacked(&self, game: &Game) -> bool {
        self.deref(game).night_variables.attacked
    }
    pub fn set_night_attacked(&self, game: &mut Game, attacked: bool){
        self.deref_mut(game).night_variables.attacked = attacked;
    }

    pub fn night_roleblocked(&self, game: &Game) -> bool {
        self.deref(game).night_variables.roleblocked
    }
    pub fn set_night_roleblocked(&self, game: &mut Game, roleblocked: bool){
        self.deref_mut(game).night_variables.roleblocked = roleblocked;
    }

    pub fn night_defense(&self, game: &Game) -> u8 {
        self.deref(game).night_variables.defense
    }
    pub fn set_night_defense(&self, game: &mut Game, defense: u8){
        self.deref_mut(game).night_variables.defense = defense;
    }

    pub fn night_appeared_suspicious(&self, game: &Game) -> bool {
        self.deref(game).night_variables.appeard_suspicious
    }
    pub fn set_night_appeared_suspicious(&self, game: &mut Game, suspicious: bool){
        self.deref_mut(game).night_variables.appeard_suspicious = suspicious;
    }

    pub fn night_appeared_visits<'a>(&self, game: &'a Game) -> &'a Vec<Visit>{
        &self.deref(game).night_variables.appeared_visits
    }
    pub fn set_night_appeared_visits(&self, game: &mut Game, appeared_visits: Vec<Visit>){
        self.deref_mut(game).night_variables.appeared_visits = appeared_visits;
    }
    
    pub fn chosen_targets<'a>(&self, game: &'a Game) -> &'a Vec<PlayerReference>{
        &self.deref(game).night_variables.chosen_targets
    }
    pub fn set_chosen_targets(&self, game: &mut Game, chosen_targets: Vec<PlayerReference>){
        self.deref_mut(game).night_variables.chosen_targets = vec![];

        let role = self.deref(game).role_data.role();

        for target_ref in chosen_targets {
            if role.can_night_target(game, *self, target_ref){
                self.deref_mut(game).night_variables.chosen_targets.push(target_ref);
            }
        }

        let packet = ToClientPacket::YourTarget { 
            player_indices: PlayerReference::ref_vec_to_index(
                &self.deref(game).night_variables.chosen_targets
            )
        };
        self.send_packet(game, packet);
    }

    pub fn night_visits<'a>(&self, game: &'a Game) -> &'a Vec<Visit>{
        &self.deref(game).night_variables.visits
    }
    pub fn set_night_visits(&self, game: &mut Game, visits: Vec<Visit>){
        self.deref_mut(game).night_variables.visits = visits;
    }

    pub fn night_messages<'a>(&self, game: &'a Game) -> &'a Vec<NightInformation> {
        &self.deref(game).night_variables.messages
    }
    pub fn push_night_messages(&self, game: &mut Game, message: NightInformation){
        self.deref_mut(game).night_variables.messages.push(message);
    }
    pub fn set_night_messages(&self, game: &mut Game, messages: Vec<NightInformation>){
        self.deref_mut(game).night_variables.messages = messages;
    }

    pub fn night_grave_role<'a>(&self, game: &'a Game) -> &'a GraveRole {
        &self.deref(game).night_variables.grave_role
    }
    pub fn set_night_grave_role(&self, game: &mut Game, grave_role: GraveRole){
        self.deref_mut(game).night_variables.grave_role = grave_role;
    }

    pub fn night_grave_killers<'a>(&self, game: &'a Game) -> &'a Vec<GraveKiller> {
        &self.deref(game).night_variables.grave_killers
    }
    pub fn push_night_grave_killers(&self, game: &mut Game, grave_killer: GraveKiller){
        self.deref_mut(game).night_variables.grave_killers.push(grave_killer);
    }
    pub fn set_night_grave_killers(&self, game: &mut Game, grave_killers: Vec<GraveKiller>){
        self.deref_mut(game).night_variables.grave_killers = grave_killers;
    }

    pub fn night_grave_will<'a>(&self, game: &'a Game) -> &'a String {
        &self.deref(game).night_variables.grave_will
    }
    pub fn set_night_grave_will(&self, game: &mut Game, grave_will: String){
        self.deref_mut(game).night_variables.grave_will = grave_will;
    }

    pub fn night_grave_death_notes<'a>(&self, game: &'a Game) -> &'a Vec<String> {
        &self.deref(game).night_variables.grave_death_notes
    }
    pub fn push_night_grave_death_notes(&self, game: &mut Game, death_note: String){
        self.deref_mut(game).night_variables.grave_death_notes.push(death_note);
    }
    pub fn set_night_grave_death_notes(&self, game: &mut Game, grave_death_notes: Vec<String>){
        self.deref_mut(game).night_variables.grave_death_notes = grave_death_notes;
    }

    pub fn night_jailed(&self, game: &Game) -> bool {
        self.deref(game).night_variables.jailed
    }
    ///add chat message saying that they were jailed, and sends packet
    pub fn set_night_jailed(&self, game: &mut Game, jailed: bool){
        if jailed {
            self.send_packet(game, ToClientPacket::YouAreJailed);

            let mut message_sent = false;
            for chat_group in self.role(game).get_current_send_chat_groups(game, *self){
                match chat_group {
                    ChatGroup::All | ChatGroup::Dead | ChatGroup::Jail => {},
                    ChatGroup::Mafia | ChatGroup::Vampire | ChatGroup::Coven => {
                        game.add_message_to_chat_group(
                            chat_group,
                            ChatMessage::JailedSomeone { player_index: self.index() }
                        );
                        message_sent = true;
                    },
                }
            }
            if !message_sent {
                self.add_chat_message(game, 
                    ChatMessage::JailedSomeone { player_index: self.index() }
                );
            }
        }
        self.deref_mut(game).night_variables.jailed = jailed;
    }

    pub fn night_silenced(&self, game: &Game) -> bool {
        self.deref(game).night_variables.silenced
    }
    pub fn set_night_silenced(&self, game: &mut Game, silenced: bool){
        self.deref_mut(game).night_variables.silenced = silenced;
        if silenced {
            self.send_packet(game, ToClientPacket::YouAreSilenced);
            self.push_night_messages(game, NightInformation::Silenced);
        }
    }
}



