mod kit;
use std::{ops::Deref, vec};

pub(crate) use kit::{assert_contains, assert_not_contains};

use mafia_server::game::ability_input::{ability_selection::AbilitySelection, ControllerID};
pub use mafia_server::game::{
    chat::{ChatMessageVariant, MessageSender, ChatGroup}, 
    grave::*,
    ability_input::{
        selection_type::{
            two_role_option_selection::TwoRoleOptionSelection,
            two_role_outline_option_selection::TwoRoleOutlineOptionSelection
        },
        AbilityInput,
    }, 
    components::{cult::CultAbility, insider_group::InsiderGroupID},  
    role_list::RoleSet, 
    role_outline_reference::RoleOutlineReference,
     
    player::PlayerReference,
    tag::Tag,
    verdict::Verdict,
    role::{
        Role,
        RoleState,

        jailor::Jailor,
        villager::Villager,
        
        detective::Detective,
        snoop::Snoop,
        lookout::Lookout,
        spy::{Spy, SpyBug},
        tracker::Tracker,
        philosopher::Philosopher,
        psychic::Psychic,
        gossip::Gossip,
        auditor::Auditor,
        
        doctor::Doctor,
        bodyguard::Bodyguard,
        cop::Cop,
        bouncer::Bouncer,
        engineer::Engineer,

        vigilante::Vigilante,
        veteran::Veteran,
        deputy::Deputy,
        marksman::Marksman,
        rabblerouser::Rabblerouser,
        
        transporter::Transporter,
        escort::Escort,
        mayor::Mayor,
        medium::Medium,
        retributionist::Retributionist,

        godfather::Godfather,
        impostor::Impostor,
        recruiter::Recruiter,
        counterfeiter::Counterfeiter,
        mafioso::Mafioso,
        
        framer::Framer,
        hypnotist::Hypnotist,
        blackmailer::Blackmailer,
        informant::Informant,
        mafia_witch::MafiaWitch,
        necromancer::Necromancer,
        mortician::Mortician,
        mafia_support_wildcard::MafiaSupportWildcard, 
        

        jester::Jester,
        revolutionary::Revolutionary,
        geist::Geist,
        witch::Witch,
        politician::Politician,
        doomsayer::{Doomsayer, DoomsayerGuess},
        wild_card::Wildcard,
        martyr::Martyr,

        apostle::Apostle,
        zealot::Zealot,
        
        arsonist::Arsonist,
        spiral::Spiral,
        pyrolisk::Pyrolisk,
        puppeteer::{Puppeteer, PuppeteerAction},
        fiends_wildcard::FiendsWildcard,

        armorsmith::Armorsmith, auditor::AuditorResult,
        drunk::Drunk, ojo::Ojo,
        scarecrow::Scarecrow, tally_clerk::TallyClerk,
        warper::Warper
    }, 
    phase::{
        PhaseState, 
        PhaseType::{self, *}
    }
};
// Pub use so that submodules don't have to reimport everything.
pub use mafia_server::packet::ToServerPacket;

#[test]
fn no_unwanted_tags() {
    kit::scenario!(game in Dusk 1 where
        jester: Jester,
        townie: Detective,
        mafioso: Godfather,
        mortician: Mortician,
        politician: Politician,
        spy: Spy,
        lookout: Lookout,
        detective: Detective,
        arsonist: Arsonist,
        vigilante: Vigilante,
        puppeteer: Puppeteer,
        warper: Warper,
        witch: Witch
    );

    assert!(jester.get_player_tags().is_empty());
    assert!(townie.get_player_tags().is_empty());
    assert!(mafioso.get_player_tags().is_empty());
    assert!(mortician.get_player_tags().is_empty());
    assert!(politician.get_player_tags().is_empty());
    assert!(spy.get_player_tags().is_empty());
    assert!(lookout.get_player_tags().is_empty());
    assert!(detective.get_player_tags().is_empty());
    assert!(arsonist.get_player_tags().is_empty());
    assert!(vigilante.get_player_tags().is_empty());
    assert!(puppeteer.get_player_tags().is_empty());
    assert!(warper.get_player_tags().is_empty());
    assert!(witch.get_player_tags().is_empty());
}

#[test]
fn detective_basic() {
    kit::scenario!(game in Night 1 where
        sher: Detective,
        mafia: Mafioso,
        townie: Detective
    );
    sher.send_ability_input_player_list_typical(mafia);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );

    game.skip_to(Night, 2);
    sher.send_ability_input_player_list_typical(townie);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(2),
        ChatMessageVariant::SheriffResult { suspicious: false }
    );
}

#[test]
fn detective_neutrals(){
    kit::scenario!(game in Night 1 where
        sher: Detective,
        _mafia: Godfather,
        witch: Witch,
        jester: Jester,
        politician: Politician
    );

    sher.send_ability_input_player_list_typical(witch);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );
    
    game.skip_to(Night, 2);
    sher.send_ability_input_player_list_typical(jester);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(2),
        ChatMessageVariant::SheriffResult { suspicious: false }
    );
    
    game.skip_to(Night, 3);
    sher.send_ability_input_player_list_typical(politician);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(3),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );

}

#[test]
fn auditor_standard_double_audit(){
    kit::scenario!(game in Night 1 where
        auditor: Auditor,
        _townie: Auditor,
        _mafioso: Mafioso
    );

    let input = AbilityInput::new(
        ControllerID::role(auditor.player_ref(), Role::Auditor, 0), 
        AbilitySelection::TwoRoleOutlineOption { selection: TwoRoleOutlineOptionSelection(
            RoleOutlineReference::new(&game, 0), 
            RoleOutlineReference::new(&game, 1)
        ) }
    );

    auditor.send_ability_input(input);
    game.next_phase();
    
    let messages = auditor.get_messages_after_night(1);

    let mut results: u8 = 0;
    let mut found_auditor = false;
    for message in messages.iter() {
        if let ChatMessageVariant::AuditorResult { role_outline: _, result } = message {
            results += 1;
            if match result {
                AuditorResult::Two { roles } => roles.contains(&Role::Auditor),
                AuditorResult::One { role } => *role == Role::Auditor,
            } {
                found_auditor = true;
            }
        }
    }
    assert!(results == 2);
    assert!(found_auditor);
}

#[test]
fn mortician_obscures_on_stand(){
    kit::scenario!(game in Night 1 where
        mortician: Mortician,
        townie: Detective,
        jail: Jailor,
        gf: Godfather
    );

    mortician.send_ability_input_player_list_typical(townie);
    game.skip_to(Nomination, 2);
    
    jail.vote_for_player(Some(townie));
    gf.vote_for_player(Some(townie));
    mortician.vote_for_player(Some(townie));

    game.skip_to(Judgement, 2);
    jail.set_verdict(Verdict::Guilty);

    game.skip_to(Night, 2);
    
    assert_eq!(game.graves[0].information, GraveInformation::Obscured);
    assert_contains!(mortician.get_messages(), ChatMessageVariant::PlayerRoleAndAlibi { player: townie.player_ref(), role: Role::Detective, will: "".to_string() });
}

#[test]
fn mortician_obscures_fail_after_death(){
    kit::scenario!(game in Night 1 where
        mortician: Mortician,
        townie: Detective,
        jail: Jailor,
        gf: Godfather
    );

    mortician.send_ability_input_player_list_typical(townie);
    game.skip_to(Nomination, 2);
    
    jail.vote_for_player(Some(mortician));
    gf.vote_for_player(Some(mortician));
    townie.vote_for_player(Some(mortician));

    game.skip_to(Judgement, 2);
    jail.set_verdict(Verdict::Guilty);

    game.skip_to(Night, 2);
    gf.send_ability_input_player_list_typical(townie);
    game.next_phase();
    assert!(matches!(game.graves[1].information, GraveInformation::Normal { role: Role::Detective, .. }));
    assert_not_contains!(mortician.get_messages(), ChatMessageVariant::PlayerRoleAndAlibi { player: townie.player_ref(), role: Role::Detective, will: "".to_string() });
}

