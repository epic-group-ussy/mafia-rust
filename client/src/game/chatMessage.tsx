import { Phase, PlayerIndex, Role, Verdict } from "./gameState.d"
import { Grave } from "./grave"

export type ChatMessage = {
    type: "normal", 
    messageSender: MessageSender, 
    text: string, 
    chatGroup: ChatGroup
} | {
    type: "whisper", 
    fromPlayerIndex: PlayerIndex, 
    toPlayerIndex: PlayerIndex, 
    text: string
} | {
    type: "broadcastWhisper", 
    whisperer: PlayerIndex, 
    whisperee: PlayerIndex 
} | 
// System
{
    type: "roleAssignment", 
    role: Role
} | {
    type: "playerDied", 
    grave: Grave
} | {
    type: "gameOver"
} | {
    type: "phaseChange", 
    phase: Phase, 
    dayNumber: number
} | 
// Trial
{
    type: "trialInformation", 
    requiredVotes: number, 
    trialsLeft: number
} | {
    type: "voted", 
    voter: PlayerIndex, 
    votee: PlayerIndex | null 
} | {
    type: "playerOnTrial", 
    playerIndex: PlayerIndex
} | {
    type: "judgementVote", 
    voterPlayerIndex: PlayerIndex
} | {
    type: "judgementVerdict", 
    voterPlayerIndex: PlayerIndex, 
    verdict: Verdict
} | {
    type: "trialVerdict", 
    playerOnTrial: PlayerIndex, 
    innocent: number, 
    guilty: number
} | 
// Misc.
{
    type: "targeted", 
    targeter: PlayerIndex, 
    targets: PlayerIndex[]
} | {
    type: "nightInformation", 
    nightInformation: NightInformation 
} | 
// Role-specific
{
    type: "mayorRevealed", 
    playerIndex: PlayerIndex
} | {
    type: "mayorCantWhisper"
} | {
    type: "jailedSomeone",
    playerIndex: PlayerIndex
} | {
    type: "jailedTarget"
    playerIndex: PlayerIndex
} | {
    type: "jailorDecideExecute"
    targets: PlayerIndex[]
} | {
    type: "mediumSeanceYou"
} | {
    type: "jesterWon"
} | {
    type: "executionerWon"
} | {
    type: "deputyShot",
    deputyIndex: PlayerIndex,
    shotIndex: PlayerIndex
} | {
    type: "playerWithNecronomicon",
    playerIndex: PlayerIndex
} | {
    type: "roleData", 
    roleData: Role | {
        0: Role
    }
}

export type MessageSender = {
    type: "player", 
    player: PlayerIndex
} | {
    type: "jailor"
} | {
    type: "medium"
}

export type ChatGroup =
    | "all"
    | "mafia"
    | "dead"
    | "vampire"
    | "coven"

export type NightInformation = {
    type: "roleBlocked", 
    immune : boolean
} | {
    type: "targetSurvivedAttack"
} | {
    type: "youSurvivedAttack"
} | {
    type: "youDied"
} |
/* Role-specific */
{
    type: "targetJailed"
} | {
    type: "sheriffResult", 
    suspicious: boolean
} | {
    type: "lookoutResult", 
    players: PlayerIndex[]
} | {
    type: "seerResult",
    enemies: boolean
} | {
    type: "spyMafiaVisit", 
    players: PlayerIndex[]
} | {
    type: "spyBug", 
    message: ChatMessage
} | {
    type: "veteranAttackedYou"
} | {
    type: "veteranAttackedVisitor"
} | {
    type: "vigilanteSuicide"
} | {
    type: "doctorHealed"
} | {
    type: "doctorHealedYou"
} | {
    type: "bodyguardProtected"
} | {
    type: "bodyguardProtectedYou"
} | {
    type: "transported"
} | {
    type: "godfatherForcedMafioso"
} | {
    type: "godfatherForcedYou"
} | {
    type: "silenced"
} | {
    type: "framerFramedPlayers", 
    players: PlayerIndex[]
} | {
    type: "playerRoleAndWill", 
    role: Role,
    will: string
} | {
    type: "consigliereResult", 
    role: Role,
    visitedBy: PlayerIndex[],
    visited: PlayerIndex[]
} | {
    type: "witchTargetImmune"
} | {
    type: "witchedYou", 
    immune: boolean
} | {
    type: "witchBug", 
    message: ChatMessage
} | {
    type: "arsonistCleanedSelf"
} | {
    type: "arsonistDousedPlayers", 
    players: PlayerIndex[]
}