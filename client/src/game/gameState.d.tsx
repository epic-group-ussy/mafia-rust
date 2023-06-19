import { Grave } from "./grave";
import { ChatMessage } from "./chatMessage";
import ROLES from "./../resources/roles.json";
import translate, { styleText } from "./lang";

export default interface GameState {
    inGame: boolean;

    myName: string | null,
    myIndex: PlayerIndex | null,

    chatMessages : ChatMessage[],
    graves: Grave[],
    players: Player[],
    
    phaseState: PhaseState,
    secondsLeft: number,
    dayNumber: number,

    roleState: RoleState,

    will: string,
    notes: string,
    targets: PlayerIndex[],
    voted: PlayerIndex | null,
    judgement: Verdict,
    
    roleList: RoleListEntry[],
    excludedRoles: RoleListEntry[],
    phaseTimes: PhaseTimes
}

export type PlayerIndex = number;
export type Verdict = "innocent"|"guilty"|"abstain";
export type Phase = "morning" | "discussion" | "voting" | "testimony" | "judgement" | "evening" | "night"

export type PhaseState = {
    phase: "morning" | "discussion" | "night"
} | {
    phase: "voting",
    trialsLeft: number
} | {
    phase: "testimony" | "judgement",
    trailsLeft: number,
    playerOnTrial: PlayerIndex
} | {
    phase: "evening",
    playerOnTrial: PlayerIndex | null
}

export interface PhaseTimes {
    "morning": number,
    "discussion": number,
    "voting": number,
    "testimony": number,
    "judgement": number,
    "evening": number,
    "night": number,
}

export interface Player {
    name: string,
    index: number
    buttons: {
        dayTarget: boolean,
        target: boolean,
        vote: boolean,
    },
    numVoted: number,
    alive: boolean,
    roleLabel: Role | null,

    toString(): string
}

export type RoleState = {
    role: "jailor",
    executionsRemaining: number,
    jailedTargetRef: number | null
} | {
    role: "transporter"
} | {
    role: "sheriff"
} | {
    role: "lookout"
} | {
    role: "seer"
} | {
    role: "doctor",
    selfHealsRemaining: number,
} | {
    role: "bodyguard",
    selfShieldsRemaining: number,
} | {
    role: "vigilante",
    bulletsRemaining: number,
    willSuicide: boolean,
} | {
    role:"veteran"
    alertsRemaining: number,
} | {
    role:"escort"
} | {
    role:"medium"
} | {
    role:"retributionist"
} | {
    role:"mafioso"
} | {
    role:"consort"
} | {
    role:"blackmailer"
} | {
    role:"consigliere",
} | {
    role:"janitor"
    cleansRemaining: number,
} | {
    role:"covenLeader"
} | {
    role:"voodooMaster"
}

export type Role = keyof typeof ROLES;
export function getFactionFromRole(role: Role): Faction {
    return getFactionFromFactionAlignment(getFactionAlignmentFromRole(role));
}
export function getFactionAlignmentFromRole(role: Role): FactionAlignment {
    return ROLES[role as keyof typeof ROLES].factionAlignment as FactionAlignment;
}

export const FACTIONS = ["town", "mafia", "neutral", "coven"] as const;
export type Faction = typeof FACTIONS[number]
export function getAllFactionAlignments(faction: Faction): FactionAlignment[] {
    switch(faction){
        case "town": return [
            "townPower", "townKilling", "townProtective", "townInvestigative", "townSupport"
        ];
        case "mafia": return [
            "mafiaKilling", "mafiaDeception", "mafiaSupport"
        ];
        case "neutral": return [
            "neutralKilling", "neutralEvil", "neutralChaos"
        ];
        case "coven": return [
            "covenPower", "covenKilling", "covenUtility", "covenDeception"
        ];
    }
}
export function getRoleListEntryFromFaction(faction: Faction): RoleListEntry {
    return {
        type: "faction",
        faction: faction
    }
}

export const FACTION_ALIGNMENTS = [
    "townPower","townKilling","townProtective","townInvestigative","townSupport",
    "mafiaKilling","mafiaDeception","mafiaSupport",
    "neutralKilling","neutralEvil","neutralChaos",
    "covenPower","covenKilling","covenUtility","covenDeception"
] as const;
export type FactionAlignment = typeof FACTION_ALIGNMENTS[number]

export function getFactionFromFactionAlignment(factionAlignment: FactionAlignment): Faction {
    switch(factionAlignment){
        case "townPower": return "town";
        case "townKilling": return "town";
        case "townProtective": return "town";
        case "townInvestigative": return "town";
        case "townSupport": return "town";

        case "mafiaKilling": return "mafia";
        case "mafiaDeception": return "mafia";
        case "mafiaSupport": return "mafia";

        case "neutralKilling": return "neutral";
        case "neutralEvil": return "neutral";
        case "neutralChaos": return "neutral";

        case "covenPower": return "coven";
        case "covenKilling": return "coven";
        case "covenUtility": return "coven";
        case "covenDeception": return "coven";
    }
}
export function getAlignmentStringFromFactionAlignment(factionAlignment: FactionAlignment): string {
    //make first letter lowercase
    let alignment = factionAlignment.replace(getFactionFromFactionAlignment(factionAlignment).toString(), "");
    return alignment.charAt(0).toLowerCase() + alignment.slice(1);
}
export function getRoleListEntryFromFactionAlignment(factionAlignment: FactionAlignment): RoleListEntry {
    return {
        type: "factionAlignment",
        factionAlignment: factionAlignment
    }
}


export type RoleListEntry={
    type: "any",
} | {
    type: "faction",
    faction: Faction,
} | {
    type: "factionAlignment",
    factionAlignment: FactionAlignment,
} | {
    type: "exact",
    role: Role,
};
export type RoleListEntryType = RoleListEntry["type"];

export function getFactionFromRoleListEntry(roleListEntry: RoleListEntry): Faction | null {
    switch(roleListEntry.type){
        case "any": return null;
        case "faction": return roleListEntry.faction;
        case "factionAlignment": return getFactionFromFactionAlignment(roleListEntry.factionAlignment);
        case "exact": return getFactionFromRole(roleListEntry.role);
    }
}
export function getFactionAlignmentFromRoleListEntry(roleListEntry: RoleListEntry): FactionAlignment | null {
    switch(roleListEntry.type){
        case "any": return null;
        case "faction": return null;
        case "factionAlignment": return roleListEntry.factionAlignment;
        case "exact": return getFactionAlignmentFromRole(roleListEntry.role);
    }
}

export function  renderRoleListEntry(roleListEntry: RoleListEntry): JSX.Element[] | null{
    if(roleListEntry.type === "any"){
        return styleText(translate("any"))
    }
    if(roleListEntry.type === "faction"){
        return styleText(translate("faction."+roleListEntry.faction.toString())+" "+translate("any"))
    }
    if(roleListEntry.type === "factionAlignment"){
        return styleText(translate("faction."+getFactionFromFactionAlignment(roleListEntry.factionAlignment))+" "+translate("alignment."+getAlignmentStringFromFactionAlignment(roleListEntry.factionAlignment)))
    }
    if(roleListEntry.type === "exact"){
        return styleText(translate("role."+roleListEntry.role+".name"))
    }
    return null
}