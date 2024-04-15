import { PhaseType, PlayerIndex, Verdict, PhaseTimes, Tag, LobbyClientID, ChatGroup, PhaseState, LobbyClient } from "./gameState.d"
import { Grave } from "./graveState"
import { ChatMessage } from "../components/ChatMessage"
import { RoleList, RoleOutline } from "./roleListState.d"
import { Role, RoleState } from "./roleState.d"
import { DoomsayerGuess } from "../menu/game/gameScreenContent/RoleSpecificMenus/LargeDoomsayerMenu"
import { OjoAction } from "../menu/game/gameScreenContent/RoleSpecificMenus/SmallOjoMenu"

export type LobbyPreviewData = {
    name: string,
    players: [LobbyClientID, string][]
}

export type ToClientPacket = {
    type: "pong",
} | {
    type: "rateLimitExceeded",
} | {
    type: "lobbyList",
    lobbies: Record<number, LobbyPreviewData>,
} | {
    type: "acceptJoin",
    roomCode: number,
    inGame: boolean,
    playerId: number,
    spectator: boolean
} | {
    type: "rejectJoin",
    reason: string
} | 
// Lobby
{
    type: "yourId",
    playerId: LobbyClientID
} | {
    type: "lobbyClients",
    clients: Record<LobbyClientID, LobbyClient>
} | {
    type: "lobbyName",
    name: string
} | {
    type: "yourPlayerIndex",
    playerIndex: PlayerIndex
} | {
    type: "rejectStart",
    reason: string
} | {
    type: "playersHost",
    hosts: LobbyClientID[],
} | {
    type: "playersLostConnection",
    lostConnection: LobbyClientID[],
} | {
    type: "startGame"
} | {
    type: "gamePlayers",
    players: string[]
} | {
    type: "roleList",
    roleList: RoleList,
} | {
    type: "roleOutline",
    index: number,
    roleOutline: RoleOutline
} | {
    type: "phaseTime",
    phase: PhaseState, 
    time: number
} | {
    type: "phaseTimes",
    phaseTimeSettings: PhaseTimes
} | {
    type: "excludedRoles",
    roles: Role[]
} | 
// Game
{
    type: "phase",
    phase: PhaseState, 
    dayNumber: number, 
} | {
    type: "phaseTimeLeft",
    secondsLeft: number
} |{
    type: "playerOnTrial",
    playerIndex: PlayerIndex
} | {
    type: "playerAlive", 
    alive: [boolean]
} | {
    type: "playerVotes",
    votesForPlayer: any
} | {
    type: "yourSendChatGroups",
    sendChatGroups: ChatGroup[]
} | {
    type: "yourButtons", 
    buttons: [{
        dayTarget: boolean,
        target: boolean,
        vote: boolean,
    }]
} | {
    type: "yourRoleLabels",
    roleLabels: Record<PlayerIndex, Role>
} | {
    type: "yourPlayerTags",
    playerTags: Record<PlayerIndex, Tag[]>
} | {
    type: "yourWill",
    will: string
} | {
    type: "yourNotes",
    notes: string
} | {
    type: "yourCrossedOutOutlines",
    crossedOutOutlines: number[]
} | {
    type: "yourDeathNote", 
    deathNote: string | null
} | {
    type: "yourRoleState",
    roleState: RoleState
} | {
    type: "yourTarget",
    playerIndices: [PlayerIndex]
} | {
    type: "yourVoting",
    playerIndex: PlayerIndex | null
} | {
    type: "yourJudgement",
    verdict: Verdict
} | {
    type: "yourVoteFastForwardPhase",
    fastForward: boolean
} | {
    type: "addChatMessages",
    chatMessages: ChatMessage[]
} | {
    type: "addGrave",
    grave: Grave
} | {
    type: "gameOver",
    reason: string
}

export type ToServerPacket = {
    type: "ping",
} | {
    type: "lobbyListRequest",
} | {
    type: "reJoin",
    roomCode: number,
    playerId: number,
} | {
    type: "join", 
    roomCode: number
} | {
    type: "host",
} | {
    type: "kick",
    playerId: number
}
// Lobby
| {
    type: "setSpectator",
    spectator: boolean
} | {
    type: "setName", 
    name: string
} | {
    type: "setLobbyName", 
    name: string
} | {
    type: "startGame",
} | {
    type: "setRoleList", 
    roleList: RoleList,
} | {
    type: "setRoleOutline", 
    index: number,
    roleOutline: RoleOutline
} | {
    type: "simplifyRoleList"
} | {
    type: "setPhaseTime", 
    phase: PhaseType, 
    time: number
} | {
    type: "setPhaseTimes", 
    phaseTimeSettings: PhaseTimes
} | {
    type: "setExcludedRoles", 
    roles: Role[], 
} | 
// Game
{
    type: "vote", 
    playerIndex: PlayerIndex | null
} | {
    type: "judgement", 
    verdict: Verdict
} | {
    type: "target", 
    playerIndexList: PlayerIndex[]
} | {
    type: "dayTarget", 
    playerIndex:  PlayerIndex
} | {
    type: "sendMessage", 
    text: string
} | {
    type: "sendWhisper", 
    playerIndex: PlayerIndex, 
    text: string
} | {
    type: "saveWill", 
    will: string
} | {
    type: "saveNotes", 
    notes: string
} | {
    type: "saveCrossedOutOutlines",
    crossedOutOutlines: number[]
} | {
    type: "saveDeathNote", 
    deathNote: string | null
} | {
    type: "leave",
} | {
    type: "setForgerWill",
    role: Role,
    will: string
} | {
    type: "setDoomsayerGuess",
    guesses: [
        [number, DoomsayerGuess],
        [number, DoomsayerGuess],
        [number, DoomsayerGuess]
    ]
} | {
    type: "setWildCardRole",
    role: Role
} | {
    type: "setJournalistJournal",
    journal: string
} | {
    type: "setJournalistJournalPublic",
    public: boolean
} | {
    type: "setConsortOptions",
    roleblock: boolean,
    
    youWereRoleblockedMessage: boolean,
    youSurvivedAttackMessage: boolean,
    youWereProtectedMessage: boolean,
    youWereTransportedMessage: boolean,
    youWerePossessedMessage: boolean,
    yourTargetWasJailedMessage: boolean
} | {
    type: "setForgerWill",
    role: Role | null,
    will: string
} | {
    type: "setAuditorChosenOutline",
    index: number
} | {
    type: "setOjoAction",
    action: OjoAction
} | {
    type: "voteFastForwardPhase",
    fastForward: boolean
}