#[test]
fn detective_godfather() {
    kit::scenario!(game in Night 1 where
        sher: Detective,
        mafia: Godfather
    );
    sher.send_ability_input_player_list_typical(mafia);
    game.next_phase();
    assert_contains!(sher.get_messages(), ChatMessageVariant::SheriffResult { suspicious: false });
}

#[test]
fn philosopher_basic() {
    kit::scenario!(game in Night 1 where
        philosopher: Philosopher,
        mafia1: Mafioso,
        mafia2: Informant,
        townie1: Detective,
        townie2: Vigilante
    );

    philosopher.send_ability_input_two_player_typical(mafia1, townie1);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(1),
        ChatMessageVariant::SeerResult { enemies: true }
    );

    game.skip_to(Night, 2);
    philosopher.send_ability_input_two_player_typical(mafia1, mafia2);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(2),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 3);
    philosopher.send_ability_input_two_player_typical(townie2, townie1);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(3),
        ChatMessageVariant::SeerResult { enemies: false }
    );
}

#[test]
fn philosopher_neutrals() {
    kit::scenario!(game in Night 1 where
        philosopher: Philosopher,
        mafia1: Mafioso,
        townie1: Vigilante,
        jester: Jester,
        witch: Witch
    );

    game.skip_to(Night, 3);
    philosopher.send_ability_input_two_player_typical(jester, mafia1);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(3),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 4);
    philosopher.send_ability_input_two_player_typical(townie1, jester);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(4),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 6);
    philosopher.send_ability_input_two_player_typical(townie1, witch);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(6),
        ChatMessageVariant::SeerResult { enemies: true }
    );

    game.skip_to(Night, 7);
    philosopher.send_ability_input_two_player_typical(jester, witch);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(7),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 8);
    philosopher.send_ability_input_two_player_typical(mafia1, witch);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(8),
        ChatMessageVariant::SeerResult { enemies: false }
    );
}

#[test]
fn jester_basic() {
    kit::scenario!(game in Night 1 where
        jester: Jester,
        townie: Detective,
        mafia: Godfather,
        lookout1: Lookout,
        lookout2: Lookout,
        mafia2: Mafioso
    );

    game.skip_to(Nomination, 2);
    mafia.vote_for_player(Some(jester));
    townie.vote_for_player(Some(jester));
    lookout1.vote_for_player(Some(jester));
    lookout2.vote_for_player(Some(jester));


    game.skip_to(Judgement, 2);
    townie.set_verdict(mafia_server::game::verdict::Verdict::Guilty);
    mafia.set_verdict(mafia_server::game::verdict::Verdict::Guilty);
    mafia2.set_verdict(mafia_server::game::verdict::Verdict::Guilty);
    lookout1.set_verdict(mafia_server::game::verdict::Verdict::Innocent);
    lookout2.set_verdict(mafia_server::game::verdict::Verdict::Innocent);

    game.skip_to(Night, 2);
    assert!(!jester.alive());
    lookout1.send_ability_input_player_list_typical(townie);
    lookout2.send_ability_input_player_list_typical(mafia);


    game.skip_to(Obituary, 3);

    assert_eq!(
        PlayerReference::all_players(&game)
            .filter(|p|!p.alive(&game)).count(), 2
    );
    assert_contains!(
        lookout1.get_messages(), 
        ChatMessageVariant::LookoutResult { players: vec![] }
    );
    assert_contains!(
        lookout2.get_messages(), 
        ChatMessageVariant::LookoutResult { players: vec![] }
    );
}

#[test]
fn rabble_rouser_dies(){
    kit::scenario!(game in Night 1 where
        exe: Revolutionary,
        townie: Detective,
        mafioso: Mafioso
    );

    game.skip_to(Nomination, 2);
    mafioso.vote_for_player(Some(townie));
    exe.vote_for_player(Some(townie));

    game.skip_to(Judgement, 2);
    exe.set_verdict(Verdict::Guilty);
    mafioso.set_verdict(Verdict::Guilty);

    game.skip_to(FinalWords, 2);
    assert!(!exe.alive());
    assert!(townie.alive());
    assert!(mafioso.alive());
}

#[test]
fn psychic_auras(){
    for _ in 0..100 {
        kit::scenario!(game in Night 1 where
            psy: Psychic,
            god: Godfather,
            maf: Framer,
            town1: Detective,
            town2: Vigilante
        );

        psy.send_ability_input_player_list_typical(maf);
    
        game.next_phase();
        let messages = psy.get_messages_after_night(1);
        let messages: Vec<_> = 
            messages.into_iter()
            .filter(|msg|match msg {
                ChatMessageVariant::PsychicEvil { players } => {
                    players.contains(&maf.index()) &&
                    !players.contains(&god.index()) &&
                    !players.contains(&psy.index())
                }
                _ => false
            }).collect();

        if messages.len() != 1 {
            panic!("{:?}", messages);
        }

        game.skip_to(Night, 2);
        maf.send_ability_input_player_list_typical(town1);
        psy.send_ability_input_player_list_typical(town2);
        game.next_phase();
        let messages = psy.get_messages_after_night(2);
        let messages: Vec<_> = 
            messages.into_iter()
            .filter(|msg|match msg {
                ChatMessageVariant::PsychicGood { players } => {
                    players.contains(&town2.index()) &&
                    !players.contains(&town1.index()) &&
                    !players.contains(&psy.index())
                }
                _ => false
            }).collect();

        if messages.len() != 1 {
            panic!("{:?}", messages);
        }
    }
}

#[test]
fn tally_clerk_basic(){
    kit::scenario!(game in Nomination 2 where
        fg: TallyClerk,
        townie: Detective,
        mafioso: Mafioso
    );

    fg.vote_for_player(Some(townie));
    mafioso.vote_for_player(Some(townie));

    game.skip_to(Judgement, 2);

    fg.set_verdict(Verdict::Guilty);
    mafioso.set_verdict(Verdict::Guilty);
    
    game.skip_to(Obituary, 3);
    assert_contains!(
        fg.get_messages_after_night(1),
        ChatMessageVariant::TallyClerkResult { evil_count: 1 }
    );
}


#[test]
fn spy_basic_transported() {
    kit::scenario!(game in Night 1 where
        spy: Spy,
        _mafioso: Mafioso,
        blackmailer: Blackmailer,
        esc: Escort,
        transp: Transporter,
        bugged: Detective,
        jester: Jester,
        witch: Witch
    );
    spy.send_ability_input_player_list_typical(jester);
    transp.send_ability_input_two_player_typical(jester, bugged);
    blackmailer.send_ability_input_player_list_typical(jester);
    esc.send_ability_input_player_list_typical(jester);
    witch.send_ability_input_two_player_typical(jester, esc);

    game.next_phase();

    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Silenced });
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Roleblocked });
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Transported });
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Possessed });

    
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyMafiaVisit { players: vec![bugged.index()] });
}

#[test]
fn bodyguard_basic() {
    kit::scenario!(game in Night 2 where
        maf: Mafioso,
        bg: Bodyguard,
        townie: Detective
    );

    maf.send_ability_input_player_list_typical(townie);
    bg.send_ability_input_player_list_typical(townie);

    game.skip_to(Obituary, 3);

    assert!(townie.get_messages().contains(&ChatMessageVariant::YouWereProtected));

    assert!(townie.alive());
    assert!(!bg.alive());
    assert!(!maf.alive());
}

/// Tests if transporter properly swaps, redirecting actions on their first target to their
/// second. The vigilante will try to kill town1, which should end up killing town2.
#[test]
fn transporter_basic_vigilante_escort() {
    kit::scenario!(game in Night 2 where
        trans: Transporter,
        vigi: Vigilante,
        escort: Escort,
        town1: Detective,
        town2: Detective
    );
    trans.send_ability_input_two_player_typical(town1, town2);
    vigi.send_ability_input_player_list_typical(town1);
    escort.send_ability_input_player_list_typical(town2);

    game.skip_to(Obituary, 3);
    assert!(town1.alive());
    assert!(!town2.alive());

    assert!(town1.was_roleblocked());
    assert!(!town2.was_roleblocked());
    
    game.skip_to(Obituary, 4);
    assert!(!vigi.alive());
}

