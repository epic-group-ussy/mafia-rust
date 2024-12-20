import { Grave } from "./graveState";
import { ChatMessage } from "../components/ChatMessage";
import { Role, RoleState } from "./roleState.d";
import { RoleList } from "./roleListState.d";
import { LobbyPreviewData } from "./packet";
import { ChatFilter } from "../menu/game/gameScreenContent/ChatMenu";
import { ControllerID, SavedController } from "./abilityInput";
import { ListMapData } from "../ListMap";
import translate from "./lang";


export type State = Disconnected | OutsideLobbyState | LobbyState | GameState;

export type Disconnected = {
    stateType: "disconnected"
}

export type OutsideLobbyState = {
    stateType: "outsideLobby",

    selectedRoomCode: string | null,
    lobbies: Map<number, LobbyPreviewData>,
}


//Change this to use PlayerID for player map and playerID for who I AM instead of myName and host
export type LobbyState = {
    stateType: "lobby"
    roomCode: number,
    lobbyName: string,

    myId: number | null,

    roleList: RoleList,
    phaseTimes: PhaseTimes,
    enabledRoles: Role[],
    enabledModifiers: ModifierType[],

    players: Map<LobbyClientID, LobbyClient>,
    chatMessages: ChatMessage[],
}
export type LobbyClient = {
    ready: "host" | "ready" | "notReady",
    connection: "connected" | "disconnected" | "couldReconnect",
    clientType: LobbyClientType
}
export type LobbyClientType = {
    type: "spectator"
} | PlayerClientType;
export type PlayerClientType = {
    type: "player",
    name: string,
}

type GameState = {
    stateType: "game"
    roomCode: number,
    lobbyName: string,

    chatMessages : ChatMessage[],
    graves: Grave[],
    players: Player[],
    
    phaseState: PhaseState,
    timeLeftMs: number,
    dayNumber: number,

    fastForward: boolean,
    
    roleList: RoleList,
    enabledRoles: Role[],
    phaseTimes: PhaseTimes,
    enabledModifiers: ModifierType[],

    ticking: boolean,

    clientState: PlayerGameState | {type: "spectator"},
    host: boolean,

}
export default GameState;

export type PlayerGameState = {
    type: "player",

    myIndex: PlayerIndex,
    
    roleState: RoleState,

    will: string,
    notes: string[],
    crossedOutOutlines: number[],
    chatFilter: ChatFilter,
    deathNote: string,
    targets: PlayerIndex[],
    voted: PlayerIndex | null,
    judgement: Verdict,

    savedControllers: ListMapData<ControllerID, SavedController>,

    fellowInsiders: PlayerIndex[],

    sendChatGroups: ChatGroup[],
    insiderGroups: InsiderGroup[],
}

export type PlayerIndex = number;
export type LobbyClientID = number;
export type Verdict = "innocent"|"guilty"|"abstain";
export const PHASES = ["briefing", "obituary", "discussion", "nomination", "testimony", "judgement", "finalWords", "dusk", "night", "recess"] as const;
export type PhaseType = (typeof PHASES)[number];
export type PhaseState = {type: "briefing"} | {type: "recess"} | {type: "dusk"} | {type: "night"} | {type: "obituary"} | {type: "discussion"} | 
{
    type: "nomination",
    trialsLeft: number
} | {
    type: "testimony",
    playerOnTrial: PlayerIndex
    trialsLeft: number
} | {
    type: "judgement",
    playerOnTrial: PlayerIndex
    trialsLeft: number
} | {
    type: "finalWords",
    playerOnTrial: PlayerIndex
}

export type ChatGroup = "all" | "dead" | "mafia" | "cult" | "jail" | "kidnapper" | "interview" | "puppeteer";
export type InsiderGroup = (typeof INSIDER_GROUPS)[number];
export const INSIDER_GROUPS = ["mafia", "cult", "puppeteer"] as const;
export type PhaseTimes = Record<Exclude<PhaseType, "recess">, number>;

export type Tag = 
    "disguise" |
    "syndicateGun" |
    "godfatherBackup" |
    "werewolfTracked" |
    "doused" |
    "revolutionaryTarget" |
    "morticianTagged" |
    "puppeteerMarionette" |
    "loveLinked" |
    "frame" |
    "forfeitVote" |
    "spiraling";

export const MODIFIERS = [
    "obscuredGraves", "randomLoveLinks",
    "deadCanChat", "noAbstaining",
    "noDeathCause",
    "roleSetGraveKillers"
] as const;
export type ModifierType = (typeof MODIFIERS)[number];

export type Player = {
    name: string,
    index: number
    buttons: {
        vote: boolean,
    },
    numVoted: number,
    alive: boolean,
    roleLabel: Role | null,
    playerTags: Tag[]

    toString(): string
}

export type Conclusion = "town" | "mafia" | "cult" | "fiends" | "politician" | "draw";

export type WinCondition = {
    type: "gameConclusionReached"
    winIfAny: Conclusion[]
} | {
    type: "roleStateWon"
}

export function translateWinCondition(winCondition: WinCondition): string {
    if (winCondition.type === "gameConclusionReached") {
        if (winCondition.winIfAny.length === 0) {
            return translate("winCondition.loser")
        } else if (winCondition.winIfAny.length === 1) {
            const winningTeam = winCondition.winIfAny[0];
            switch (winningTeam) {
                case "politician":
                    return translate("role.politician.name")
                case "draw":
                    return translate("winCondition.draw")
                default:
                    return translate(winningTeam)
            }
        } else {
            if (winCondition.winIfAny.length === 4 && 
                (["mafia", "fiends", "cult", "politician"] as const).every(team => winCondition.winIfAny.includes(team))
            ) {
                return translate(`winCondition.evil`)
            }
            return translate(`winCondition.many`)
        }
    } else {
        return translate("winCondition.independent");
    }
}