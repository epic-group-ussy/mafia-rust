import { PlayerIndex } from "./gameState.d"
import { Faction, RoleOutline } from "./roleListState.d"
import ROLES from "./../resources/roles.json";
import { Doomsayer } from "../menu/game/gameScreenContent/RoleSpecificMenus/LargeDoomsayerMenu";
import { AuditorResult } from "../menu/game/gameScreenContent/RoleSpecificMenus/LargeAuditorMenu";
import { OjoAction } from "../menu/game/gameScreenContent/RoleSpecificMenus/SmallOjoMenu";

export type RoleState = {
    role: "jailor",
    executionsRemaining: number,
    jailedTargetRef: number | null
} | {
    role: "mayor",
    revealed: boolean
} | {
    role: "transporter"
} | {
    role: "detective"
} | {
    role: "lookout"
} | {
    role: "spy"
} | {
    role: "tracker"
} | {
    role: "philosopher"
} | {
    role: "psychic"
} | {
    role: "auditor",
    chosenOutline: number,
    previouslyGivenResults: [number, AuditorResult][]
} | {
    role: "doctor",
    selfHealsRemaining: number,
} | {
    role: "bodyguard",
    selfShieldsRemaining: number,
} | {
    role: "cop",
} | {
    role: "bouncer"
} | {
    role: "trapper"
} | {
    role: "vigilante",
    state: {type:"notLoaded"} | {type:"willSuicide"} | {type:"loaded",bullets:number} | {type:"suicided"}
} | {
    role: "veteran"
    alertsRemaining: number,
} | {
    role: "deputy"
} | {
    role: "escort"
} | {
    role: "medium",
    seancesRemaining: number,
    seancedTarget: PlayerIndex | null
} | {
    role: "retributionist"
} | {
    role: "journalist",
    public: boolean,
    journal: string,
    interviewedTarget: PlayerIndex | null
} | {
    role: "godfather"
    backup: PlayerIndex | null
} | {
    role: "mafioso"
} | {
    role: "hypnotist"
    roleblock: boolean,
    
    youWereRoleblockedMessage: boolean,
    youSurvivedAttackMessage: boolean,
    youWereProtectedMessage: boolean,
    youWereTransportedMessage: boolean,
    youWerePossessedMessage: boolean,
    yourTargetWasJailedMessage: boolean
} | {
    role: "blackmailer"
} | {
    role: "informant",
} | {
    role: "janitor"
    cleansRemaining: number,
    // cleanedRef
} | {
    role: "forger",
    fakeRole: Role,
    fakeWill: string,
    forgesRemaining: number,
    // forgedRef
} | {
    role: "witch"
} | {
    role: "jester"
} | {
    role: "hater"
} | 
Doomsayer 
| {
    role: "politician"
} | {
    role: "arsonist"
} | {
    role: "werewolf",
    trackedPlayers: PlayerIndex[]
} | {
    role: "ojo"
    chosenAction: OjoAction
} | {
    role: "death",
    souls: number
} | {
    role: "wildCard"
    roleOutline: RoleOutline
} | {
    role: "apostle"
} | {
    role: "disciple"
} | {
    role: "zealot"
} | {
    role: "martyr",
    state: {
        type: "won"
    } | {
        type: "leftTown"
    } | {
        type: "stillPlaying",
        bullets: number
    }
}


export type Role = keyof typeof ROLES;
export function getFactionFromRole(role: Role): Faction {
    return ROLES[role].faction as Faction;
}