#[test]
fn transporter_basic_seer_sheriff_framer() {
    kit::scenario!(game in Night 1 where
        trans: Transporter,
        philosopher: Philosopher,
        _mafioso: Mafioso,
        framer: Framer,
        town1: Detective,
        town2: Detective
    );
    assert!(trans.send_ability_input_two_player_typical(town1, town2));
    assert!(framer.send_ability_input_player_list_typical(town1));
    assert!(framer.send_ability_input_player_list(philosopher, 1));
    assert!(philosopher.send_ability_input_two_player_typical(town1, town2));
    assert!(town1.send_ability_input_player_list_typical(town2));
    assert!(town2.send_ability_input_player_list_typical(town1));

    game.skip_to(Obituary, 2);
    assert_contains!(
        philosopher.get_messages_after_night(1),
        ChatMessageVariant::SeerResult { enemies: true }
    );
    assert_contains!(
        town1.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: false }
    );
    assert_contains!(
        town2.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );
}

/// Test that the bodyguard protects the person their target was swapped with
#[test]
fn bodyguard_protects_transported_target() {
    kit::scenario!(game in Night 2 where
        trans: Transporter,
        maf: Mafioso,
        bg: Bodyguard,
        t1: Detective,
        t2: Detective
    );
    trans.send_ability_input_two_player_typical(t1, t2);
    maf.send_ability_input_player_list_typical(t1);
    bg.send_ability_input_player_list_typical(t1);
    
    game.skip_to(Obituary, 3);
    assert!(t1.alive());
    assert!(t2.alive());
    assert!(trans.alive());
    assert!(!bg.alive());
    assert!(!maf.alive());

    assert_not_contains!(t1.get_messages(), ChatMessageVariant::YouWereProtected);
    assert_contains!(t2.get_messages(), ChatMessageVariant::YouWereProtected);
    assert_contains!(bg.get_messages(), ChatMessageVariant::TargetWasAttacked);
}

#[test]
fn mayor_reveals_after_they_vote(){
    kit::scenario!(game where
        mayor: Mayor,
        _townie: Detective,
        mafioso: Mafioso
    );

    game.skip_to(Nomination, 2);
    mayor.vote_for_player(Some(mafioso));
    mayor.send_ability_input_unit_typical();
    assert_eq!(game.current_phase().phase(), Testimony);
}


#[test]
fn retributionist_basic(){
    kit::scenario!(game in Night 2 where
        ret: Retributionist,
        sher1: Detective,
        sher2: Detective,
        mafioso: Mafioso
    );

    mafioso.send_ability_input_player_list_typical(sher1);
    game.skip_to(Night, 3);
    mafioso.send_ability_input_player_list_typical(sher2);
    game.skip_to(Night, 4);

    assert!(!sher1.alive());
    assert!(!sher2.alive());

    assert!(ret.send_ability_input_two_player_typical(sher1, mafioso));
    game.next_phase();
    assert_contains!(
        ret.get_messages_after_night(4),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SheriffResult{ suspicious: true }
        )}
    );

    game.skip_to(Night, 5);
    assert!(ret.send_ability_input_two_player_typical(sher1, mafioso));
    game.next_phase();
    assert_contains!(
        ret.get_messages_after_night(5),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SheriffResult{ suspicious: true }
        )}
    );

    game.skip_to(Night, 6);
    ret.send_ability_input_two_player_typical(sher1, mafioso);
    game.next_phase();
    assert_not_contains!(
        ret.get_messages_after_night(6),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SheriffResult{ suspicious: true }
        )}
    );
}

#[test]
fn necromancer_basic(){
    kit::scenario!(game in Night 2 where
        ret: Necromancer,
        sher: Snoop,
        informant: Informant,
        mafioso: Mafioso,
        vigilante: Vigilante
    );
    
    mafioso.send_ability_input_player_list_typical(sher);
    game.skip_to(Night, 3);
    vigilante.send_ability_input_player_list_typical(informant);
    game.skip_to(Night, 4);



    assert!(ret.send_ability_input_two_player_typical(sher, mafioso));
    game.next_phase();
    assert_contains!(
        ret.get_messages_after_night(3),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SnoopResult { townie: false }
        )}
    );
}

#[test]
fn witch_basic(){
    kit::scenario!(game in Night 1 where
        witch: Witch,
        sher: Detective,
        informant: Informant,
        mafioso: Mafioso,
        philosopher: Philosopher
    );

    assert!(witch.send_ability_input_two_player_typical(sher, mafioso));
    game.next_phase();
    assert_contains!(witch.get_messages(), ChatMessageVariant::TargetsMessage{message: Box::new(
        ChatMessageVariant::SheriffResult{ suspicious: true }
    )});
    
    game.skip_to(Night, 2);
    assert!(philosopher.send_ability_input_two_player_typical(sher, informant));
    assert!(witch.send_ability_input_two_player_typical(philosopher, mafioso));
    game.next_phase();
    assert_contains!(
        witch.get_messages_after_night(2),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SeerResult { enemies: false }
        )}
    );
}

#[test]
fn cop_basic(){
    kit::scenario!(game in Night 2 where
        crus: Cop,
        protected: Jester,
        townie1: Detective,
        townie2: Detective,
        mafioso: Mafioso,
        _maf2: Framer
    );

    crus.send_ability_input_player_list_typical(protected);
    townie1.send_ability_input_player_list_typical(protected);
    townie2.send_ability_input_player_list_typical(protected);
    mafioso.send_ability_input_player_list_typical(protected);

    game.skip_to(Night, 3);

    assert!(crus.alive());
    assert!(protected.alive());
    assert!(townie1.alive());
    assert!(townie2.alive());
    assert!(!mafioso.alive());

    crus.send_ability_input_player_list_typical(protected);
    townie1.send_ability_input_player_list_typical(protected);
    townie2.send_ability_input_player_list_typical(protected);

    game.next_phase();
    
    assert!(crus.alive());
    assert!(protected.alive());
    assert!(townie1.alive() || townie2.alive());
    assert!(!townie1.alive() || !townie2.alive());
}

#[test]
fn cop_does_not_kill_framed_player(){
    kit::scenario!(game in Night 2 where
        crus: Cop,
        protected: Jester,
        townie: Detective,
        framer: Framer,
        mafioso: Mafioso
    );

    assert!(crus.send_ability_input_player_list_typical(protected));
    assert!(framer.send_ability_input_player_list(townie, 0));
    assert!(framer.send_ability_input_player_list(protected, 1));

    game.next_phase();

    assert!(crus.alive());
    assert!(protected.alive());
    assert!(framer.alive());
    assert!(mafioso.alive());
    assert!(townie.alive());
}

#[test]
fn veteran_basic(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        townie: Lookout,
        _godfather: Godfather,
        framer: Framer,
        tracker: Tracker
    );

    assert!(vet.send_ability_input_boolean_typical(true));
    assert!(framer.send_ability_input_player_list_typical(vet));
    assert!(townie.send_ability_input_player_list_typical(vet));
    assert!(tracker.send_ability_input_player_list_typical(vet));

    game.next_phase();

    assert!(vet.alive());
    assert!(!framer.alive());
    assert!(!townie.alive());

    assert!(
        townie.get_messages().contains(&ChatMessageVariant::LookoutResult { players: vec![framer.index(), tracker.index()] }) ||
        townie.get_messages().contains(&ChatMessageVariant::LookoutResult { players: vec![tracker.index(), framer.index()] })
    );

    // assert_contains!(
    //     townie.get_messages(),
    //     ChatMessageVariant::LookoutResult { players: vec![framer.index(), tracker.index()] }
    // );
    assert_contains!(
        tracker.get_messages(),
        ChatMessageVariant::TrackerResult { players: vec![] }
    );
}

#[test]
fn veteran_does_not_kill_framed_player(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        townie: Detective,
        framer: Framer,
        mafioso: Mafioso
    );

    assert!(vet.send_ability_input_boolean_typical(true));
    assert!(framer.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list(vet, 1));

    game.next_phase();

    assert!(vet.alive());
    assert!(framer.alive());
    assert!(mafioso.alive());
    assert!(townie.alive());
}

