import { Grave } from "./grave";
import { ChatMessage } from "./net/chatMessage";

export default interface GameState {
    myName: string | null,
    myIndex: PlayerIndex | null,

    chatMessages : ChatMessage[],
    graves: Grave[],
    players: Player[],
    
    playerOnTrial: PlayerIndex | null,
    phase: Phase | null,
    secondsLeft: number,
    dayNumber: number,

    role: Role | null,

    will: string,
    targets: PlayerIndex[],
    voted: PlayerIndex | null,
    judgement: Verdict | null,
    
    roleList: RoleListEntry[],
    investigatorResults: Role[][],
    phaseTimes: PhaseTimes
}

export type PlayerIndex = number;
export const enum Verdict {
    Innocent = "Innocent",
    Guilty = "Guilty",
    Abstain = "Abstain",
}
export type Phase = 
    | "morning"
    | "discussion"
    | "voting"
    | "testimony"
    | "judgement"
    | "evening"
    | "night"

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
    numVoted: number | null,
    alive: boolean,

    toString(): string
}

export type Role = string;
export type Faction = string;
export type Alignment = string;

export type RoleListEntry = {
    type: "any"
} | {
    type: "faction"
    faction: Faction,
} | {
    type: "alignment"
    faction: Faction,
    alignment: Alignment,
} | {
    type: "exact"
    faction: Faction,
    alignment: Alignment,
    role: Role,
};
