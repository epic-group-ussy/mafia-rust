use serde::{Serialize, Deserialize};

use crate::game::ability_input::AbilityInput;
use crate::game::attack_power::AttackPower;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::Game;
use crate::vec_map::VecMap;

use super::{Priority, Role, RoleState, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Kira {
    pub guesses: VecMap<PlayerReference, KiraGuess>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum KiraGuess{
    None,
    #[default] NonTown,

    Jailor, Villager, Drunk,
    Detective, Lookout, Tracker, Psychic, Philosopher, Gossip, Auditor, Snoop, Spy, TallyClerk,
    Doctor, Bodyguard, Cop, Bouncer, Engineer, Armorsmith, Steward,
    Vigilante, Veteran, Marksman, Deputy, Rabblerouser,
    Escort, Medium, Retributionist, CrossingGuard, Reporter, Mayor, Transporter
}
impl KiraGuess{
    fn convert_to_guess(role: Role)->Option<KiraGuess>{
        match role {
            Role::Jailor => Some(Self::Jailor),
            Role::Villager => Some(Self::Villager),
            Role::Drunk => Some(Self::Drunk),

            Role::Detective => Some(Self::Detective),
            Role::Lookout => Some(Self::Lookout),
            Role::Tracker => Some(Self::Tracker),
            Role::Philosopher => Some(Self::Philosopher),
            Role::Psychic => Some(Self::Psychic),
            Role::Auditor => Some(Self::Auditor),
            Role::Snoop => Some(Self::Snoop),
            Role::Gossip => Some(Self::Gossip),
            Role::Spy => Some(Self::Spy),
            Role::TallyClerk => Some(Self::TallyClerk),

            Role::Doctor => Some(Self::Doctor),
            Role::Bodyguard => Some(Self::Bodyguard),
            Role::Cop => Some(Self::Cop),
            Role::Bouncer => Some(Self::Bouncer),
            Role::Engineer => Some(Self::Engineer),
            Role::Armorsmith => Some(Self::Armorsmith),
            Role::Steward => Some(Self::Steward),

            Role::Vigilante => Some(Self::Vigilante),
            Role::Veteran => Some(Self::Veteran),
            Role::Marksman => Some(Self::Marksman),
            Role::Deputy => Some(Self::Deputy),
            Role::Rabblerouser => Some(Self::Rabblerouser),

            Role::Escort => Some(Self::Escort),
            Role::Medium => Some(Self::Medium),
            Role::Retributionist => Some(Self::Retributionist),
            Role::CrossingGuard => Some(Self::CrossingGuard),
            Role::Reporter => Some(Self::Reporter),
            Role::Mayor => Some(Self::Mayor),
            Role::Transporter => Some(Self::Transporter),

            //Mafia
            Role::Godfather | Role::Mafioso | Role::Eros |
            Role::Counterfeiter | Role::Retrainer | Role::Recruiter | Role::Impostor | Role::MafiaKillingWildcard |
            Role::Goon |
            Role::Hypnotist | Role::Blackmailer | Role::Informant | 
            Role::MafiaWitch | Role::Necromancer | Role::Consort |
            Role::Mortician | Role::Framer | Role::Forger | 
            Role::Cupid | Role::MafiaSupportWildcard => Some(Self::NonTown),

            //Neutral
            Role::Jester | Role::Revolutionary | Role::Politician |
            Role::Doomsayer | Role::Death |
            Role::Witch | Role::Scarecrow | Role::Warper | Role::Kidnapper | Role::Chronokaiser |
            Role::Wildcard | Role::TrueWildcard => Some(Self::NonTown),
            Role::Martyr => None,

            //Fiends
            Role::Arsonist | Role::Werewolf | 
            Role::Ojo | Role::Puppeteer | Role::Pyrolisk | Role::Kira | 
            Role::SerialKiller |
            Role::FiendsWildcard => Some(Self::NonTown),
            
            //Cult
            Role::Apostle | Role::Disciple | Role::Zealot => Some(Self::NonTown),
        }
    }
    fn guess_matches_role(&self, role: Role)->bool{
        if let Some(guess) = Self::convert_to_guess(role) {
            *self == guess
        }else{
            false
        }
    }
    fn is_in_game(&self, game: &Game)->bool{
        PlayerReference::all_players(game).into_iter().any(|player_ref| {
            let role = player_ref.role(game);
            self.guess_matches_role(role) && player_ref.alive(game)
        })
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct KiraResult {
    pub guesses: VecMap<PlayerReference, (KiraGuess, KiraGuessResult)>,
}
impl KiraResult{
    pub fn new(guesses: VecMap<PlayerReference, KiraGuess>, game: &Game)->Self{
        Self{
            guesses: guesses.into_iter().map(|(player_ref, guess)|{
                let result = if guess.guess_matches_role(player_ref.role(game)){
                    KiraGuessResult::Correct
                }else if guess.is_in_game(game) {
                    KiraGuessResult::WrongSpot
                }else{
                    KiraGuessResult::NotInGame
                };
                (player_ref, (guess, result))
            }).collect()
        }
    }
    pub fn all_correct(&self)->bool{
        self.guesses.iter().all(|(_, (guess, result))| 
            *result == KiraGuessResult::Correct || *guess == KiraGuess::None
        )
    }
}
impl Ord for KiraResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.guesses.len().cmp(&other.guesses.len())
    }
}
impl PartialOrd for KiraResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KiraGuessResult {
    Correct,    //green
    NotInGame,  //black
    WrongSpot,  //yellow
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct KiraAbilityInput(Vec<(PlayerReference, KiraGuess)>);

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armor;

impl RoleStateImpl for Kira {
    type ClientRoleState = Kira;
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if actor_ref.night_blocked(game) {return;}
        if !actor_ref.alive(game) {return;}

        let result = KiraResult::new(self.guesses.clone(), game);

        match priority {
            Priority::Kill if result.all_correct() => {
                if game.day_number() == 1 {return};
                
                for (player, (guess, result)) in result.guesses.iter(){
                    if player.alive(game) && *result == KiraGuessResult::Correct && *guess != KiraGuess::None {
                        player.try_night_kill_single_attacker(actor_ref, game, GraveKiller::Role(super::Role::Kira), AttackPower::ArmorPiercing, true);
                    }
                }
            },
            Priority::Investigative => {
                actor_ref.push_night_message(game, ChatMessageVariant::KiraResult { result });
            },
            _ => return,
        }    
    }
    fn on_ability_input_received(mut self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: crate::game::ability_input::AbilityInput) {
        if actor_ref != input_player {return};
        let AbilityInput::Kira{input: KiraAbilityInput(input)} = ability_input else {return};
        
        let mut new_guesses: VecMap<PlayerReference, KiraGuess> = VecMap::new();

        for (player_ref, guess) in input {
            if Kira::allowed_to_guess(actor_ref, player_ref, game){
                new_guesses.insert(player_ref, guess);
            }
        }

        self.guesses = new_guesses;
        actor_ref.set_role_state(game, self);
        Kira::set_guesses(actor_ref, game);
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType) {
        Kira::set_guesses(actor_ref, game);
    }
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {
        Kira::set_guesses(actor_ref, game);
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
        Kira::set_guesses(actor_ref, game);
    }
}

impl Kira{
    pub fn set_guesses(kira_player_ref: PlayerReference, game: &mut Game){
        
        let RoleState::Kira(mut kira) = kira_player_ref.role_state(game).clone() else {return};

        kira.guesses.retain(|player_ref, _|
            Kira::allowed_to_guess(kira_player_ref, *player_ref, game)
        );

        for player_ref in PlayerReference::all_players(game){
            if
                !kira.guesses.contains_key(&player_ref) &&
                Kira::allowed_to_guess(kira_player_ref, player_ref, game)
            {
                kira.guesses.insert(player_ref, KiraGuess::None);
            }
        }

        kira_player_ref.set_role_state(game, RoleState::Kira(kira));
    }
    pub fn allowed_to_guess(kira_player_ref: PlayerReference, player_ref: PlayerReference, game: &mut Game)->bool{
        player_ref.alive(game) &&
        player_ref != kira_player_ref
    }
}