#[test]
fn rabble_rouser_turns_into_jester(){
    kit::scenario!(game in Night 2 where
        target: Detective,
        mafioso: Mafioso,
        exe: Revolutionary
    );

    assert!(mafioso.send_ability_input_player_list_typical(target));

    game.skip_to(Nomination, 3);

    assert!(!target.alive());
    assert!(exe.alive());
    assert!(mafioso.alive());
    let RoleState::Jester(_) = exe.role_state() else {panic!()};
}
#[test]
fn rabble_rouser_instantly_turns_into_jester(){
    kit::scenario!(_game where
        exe: Revolutionary
    );
    let RoleState::Jester(_) = exe.role_state() else {panic!()};
}

#[test]
fn can_type_in_jail() {
    kit::scenario!(game in Dusk 1 where
        jailor: Jailor,
        detective: Detective
    );

    jailor.day_target(detective);
    game.next_phase();

    detective.send_message("Hello!");
    
    assert_contains!(jailor.get_messages(), 
        ChatMessageVariant::Normal { 
            message_sender: MessageSender::Player { player: detective.index() }, 
            text: "Hello!".to_string(),
            block: false
        }
    );
    
    assert_contains!(detective.get_messages(), 
        ChatMessageVariant::Normal { 
            message_sender: MessageSender::Player { player: detective.index() }, 
            text: "Hello!".to_string(),
            block: false
        }
    );
}

#[test]
fn mafioso_cant_kill_mafia() {
    kit::scenario!(game in Night 1 where
        mafioso: Mafioso,
        mortician: Mortician
    );

    mafioso.set_night_selection_single(mortician);

    game.next_phase();

    assert!(mortician.alive());
}

#[test]
fn marksman_basic() {
    kit::scenario!(game in Night 2 where
        mk: Marksman,
        dt: Detective,
        gf: Godfather
    );

    assert!(dt.send_ability_input_player_list_typical(gf));
    mk.send_ability_input(AbilityInput::new(
        ControllerID::role(mk.player_ref(), Role::Marksman, 0),
        AbilitySelection::new_player_list(vec!(dt.player_ref()))
    ));
    mk.send_ability_input(AbilityInput::new(
        ControllerID::role(mk.player_ref(), Role::Marksman, 1),
        AbilitySelection::new_player_list(vec!(gf.player_ref()))
    ));

    game.next_phase();

    assert!(!dt.alive());
    assert!(gf.alive());
    assert!(mk.alive());
}

#[test]
fn transporter_cant_transport_dead() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
        _vet: Veteran,
        _necro: Necromancer,
        _seer: Philosopher,
        townie: Detective,
        thomas: Jailor,
        trans: Transporter
    );

    mafioso.send_ability_input_player_list_typical(thomas);

    game.next_phase();

    assert!(!thomas.alive());

    game.skip_to(Night, 3);

    trans.send_ability_input_two_player_typical(townie, thomas);

    game.next_phase();

    assert_not_contains!(thomas.get_messages(), ChatMessageVariant::Transported);
    assert_not_contains!(townie.get_messages(), ChatMessageVariant::Transported);
}

#[test]
fn double_transport() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
 
        townie_a: Detective,
        townie_b: Jailor,

        trans_a: Transporter,
        trans_b: Transporter
    );
    
    assert!(mafioso.send_ability_input_player_list_typical(townie_a));

    assert!(trans_a.send_ability_input_two_player_typical(townie_a, townie_b));
    assert!(trans_b.send_ability_input_two_player_typical(townie_b, townie_a));

    game.next_phase();
    assert!(!townie_a.alive());
    assert!(townie_b.alive());
}


#[test]
fn double_transport_single_player() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
 
        townie_a: Detective,
        townie_b: Jailor,
        townie_c: Vigilante,

        trans_a: Transporter,
        trans_b: Transporter
    );
    
    assert!(mafioso.send_ability_input_player_list_typical(townie_a));

    assert!(trans_a.send_ability_input_two_player_typical(townie_a, townie_b));
    assert!(trans_b.send_ability_input_two_player_typical(townie_a, townie_c));


    game.next_phase();
    assert!(townie_a.alive());
    assert!(!townie_b.alive());
    assert!(townie_c.alive());
}

#[test]
fn double_transport_three_players() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
 
        townie_a: Detective,
        townie_b: Jailor,
        townie_c: Vigilante,

        trans_a: Transporter,
        trans_b: Transporter,
        trans_c: Transporter
    );
    
    assert!(mafioso.send_ability_input_player_list_typical(townie_a));

    assert!(trans_a.send_ability_input_two_player_typical(townie_a, townie_b));
    assert!(trans_b.send_ability_input_two_player_typical(townie_a, townie_c));
    assert!(trans_c.send_ability_input_two_player_typical(townie_b, townie_c));


    game.next_phase();
    assert!(townie_a.alive());
    assert!(townie_b.alive());
    assert!(!townie_c.alive());
}


#[test]
fn grave_contains_multiple_killers() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
        vigilante: Vigilante,
        townie: Detective
    );

    assert!(mafioso.send_ability_input_player_list_typical(townie));
    assert!(vigilante.send_ability_input_player_list_typical(townie));
    game.next_phase();
    assert_eq!(
        *game.graves.first().unwrap(),
        Grave{ 
            player: townie.player_ref(),
            died_phase: GravePhase::Night,
            day_number: 2,
            information: GraveInformation::Normal{
                role: Role::Detective,
                death_cause: GraveDeathCause::Killers(vec![GraveKiller::RoleSet(RoleSet::Mafia), GraveKiller::Role(Role::Vigilante)]),
                will: "".to_string(),
                death_notes: vec![],
            }
        }
    )
}

#[test]
fn grave_contains_multiple_killers_roles() {
    kit::scenario!(game in Night 2 where
        townie_b: Doctor,
        _townie_a: Doctor,
        mafioso: Mafioso,
        vigilante: Vigilante,
        doom: Doomsayer
    );

    assert!(mafioso.send_ability_input_player_list_typical(townie_b));
    assert!(vigilante.send_ability_input_player_list_typical(townie_b));
    doom.set_role_state(RoleState::Doomsayer(
        Doomsayer { guesses: [
            (PlayerReference::new(&game, 0).expect("that player doesnt exist"), DoomsayerGuess::Doctor),
            (PlayerReference::new(&game, 1).expect("that player doesnt exist"), DoomsayerGuess::Doctor),
            (PlayerReference::new(&game, 2).expect("that player doesnt exist"), DoomsayerGuess::NonTown)
        ],
        won: doom.get_won_game()
    }));


    game.next_phase();
    assert_eq!(
        *game.graves.first().unwrap(), 
        Grave {
            player: townie_b.player_ref(),
            died_phase: GravePhase::Night,
            day_number: 2,
            information: GraveInformation::Normal{
                role: Role::Doctor,
                death_cause: GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Doomsayer), GraveKiller::RoleSet(RoleSet::Mafia), GraveKiller::Role(Role::Vigilante)]),
                will: "".to_string(),
                death_notes: vec![],
            }
        }
    );
}

#[test]
fn drunk_suspicious_aura() {
    kit::scenario!(game in Night 1 where
        drunk: Drunk,
        detective: Detective,
        _mafioso: Mafioso
    );

    assert!(detective.send_ability_input_player_list_typical(drunk));

    game.next_phase();

    assert_contains!(
        detective.get_messages(),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );
}

#[test]
fn drunk_framer() {
    kit::scenario!(game in Night 2 where
        drunk: Drunk,
        lookout: Lookout,
        lookout2: Lookout,
        mafioso: Mafioso,
        townie: Doctor,
        framer: Framer
    );

    assert!(mafioso.send_ability_input_player_list_typical(townie));
    assert!(lookout.send_ability_input_player_list_typical(townie));
    assert!(lookout2.send_ability_input_player_list_typical(mafioso));
    framer.send_ability_input_player_list_typical(drunk);
    framer.send_ability_input_player_list(mafioso, 1);

    game.next_phase();

    let messages2 = lookout2.get_messages();
    if !(
        messages2.contains(&ChatMessageVariant::LookoutResult { players: vec![drunk.index()] })
    ){
        panic!("{:?}", messages2);
    }

    let messages = lookout.get_messages();
    if !(
        messages.contains(&ChatMessageVariant::LookoutResult { players: vec![mafioso.index()] })
    ){
        panic!("{:?}", messages);
    }
}

#[test]
fn drunk_role_change() {
    kit::scenario!(game in Night 1 where
        drunk: Drunk,
        lo: Lookout,
        apostle: Apostle,
        mafioso: Mafioso
    );

    assert!(apostle.send_ability_input_player_list_typical(drunk));

    game.skip_to(Night, 2);

    assert!(mafioso.send_ability_input_player_list_typical(apostle));
    assert!(lo.send_ability_input_player_list_typical(apostle));

    game.next_phase();

    let messages = lo.get_messages();
    assert!(
        !messages.contains(&ChatMessageVariant::LookoutResult { players: vec![mafioso.index(), drunk.index()] }) &&
        !messages.contains(&ChatMessageVariant::LookoutResult { players: vec![drunk.index(), mafioso.index()] })
    );
    assert!(
        messages.contains(&ChatMessageVariant::LookoutResult { players: vec![mafioso.index()] })
    );
}

#[test]
fn vigilante_cant_select_night_one() {
    kit::scenario!(game in Night 1 where
        townie_b: Doctor,
        _godfather: Godfather,
        vigilante_suicide: Vigilante

    );
    assert!(!vigilante_suicide.set_night_selection_single(townie_b));
}

#[test]
fn godfather_backup_kills_esc() {
    kit::scenario!(game in Night 2 where
        godfather: Godfather,
        hypnotist: Hypnotist,
        det: Detective,
        esc: Escort
    );

    godfather.send_ability_input(AbilityInput::new(
        ControllerID::SyndicateChooseBackup,
        AbilitySelection::new_player_list(vec![hypnotist.player_ref()])
    ));
    hypnotist.send_ability_input(AbilityInput::new(
        ControllerID::SyndicateBackupAttack,
        AbilitySelection::new_player_list(vec![det.player_ref()])
    ));

    assert!(esc.send_ability_input_player_list_typical(godfather));

    game.next_phase();
    assert!(!det.alive());
    assert!(godfather.alive());
    assert!(hypnotist.alive());
    assert!(esc.alive());
}

#[test]
fn snoop_basic() {
    kit::scenario!(game in Night 1 where
        gf: Godfather,
        det: Detective,
        snoop: Snoop
    );

    assert!(snoop.send_ability_input_player_list_typical(det));
    assert!(det.send_ability_input_player_list_typical(snoop));
    game.next_phase();
    assert_contains!(
        snoop.get_messages(),
        ChatMessageVariant::SnoopResult { townie: false }
    );

    game.skip_to(Night, 2);

    assert!(snoop.send_ability_input_player_list_typical(det));
    game.next_phase();
    assert_contains!(
        snoop.get_messages(),
        ChatMessageVariant::SnoopResult { townie: true }
    );

    game.skip_to(Night, 3);

    assert!(snoop.send_ability_input_player_list_typical(gf));
    game.next_phase();
    assert_contains!(
        snoop.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 3 }
        ),
        ChatMessageVariant::SnoopResult { townie: false }
    );
}

#[test]
fn godfather_backup_kills_jail() {
    kit::scenario!(game in Dusk 2 where
        godfather: Godfather,
        hypnotist: Hypnotist,
        det: Detective,
        jail: Jailor
    );

    assert!(jail.day_target(godfather));
    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        AbilitySelection::new_player_list(vec![hypnotist.player_ref()])
    ));

    game.next_phase();
    assert!(hypnotist.send_ability_input_player_list_typical(det));
    hypnotist.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_backup_attack(),
        AbilitySelection::new_player_list(vec![det.player_ref()])
    ));

    game.next_phase();

    assert!(!det.alive());
    assert!(godfather.alive());
    assert!(hypnotist.alive());
    assert!(jail.alive());
}

#[test]
fn godfathers_backup_tag_works() {
    kit::scenario!(game in Night 2 where
        godfather: Godfather,
        blackmailer: Blackmailer,
        hypnotist: Hypnotist,
        _vigi: Vigilante
    );

    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        AbilitySelection::new_player_list(vec![blackmailer.player_ref()])
    ));
    assert!(blackmailer.get_player_tags().get(&blackmailer.player_ref()).expect("blackmailer doesnt have tag").contains(&Tag::GodfatherBackup));
    
    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        AbilitySelection::new_player_list(vec![])
    ));
    assert!(blackmailer.get_player_tags().get(&blackmailer.player_ref()).is_none());

    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        AbilitySelection::new_player_list(vec![blackmailer.player_ref()])
    ));
    assert!(blackmailer.get_player_tags().get(&blackmailer.player_ref()).expect("blackmailer doesnt have tag").contains(&Tag::GodfatherBackup));
    
    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        AbilitySelection::new_player_list(vec![hypnotist.player_ref()])
    ));
    assert!(blackmailer.get_player_tags().get(&hypnotist.player_ref()).expect("hypnotist doesnt have tag").contains(&Tag::GodfatherBackup));
    assert!(blackmailer.get_player_tags().get(&blackmailer.player_ref()).is_none());
}

#[test]
fn gossip_basic_friends() {
    kit::scenario!(game in Night 1 where
        gossip: Gossip,
        t1: Detective,
        t2: Detective,
        _gf: Godfather
    );

    assert!(gossip.send_ability_input_player_list_typical(t1));
    assert!(t1.send_ability_input_player_list_typical(t2));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: false }
    );
}

#[test]
fn gossip_basic_enemies_inverted() {
    kit::scenario!(game in Night 2 where
        gossip: Gossip,
        t1: Detective,
        _t2: Detective,
        py: Pyrolisk
    );

    assert!(gossip.send_ability_input_player_list_typical(py));
    assert!(py.send_ability_input_player_list_typical(t1));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: false }
    );
}

#[test]
fn gossip_basic_enemies() {
    kit::scenario!(game in Night 2 where
        gossip: Gossip,
        t1: Detective,
        _t2: Detective,
        py: Pyrolisk
    );

    assert!(gossip.send_ability_input_player_list_typical(t1));
    assert!(t1.send_ability_input_player_list_typical(py));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: true }
    );
}

#[test]
fn gossip_framer() {
    kit::scenario!(game in Night 1 where 
        gossip: Gossip,
        framer: Framer,
        t2: Detective,
        townie: Detective,
        _gf: Godfather    
    );

    assert!(gossip.send_ability_input_player_list_typical(townie));
    assert!(townie.send_ability_input_player_list_typical(framer));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: true }
    );

    game.skip_to(Night, 2);

    assert!(gossip.send_ability_input_player_list_typical(townie));
    assert!(townie.send_ability_input_player_list_typical(framer));
    assert!(framer.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list(gossip, 1));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: true }
    );

    game.skip_to(Night, 3);

    assert!(gossip.send_ability_input_player_list_typical(t2));
    assert!(t2.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list(gossip, 1));

    game.next_phase();

    assert_contains!(
        gossip.get_messages_after_night(3),
        ChatMessageVariant::GossipResult { enemies: true }
    );

    game.skip_to(Night, 4);

    assert!(gossip.send_ability_input_player_list_typical(t2));
    assert!(t2.send_ability_input_player_list_typical(townie));

    game.next_phase();

    assert_contains!(
        gossip.get_messages_after_night(4),
        ChatMessageVariant::GossipResult { enemies: false }
    );
}

#[test]
fn vigilante_one_bullet_with_four_players() {
    kit::scenario!(game in Night 2 where
        vigi: Vigilante,
        t1: Detective,
        t2: Detective,
        gf: Godfather
    );

    assert!(vigi.send_ability_input_player_list_typical(gf));
    game.skip_to(Night, 3);
    assert!(vigi.send_ability_input_player_list_typical(t1));

    game.next_phase();

    assert!(t1.alive());
    assert!(t2.alive());
    assert!(vigi.alive());
    assert!(gf.alive());
}

#[test]
fn seer_cant_see_godfather() {
    kit::scenario!(game in Night 1 where
        philosopher: Philosopher,
        godfather: Godfather,
        mafioso: Mafioso,
        townie: Detective
    );

    assert!(philosopher.send_ability_input_two_player_typical(godfather, mafioso));
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 1 }
        ),
        ChatMessageVariant::SeerResult { enemies: false }
    );
    game.skip_to(Night, 2);

    assert!(philosopher.send_ability_input_two_player_typical(godfather, townie));
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::SeerResult { enemies: false }
    );
}

#[test]
fn bouncer_jailor_double_block() {
    kit::scenario!(game in Dusk 1 where
        b: Bouncer,
        jail: Jailor,
        gf: Godfather,
        det: Detective
    );

    jail.day_target(gf);

    game.next_phase();

    det.send_ability_input_player_list_typical(gf);
    b.send_ability_input_player_list_typical(gf);

    game.next_phase();

    assert_contains!(
        det.get_messages_after_last_message(ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 1 }),
        ChatMessageVariant::Wardblocked
    );
}

#[test]
fn bouncer_ojo_block() {
    kit::scenario!(game in Night 2 where
        b: Bouncer,
        ojo: Ojo,
        det1: Detective,
        det2: Detective,
        det3: Detective,
        det4: Detective
    );

    ojo.send_ability_input(AbilityInput::new(
        ControllerID::role(ojo.player_ref(), Role::Ojo, 1),
        AbilitySelection::new_role_option(Some(Role::Detective))
    ));
    b.send_ability_input_player_list_typical(det1);

    game.next_phase();

    assert!(det1.alive());
    assert!(det2.alive());
    assert!(det3.alive());
    assert!(det4.alive());

    assert_contains!(
        ojo.get_messages_after_last_message(ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 1 }),
        ChatMessageVariant::Wardblocked
    );
}

#[test]
fn godfather_backup_sets_off_engineer_trap() {
    kit::scenario!(game in Night 2 where
        backup: Informant,
        eng: Engineer,
        gf: Godfather,
        esc: Escort
    );

    gf.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        AbilitySelection::new_player_list(vec![backup.player_ref()])
    ));
    gf.send_ability_input_player_list_typical(eng);

    backup.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_backup_attack(),
        AbilitySelection::new_player_list(vec![esc.player_ref()])
    ));
    assert!(esc.send_ability_input_player_list_typical(gf));
    assert!(eng.send_ability_input_player_list_typical(esc));

    game.next_phase();

    assert!(gf.alive());
    assert!(esc.alive());
    assert!(eng.alive());
    assert!(!backup.alive());
    
}

#[test]
fn godfather_wardblock_still_kills() {
    kit::scenario!(game in Night 2 where
        rev: Bouncer,
        godfather: Godfather,
        jan: Mortician,
        townie_a: Detective,
        townie_b: Detective
    );

    assert!(rev.send_ability_input_player_list_typical(townie_a));
    assert!(godfather.send_ability_input_player_list_typical(townie_a));
    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        AbilitySelection::new_player_list(vec![jan.player_ref()])
    ));
    jan.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_backup_attack(),
        AbilitySelection::new_player_list(vec![townie_b.player_ref()])
    ));

    game.next_phase();
    assert_not_contains!(
        townie_a.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 1}
        ),
        ChatMessageVariant::Wardblocked
    );
    assert_contains!(
        godfather.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 1}
        ),
        ChatMessageVariant::Wardblocked
    );
    assert_not_contains!(
        jan.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 1}
        ),
        ChatMessageVariant::Wardblocked
    );

    assert!(!townie_b.alive());
    assert!(townie_a.alive());
}

#[test]
fn cult_alternates() {
    kit::scenario!(game in Night 1 where
        apostle: Apostle,
        b: Detective,
        c: Detective,
        d: Detective,
        e: Detective,
        f: Detective,
        g: Detective
    );


    //apostle converts
    assert!(game.cult().next_ability == CultAbility::Convert);
    assert!(apostle.send_ability_input_player_list_typical(b));
    game.next_phase();
    assert!(b.alive());
    assert!(InsiderGroupID::Cult.is_player_in_revealed_group(game.deref(), b.player_ref()));

    //zealot kills, apostle waits
    game.skip_to(Night, 2);
    assert!(game.cult().next_ability == CultAbility::Kill);
    assert!(game.cult().ordered_cultists.len() == 2);
    assert!(b.send_ability_input_player_list_typical(c));
    game.next_phase();
    assert!(!c.alive());
    assert!(d.alive());
    assert!(!InsiderGroupID::Cult.is_player_in_revealed_group(game.deref(), d.player_ref()));

    //zealot waits, apostle converts
    game.skip_to(Night, 3);
    assert!(game.cult().ordered_cultists.len() == 2);
    assert!(apostle.send_ability_input_player_list_typical(d));
    game.next_phase();
    assert!(e.alive());
    assert!(d.alive());
    assert!(InsiderGroupID::Cult.is_player_in_revealed_group(game.deref(), d.player_ref()));

    //zealot kills, apostle waits
    game.skip_to(Night, 4);
    assert!(game.cult().ordered_cultists.len() == 3);
    assert!(d.send_ability_input_player_list_typical(g));
    game.next_phase();
    assert!(f.alive());
    assert!(!g.alive());
}

#[test]
fn puppeteer_marionettes_philosopher(){
    kit::scenario!(game in Night 1 where
        puppeteer: Puppeteer,
        philo: Philosopher,
        townie: Detective,
        townie2: Detective
    );

    puppeteer.set_role_state(RoleState::Puppeteer(Puppeteer{
        marionettes_remaining: 3,
        action: PuppeteerAction::String
    }));

    assert!(puppeteer.set_night_selection_single(townie));
    assert!(philo.send_ability_input_two_player_typical(townie2, townie));

    game.next_phase();
    assert_contains!(
        philo.get_messages_after_night(1),
        ChatMessageVariant::SeerResult{ enemies: true }
    );

    game.skip_to(Night, 2);

    assert!(philo.send_ability_input_two_player_typical(puppeteer, townie));

    game.next_phase();
    assert_contains!(
        philo.get_messages_after_night(2),
        ChatMessageVariant::SeerResult{ enemies: false }
    );
}

#[test]
fn puppeteer_marionettes_die(){
    kit::scenario!(game in Night 1 where
        puppeteer: Puppeteer,
        townie: Detective,
        townie2: Detective,
        townie3: Detective
    );

    puppeteer.set_role_state(RoleState::Puppeteer(Puppeteer{
        marionettes_remaining: 3,
        action: PuppeteerAction::String
    }));

    assert!(puppeteer.set_night_selection_single(townie));

    game.skip_to(Night, 2);

    assert!(puppeteer.set_night_selection_single(townie2));

    game.next_phase();

    assert!(!townie.alive());
    assert!(townie2.alive());
    assert!(townie3.alive());
    assert!(puppeteer.alive());

    game.skip_to(Obituary, 4);

    assert!(!townie.alive());
    assert!(!townie2.alive());
    assert!(townie3.alive());
    assert!(puppeteer.alive());
}

#[test]
fn puppeteer_marionettes_win(){
    kit::scenario!(game in Night 1 where
        puppeteer: Puppeteer,
        townie: Detective,
        townie2: Detective
    );

    puppeteer.set_role_state(RoleState::Puppeteer(Puppeteer{
        marionettes_remaining: 3,
        action: PuppeteerAction::String
    }));

    assert!(puppeteer.set_night_selection_single(townie));

    game.skip_to(Nomination, 2);

    puppeteer.vote_for_player(Some(townie2));
    townie.vote_for_player(Some(townie2));

    game.skip_to(Judgement, 2);

    puppeteer.set_verdict(Verdict::Guilty);

    game.skip_to(Dusk, 2);

    assert!(puppeteer.alive());
    assert!(townie.alive());
    assert!(!townie2.alive());

    assert!(puppeteer.get_won_game());
    assert!(townie.get_won_game());
    assert!(!townie2.get_won_game());
}

#[test]
fn deputy_shoots_marionette(){
    kit::scenario!(game in Night 1 where
        deputy: Deputy,
        puppeteer: Puppeteer,
        townie: Detective
    );

    puppeteer.set_role_state(RoleState::Puppeteer(Puppeteer{
        marionettes_remaining: 3,
        action: PuppeteerAction::String
    }));
    assert!(puppeteer.set_night_selection_single(townie));

    game.skip_to(Discussion, 2);

    deputy.send_ability_input_player_list_typical(townie);

    assert!(puppeteer.alive());
    assert!(!townie.alive());
    assert!(deputy.alive());
}

#[test]
fn vigilante_shoots_marionette(){
    kit::scenario!(game in Night 2 where
        vigilante: Vigilante,
        puppeteer: Puppeteer,
        townie: Detective
    );

    puppeteer.set_role_state(RoleState::Puppeteer(Puppeteer{
        marionettes_remaining: 3,
        action: PuppeteerAction::String
    }));
    assert!(puppeteer.set_night_selection_single(townie));
    assert!(vigilante.send_ability_input_player_list_typical(townie));

    game.next_phase();

    assert!(puppeteer.alive());
    assert!(!townie.alive());
    assert!(vigilante.alive());
}

#[test]
fn recruits_dont_get_converted_to_mk(){
    kit::scenario!(game in Night 2 where
        recruiter: Recruiter,
        mortician: Mortician,
        vigi: Vigilante,
        a: Detective,
        b: Detective,
        c: Detective,
        d: Detective
    );

    assert!(vigi.send_ability_input_player_list_typical(recruiter));

    game.skip_to(Night, 3);

    assert!(!recruiter.alive());
    assert!(mortician.role() == Role::Recruiter);
    assert!(vigi.role() == Role::Vigilante);

    assert!(mortician.set_night_selection_single(a));
    assert!(vigi.send_ability_input_player_list_typical(mortician));

    game.next_phase();

    //tag checks
    assert!(mortician.get_player_tags().get(&a.player_ref()).unwrap().contains(&Tag::PuppeteerMarionette));
    assert!(mortician.get_player_tags().get(&recruiter.player_ref()).is_none());
    assert!(recruiter.get_player_tags().get(&a.player_ref()).unwrap().contains(&Tag::PuppeteerMarionette));
    assert!(recruiter.get_player_tags().get(&mortician.player_ref()).is_none());
    assert!(a.get_player_tags().get(&recruiter.player_ref()).is_none());
    assert!(a.get_player_tags().get(&mortician.player_ref()).is_none());

    assert!(!mortician.alive());
    assert!(a.alive());
    assert!(a.role() == Role::Detective);
    assert!(mortician.role() == Role::Recruiter);
    assert!(vigi.role() == Role::Vigilante);

    //make sure recruiter lost
    assert!(!recruiter.get_won_game());
    assert!(!mortician.get_won_game());
    assert!(!a.get_won_game());
    assert!(b.get_won_game());
    assert!(c.get_won_game());
    assert!(d.get_won_game());
}

#[test]
fn arsonist_ignites_and_aura(){
    kit::scenario!(game in Night 1 where
        arso: Arsonist,
        townie: Detective,
        townie2: Detective,
        gf: Godfather,
        vigi: Vigilante,
        sher: Detective
    );

    assert!(townie.send_ability_input_player_list_typical(arso));
    assert!(arso.send_ability_input_player_list_typical(arso));
    assert!(sher.send_ability_input_player_list_typical(townie));

    game.next_phase();

    assert!(arso.alive());
    assert!(!townie.alive());
    assert!(sher.alive());
    assert!(gf.alive());
    assert!(vigi.alive());

    assert_contains!(sher.get_messages_after_last_message(
        ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 1}
    ), ChatMessageVariant::SheriffResult{ suspicious: true });

    game.skip_to(Night, 2);
    
    assert!(arso.send_ability_input_player_list_typical(townie2));
    assert!(sher.send_ability_input_player_list_typical(townie2));

    game.next_phase();

    assert_contains!(sher.get_messages_after_last_message(
        ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 2}
    ), ChatMessageVariant::SheriffResult{ suspicious: true });

    game.skip_to(Nomination, 3);

    townie2.vote_for_player(Some(arso));
    gf.vote_for_player(Some(arso));
    vigi.vote_for_player(Some(arso));

    game.skip_to(Judgement, 3);

    gf.set_verdict(mafia_server::game::verdict::Verdict::Guilty);

    game.skip_to(Night, 3);

    assert!(sher.send_ability_input_player_list_typical(townie2));

    game.next_phase();
    
    assert_contains!(sher.get_messages_after_last_message(
        ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 3}
    ), ChatMessageVariant::SheriffResult{ suspicious: false });

    
}

#[test]
fn bodyguard_gets_single_target_jailed_message() {
    kit::scenario!(game in Dusk 2 where
        bg: Bodyguard,
        jailor: Jailor,
        _maf: Mafioso,
        townie: Detective
    );

    jailor.day_target(townie);

    game.next_phase();

    bg.send_ability_input_player_list_typical(townie);

    game.next_phase();

    assert_contains!(
        bg.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { 
                phase: PhaseState::Night, day_number: 2
            }
        ),
        ChatMessageVariant::Wardblocked
        /* They should not get a second Wardblocked message */
    );
}

#[test]
fn martyr_suicide_ends_game() {
    kit::scenario!(game in Night 1 where
        martyr: Martyr,
        player1: Mafioso,
        player2: Detective,
        player3: Mortician,
        player4: Detective
    );

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrRevealed { martyr: martyr.index() }
    );

    martyr.set_night_selection_single(martyr);

    game.next_phase();

    assert!(!martyr.alive());
    assert!(martyr.get_won_game());
    assert!(!player1.alive());
    assert!(!player2.alive());
    assert!(!player3.alive());
    assert!(!player4.alive());

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrWon
    );

    assert!(game.game_is_over());
}

#[test]
fn martyr_roleblocked() {
    kit::scenario!(game in Night 1 where
        martyr: Martyr,
        player1: Mafioso,
        player2: Detective,
        hypnotist: Hypnotist,
        player4: Detective
    );

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrRevealed { martyr: martyr.index() }
    );

    martyr.set_night_selection_single(martyr);
    hypnotist.send_ability_input_player_list_typical(martyr);

    game.next_phase();

    assert!(martyr.alive());
    assert!(!martyr.get_won_game());
    assert!(player1.alive());
    assert!(player2.alive());
    assert!(hypnotist.alive());
    assert!(player4.alive());

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrFailed
    );
}

#[test]
fn martyr_healed() {
    kit::scenario!(game in Night 2 where
        martyr: Martyr,
        player1: Mafioso,
        player2: Detective,
        doctor: Doctor,
        player4: Detective
    );

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrRevealed { martyr: martyr.index() }
    );

    martyr.set_night_selection_single(martyr);
    doctor.send_ability_input_player_list_typical(martyr);

    game.next_phase();

    assert!(martyr.alive());
    assert!(!martyr.get_won_game());
    assert!(player1.alive());
    assert!(player2.alive());
    assert!(doctor.alive());
    assert!(player4.alive());

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrFailed
    );
}

#[test]
fn deputy_shoots_townie(){
    kit::scenario!(game in Discussion 2 where
        deputy: Deputy,
        player1: Mafioso,
        player2: Detective
    );

    assert!(deputy.send_ability_input_player_list_typical(player2));
    assert!(!deputy.alive());
    assert!(!player2.alive());
    assert!(player1.alive());
    assert!(player1.get_won_game());
    assert!(game.game_is_over());
}

#[test]
fn ojo_transporter(){
    kit::scenario!(game in Night 2 where
        ojo: Ojo,
        transporter: Transporter,
        player1: Philosopher,
        player2: Detective,
        player3: Philosopher,
        gf: Godfather
    );

    ojo.send_ability_input(AbilityInput::new(
        ControllerID::role(ojo.player_ref(), Role::Ojo, 1),
        AbilitySelection::new_role_option(Some(Role::Philosopher))
    ));

    transporter.send_ability_input_two_player_typical(player1, player2);
    gf.set_night_selection_single(ojo);

    game.next_phase(); 

    assert!(player1.alive());
    assert!(!player2.alive());
    assert!(!player3.alive());
    assert!(gf.alive());
}

#[test]
/// Sometimes this test fails because of the way tests work
/// if the engineer starts as the apostle and is instantly converted to engineer, then the test might fail
fn apostle_converting_trapped_player_day_later(){
    kit::scenario!(game in Night 2 where
        apostle: Apostle,
        _zealot: Zealot,
        trapped: Detective,
        engineer: Engineer
    );


    assert!(engineer.send_ability_input_player_list_typical(trapped));

    game.skip_to(Night, 3);

    assert!(apostle.send_ability_input_player_list_typical(trapped));

    game.next_phase();

    assert_contains!(engineer.get_messages(), ChatMessageVariant::EngineerVisitorsRole { role: Role::Apostle });
    assert!(trapped.role_state().role() != Role::Zealot);
    assert!(trapped.role_state().role() == Role::Detective);
}

#[test]
fn apostle_converting_trapped_player_same_day(){
    kit::scenario!(game in Night 2 where
        apostle: Apostle,
        _zealot: Zealot,
        trapped: Detective,
        engineer: Engineer
    );


    assert!(engineer.send_ability_input_player_list_typical(trapped));
    assert!(apostle.send_ability_input_player_list_typical(trapped));

    game.next_phase();

    assert!(trapped.role_state().role() != Role::Zealot);
    assert!(trapped.role_state().role() == Role::Detective);
}

#[test]
fn godfather_dies_to_veteran(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        gf: Godfather,
        _maf: Mortician
    );

    assert!(gf.send_ability_input_player_list_typical(vet));
    assert!(vet.send_ability_input_boolean_typical(true));

    game.next_phase();

    assert!(!gf.alive());
    assert!(vet.alive());
}

#[test]
fn witch_leaves_by_winning(){
    kit::scenario!(game in Night 2 where
        t: Veteran,
        gf: Godfather,
        arso: Arsonist,
        min: Witch
    );

    assert!(gf.send_ability_input_player_list_typical(t));

    game.next_phase();

    assert!(gf.alive());
    assert!(!min.alive());
    assert!(arso.alive());
    assert!(!t.alive());
}
#[test]
fn scarecrow_leaves_by_winning(){
    kit::scenario!(game in Night 2 where
        t: Veteran,
        gf: Godfather,
        arso: Arsonist,
        min: Scarecrow
    );

    assert!(gf.send_ability_input_player_list_typical(t));

    game.next_phase();

    assert!(gf.alive());
    assert!(!min.alive());
    assert!(arso.alive());
    assert!(!t.alive());
}
#[test]
fn witch_leaves_by_winning_puppeteer(){
    kit::scenario!(game in Night 2 where
        pup: Puppeteer,
        t: Armorsmith,
        t2: Detective,
        gf: Godfather,
        min: Witch
    );

    pup.set_role_state(RoleState::Puppeteer(Puppeteer{
        marionettes_remaining: 3,
        action: PuppeteerAction::String
    }));
    assert!(pup.set_night_selection_single(t));

    game.skip_to(Night, 3);

    assert!(t.send_ability_input_boolean_typical(true));
    assert!(gf.send_ability_input_player_list_typical(t2));

    game.next_phase();

    assert!(gf.alive());
    assert!(!min.alive());
    assert!(!t.alive());
    assert!(!t2.alive());
    assert!(pup.alive());
}

#[test]
fn armorsmith_doesnt_get_wardblocked_when_warded(){
    kit::scenario!(game in Night 2 where
        gf: Godfather,
        armor: Armorsmith,
        bouncer: Bouncer
    );

    assert!(gf.send_ability_input_player_list_typical(bouncer));
    assert!(bouncer.send_ability_input_player_list_typical(armor));
    assert!(armor.send_ability_input_player_list_typical(bouncer));

    game.next_phase();

    assert!(gf.alive());
    assert!(armor.alive());
    assert!(bouncer.alive());

    assert_not_contains!(
        armor.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::Wardblocked
    );
    
    assert_contains!(
        bouncer.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::YouWereProtected
    );
    
    assert_contains!(
        bouncer.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::ArmorsmithArmorBroke
    );
}

#[test]
fn godfather_dies_to_veteran_after_possessed(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        gf: Godfather,
        _maf: Mortician,
        min: Witch
    );

    assert!(min.send_ability_input_two_player_typical(gf, vet));
    assert!(vet.send_ability_input_boolean_typical(true));

    game.next_phase();

    assert!(!gf.alive());
    assert!(vet.alive());
    assert!(min.alive());
}

#[test]
fn fiends_wildcard_defense_upgrade(){
    kit::scenario!(game in Dusk 2 where
        fiend: FiendsWildcard,
        mafia: Godfather
    );
    
    fiend.set_role_state(RoleState::FiendsWildcard(FiendsWildcard{
        role: Role::Puppeteer
    }));

    game.next_phase();

    fiend.set_role_state(RoleState::Puppeteer(Puppeteer{
        marionettes_remaining: 3,
        action: PuppeteerAction::String
    }));

    assert!(fiend.role() == Role::Puppeteer);
    assert!(mafia.send_ability_input_player_list_typical(fiend));
    assert!(fiend.set_night_selection_single(mafia));

    game.next_phase();

    assert!(fiend.alive());
    assert!(mafia.alive());

    assert!(game.game_is_over());
}

#[test]
fn spiraling_player_infects_visitors() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        townie1: Villager,
        townie2: Snoop
    );
    spiral.send_ability_input_player_list_typical(townie1);


    townie2.send_ability_input_player_list_typical(townie1);

    game.skip_to(Obituary, 3);
    assert!(!townie1.alive());
    assert!(townie2.alive());

    game.skip_to(Obituary, 4);
    assert!(!townie2.alive());
}

#[test]
fn spiral_can_select_when_no_spiraling_players() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        townie1: Villager,
        townie2: Snoop,
        townie3: Villager,
        _townie4: Villager
    );

    spiral.send_ability_input_player_list_typical(townie1);
    townie2.send_ability_input_player_list_typical(townie1);
    //kill 1
    //spiral 2

    game.skip_to(Night, 3);
    assert!(spiral.alive());
    assert!(!townie1.alive());
    assert!(townie2.alive());


    //kill 2
    //nobody is spiraling
    spiral.send_ability_input_player_list_typical(townie2);

    game.skip_to(Night, 4);
    assert!(spiral.alive());
    assert!(!townie2.alive());
    assert!(townie3.alive());

    //nobody is spiraling so we can kill 3
    assert!(spiral.send_ability_input_player_list_typical(townie3));
}

#[test]
fn spiral_does_not_kill_protected_player() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        doctor: Doctor
    );
    spiral.send_ability_input_player_list_typical(doctor);

    doctor.send_ability_input_player_list_typical(doctor);

    game.skip_to(Obituary, 3);
    assert!(doctor.alive());
    assert!(spiral.get_player_tags().get(&doctor.player_ref()).is_none())
}

#[test]
fn killed_player_is_not_spiraling() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        townie: Villager
    );
    spiral.send_ability_input_player_list_typical(townie);

    game.skip_to(Obituary, 3);
    assert!(!townie.alive());
    assert!(spiral.get_player_tags().get(&townie.player_ref()).is_none())
}

#[test]
fn geist_transporter_and_warper_test() {
    kit::scenario!(game in Night 2 where
        geist: Geist,
        transporter: Transporter,
        warper: Warper,
        detective: Detective,
        townie1: Villager,
        townie2: Villager,
        mafioso: Mafioso
    );
    detective.send_ability_input_player_list_typical(mafioso);
    geist.send_ability_input_player_list_typical(townie2);
    transporter.send_ability_input_two_player_typical(mafioso, townie2);
    warper.send_ability_input_two_player_typical(townie2, mafioso);

    game.next_phase();

    assert!(townie1.alive());
    assert!(townie2.alive());
    assert!(mafioso.alive());
    assert!(detective.alive());
    assert!(geist.alive());
    assert!(transporter.alive());
    assert!(warper.alive());
    assert_contains!(
        detective.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: false }
    );
}

#[test]
fn geist_win() {
    kit::scenario!(game in Night 2 where
        geist: Geist,
        vig: Vigilante,
        townie1: Villager,
        maf: Mafioso
    );
    game.skip_to(Night, 3);
    geist.send_ability_input_player_list_typical(townie1);
    vig.send_ability_input_player_list_typical(townie1);
    game.next_phase();

    assert!(geist.get_won_game());
    assert!(townie1.alive());
    assert!(!vig.alive());
    assert!(maf.alive());

}
