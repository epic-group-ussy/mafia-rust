
import { createPlayer } from "./gameState";
import { ANCHOR_CONTROLLER, chatMessageToAudio } from "./../menu/Anchor";
import GAME_MANAGER from "./../index";
import GameScreen from "./../menu/game/GameScreen";
import { ToClientPacket } from "./packet";
import { Tag } from "./gameState.d";
import { Role } from "./roleState.d";
import translate from "./lang";
import { computePlayerKeywordData, computePlayerKeywordDataForLobby } from "../components/StyledText";
import { deleteReconnectData, loadSettingsParsed, saveReconnectData } from "./localStorage";
import { WikiArticleLink } from "../components/WikiArticleLink";
import React from "react";
import WikiArticle from "../components/WikiArticle";
import SpectatorGameScreen from "../menu/spectator/SpectatorGameScreen";
import LobbyMenu from "../menu/lobby/LobbyMenu";
import LoadingScreen from "../menu/LoadingScreen";
import AudioController from "../menu/AudioController";
import NightMessagePopup from "../components/NightMessagePopup";
import PlayMenu from "../menu/main/PlayMenu";
import StartMenu from "../menu/main/StartMenu";


function sendDefaultName() {
    const defaultName = loadSettingsParsed().defaultName;
    if(defaultName !== null && defaultName !== undefined && defaultName !== ""){
        GAME_MANAGER.sendSetNamePacket(defaultName)
    }
} 

export default function messageListener(packet: ToClientPacket){

    console.log(JSON.stringify(packet, null, 2));


    switch(packet.type) {
        case "pong":
            if (GAME_MANAGER.state.stateType !== "disconnected") {
                GAME_MANAGER.server.sendPacket({
                    type: "ping"
                });
            }
        break;
        case "rateLimitExceeded":
            ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rateLimitExceeded"), body: "" });
        break;
        case "forcedOutsideLobby":
            GAME_MANAGER.setOutsideLobbyState();
            ANCHOR_CONTROLLER?.setContent(<PlayMenu/>);
        break;
        case "forcedDisconnect":
            GAME_MANAGER.setDisconnectedState();
            ANCHOR_CONTROLLER?.setContent(<StartMenu/>);
        break
        case "lobbyList":
            if(GAME_MANAGER.state.stateType === "outsideLobby"){
                GAME_MANAGER.state.lobbies = new Map();

                for(let [lobbyId, lobbyData] of Object.entries(packet.lobbies))
                    GAME_MANAGER.state.lobbies.set(Number.parseInt(lobbyId), lobbyData);
            }
        break;
        case "acceptJoin":
            if(packet.inGame && packet.spectator){
                GAME_MANAGER.setSpectatorGameState();
                ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
            }else if(packet.inGame && !packet.spectator){
                GAME_MANAGER.setGameState();
                ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
            }else{
                GAME_MANAGER.setLobbyState();
                ANCHOR_CONTROLLER?.setContent(<LobbyMenu/>);
            }
            

            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.roomCode = packet.roomCode;
            }
            if(GAME_MANAGER.state.stateType === "lobby")
                GAME_MANAGER.state.myId = packet.playerId;
        

            saveReconnectData(packet.roomCode, packet.playerId);
            sendDefaultName();
            ANCHOR_CONTROLLER?.clearCoverCard();
        break;
        case "rejectJoin":
            switch(packet.reason) {
                case "roomDoesntExist":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomDoesntExist") });
                    // If the room doesn't exist, don't suggest the user to reconnect to it.
                    deleteReconnectData();
                    ANCHOR_CONTROLLER?.clearCoverCard();
                break;
                case "gameAlreadyStarted":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.gameAlreadyStarted") });
                break;
                case "roomFull":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomFull") });
                break;
                case "serverBusy":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.serverBusy") });
                break;
                case "playerTaken":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerTaken") });
                break;
                case "playerDoesntExist":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerDoesntExist") });
                break;
                default:
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: `${packet.type} message response not implemented: ${packet.reason}` });
                    console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                    console.error(packet);
                break;
            }
            deleteReconnectData();
            
        break;
        case "rejectStart":
            switch(packet.reason) {
                case "gameEndsInstantly":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.gameEndsInstantly") });
                break;
                case "roleListTooSmall":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.roleListTooSmall") });
                break;
                case "roleListCannotCreateRoles":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.roleListCannotCreateRoles") });
                break;
                case "zeroTimeGame":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.zeroTimeGame") });
                break;
                default:
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: "" });
                    console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                    console.error(packet);
                break;
            }
        break;
        case "playersHost":
            if(GAME_MANAGER.state.stateType === "lobby"){
                for(let [playerId, player] of GAME_MANAGER.state.players){
                    if (packet.hosts.includes(playerId)) {
                        player.ready = "host";
                    } else {
                        player.ready = player.ready === "host" ? "ready" : player.ready
                    }
                }
                GAME_MANAGER.state.players = new Map(GAME_MANAGER.state.players.entries());
            }
        break;
        case "playersReady":
            if(GAME_MANAGER.state.stateType === "lobby"){
                for(let [playerId, player] of GAME_MANAGER.state.players){
                    if (packet.ready.includes(playerId)) {
                        player.ready = "ready";
                    } else {
                        player.ready = player.ready === "host" ? "host" : "notReady"
                    }
                }
                GAME_MANAGER.state.players = new Map(GAME_MANAGER.state.players.entries());
            }
        break;
        case "playersLostConnection":
            if(GAME_MANAGER.state.stateType === "lobby"){
                for(let [playerId, player] of GAME_MANAGER.state.players){
                    if(packet.lostConnection.includes(playerId))
                        player.connection = "couldReconnect";
                }
                GAME_MANAGER.state.players = new Map(GAME_MANAGER.state.players.entries());
            }
        break;
        /*
        In Lobby/Game 
        */
        case "yourId":
            if(GAME_MANAGER.state.stateType === "lobby")
                GAME_MANAGER.state.myId = packet.playerId;
        break;
        case "yourPlayerIndex":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.myIndex = packet.playerIndex;

            //TODO jack Im sorry
            AudioController.clearQueue();
        break;
        case "lobbyClients":
            if(GAME_MANAGER.state.stateType === "lobby"){

                let oldMySpectator = GAME_MANAGER.getMySpectator();
                GAME_MANAGER.state.players = new Map();
                for(let [clientId, lobbyClient] of Object.entries(packet.clients)){
                    GAME_MANAGER.state.players.set(Number.parseInt(clientId), lobbyClient);
                }
                let newMySpectator = GAME_MANAGER.getMySpectator();

                
                if (oldMySpectator && !newMySpectator){
                    sendDefaultName();
                }

                // Recompute keyword data, since player names are keywords.
                computePlayerKeywordDataForLobby(
                    Array.from(GAME_MANAGER.state.players.values())
                        .filter(client => client.clientType.type === "player")
                        .map(client => (client.clientType as { type: "player", name: string }).name)
                );
            }
        break;
        case "lobbyName":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.lobbyName = packet.name;
            }
        break;
        case "startGame": {
            const isSpectator = GAME_MANAGER.getMySpectator();
            if(isSpectator){
                GAME_MANAGER.setSpectatorGameState();
                ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
            }else{
                GAME_MANAGER.setGameState();
                ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
            }
        }
        break;
        case "gameInitializationComplete": {
            const isSpectator = GAME_MANAGER.getMySpectator();
            if(isSpectator){
                ANCHOR_CONTROLLER?.setContent(<SpectatorGameScreen/>);
            }else{
                ANCHOR_CONTROLLER?.setContent(<GameScreen/>);
            }
        }
        break;
        case "backToLobby":
            GAME_MANAGER.setLobbyState();
            ANCHOR_CONTROLLER?.setContent(<LobbyMenu/>);
        break;
        case "gamePlayers":
            if(GAME_MANAGER.state.stateType === "game"){
                //only update the playerlist with the new one if there are any differences
                let playersChanged = false;
                if(GAME_MANAGER.state.players.length !== packet.players.length)
                    playersChanged = true;
                else{
                    for(let i = 0; i < packet.players.length; i++){
                        if(GAME_MANAGER.state.players[i].name !== packet.players[i]){
                            playersChanged = true;
                            break;
                        }
                    }
                }
                if(playersChanged){
                    GAME_MANAGER.state.players = [];
                    for(let i = 0; i < packet.players.length; i++){
                        GAME_MANAGER.state.players.push(createPlayer(packet.players[i], i));
                    }
                }

                // Recompute keyword data, since player names are keywords.
                computePlayerKeywordData(GAME_MANAGER.state.players);
            }
        break;
        case "roleList":
            //list of role list entriy
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.roleList = packet.roleList;
        break;
        case "roleOutline":
            //role list entriy
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game") {
                GAME_MANAGER.state.roleList[packet.index] = packet.roleOutline;
                GAME_MANAGER.state.roleList = [...GAME_MANAGER.state.roleList];
            }
        break;
        case "phaseTime":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game") {
                GAME_MANAGER.state.phaseTimes[packet.phase.type] = packet.time;
                GAME_MANAGER.state.phaseTimes = {...GAME_MANAGER.state.phaseTimes};
            }
        break;
        case "phaseTimes":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.phaseTimes = packet.phaseTimeSettings;
        break;
        case "enabledRoles":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.enabledRoles = packet.roles;
        break;
        case "enabledModifiers":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.enabledModifiers = packet.modifiers;
        break;
        case "phase":
            if(GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.phaseState = packet.phase;
                GAME_MANAGER.state.dayNumber = packet.dayNumber;
        
                if(packet.phase.type === "briefing" && GAME_MANAGER.state.clientState.type === "player"){
                    const role = GAME_MANAGER.state.clientState.roleState?.type;
                    if(role !== undefined){
                        ANCHOR_CONTROLLER?.setCoverCard(<WikiArticle article={"role/"+role as WikiArticleLink}/>);
                    }
                }
            }
        break;
        case "phaseTimeLeft":
            if(GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.timeLeftMs = packet.secondsLeft * 1000;
        break;
        case "playerOnTrial":
            if(GAME_MANAGER.state.stateType === "game" && (
                GAME_MANAGER.state.phaseState.type === "testimony" || 
                GAME_MANAGER.state.phaseState.type === "judgement" || 
                GAME_MANAGER.state.phaseState.type === "finalWords"
            ))
                GAME_MANAGER.state.phaseState.playerOnTrial = packet.playerIndex;
        break;
        case "playerAlive":
            if(GAME_MANAGER.state.stateType === "game"){
                for(let i = 0; i < GAME_MANAGER.state.players.length && i < packet.alive.length; i++){
                    GAME_MANAGER.state.players[i].alive = packet.alive[i];
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "playerVotes":
            if(GAME_MANAGER.state.stateType === "game"){
                for(let i = 0; i < GAME_MANAGER.state.players.length; i++){
                    GAME_MANAGER.state.players[i].numVoted = 0;

                    let numVoted = packet.votesForPlayer[i];
                    if(numVoted !== undefined){
                        GAME_MANAGER.state.players[i].numVoted = numVoted;
                    }
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "yourSendChatGroups":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.sendChatGroups = [...packet.sendChatGroups];
            }
        break;
        case "yourButtons":
            if(GAME_MANAGER.state.stateType === "game"){
                for(let i = 0; i < GAME_MANAGER.state.players.length && i < packet.buttons.length; i++){
                    GAME_MANAGER.state.players[i].buttons = packet.buttons[i];
                    GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
                }
            }
        break;
        case "yourRoleLabels":
            if(GAME_MANAGER.state.stateType === "game"){
                for (const player of GAME_MANAGER.state.players) {
                    player.roleLabel = null;
                }
                for (const [key, value] of Object.entries(packet.roleLabels)) { 
                    if(
                        GAME_MANAGER.state.players !== undefined && 
                        GAME_MANAGER.state.players[Number.parseInt(key)] !== undefined
                    )
                        GAME_MANAGER.state.players[Number.parseInt(key)].roleLabel = value as Role;
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "yourPlayerTags":
            if(GAME_MANAGER.state.stateType === "game"){
                for(let i = 0; i < GAME_MANAGER.state.players.length; i++){
                    GAME_MANAGER.state.players[i].playerTags = [];
                }

                for(const [key, value] of Object.entries(packet.playerTags)){
                    if(
                        GAME_MANAGER.state.players !== undefined && 
                        GAME_MANAGER.state.players[Number.parseInt(key)] !== undefined
                    )
                        GAME_MANAGER.state.players[Number.parseInt(key)].playerTags = value as Tag[];
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "yourWill":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.will = packet.will;

                if(GAME_MANAGER.state.clientState.will === ""){
                    GAME_MANAGER.sendSaveWillPacket("ROLE\nNight 1: \nNight 2:");
                }
            }
        break;
        case "yourNotes":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.notes = packet.notes;
                
                if(GAME_MANAGER.state.clientState.notes === ""){
                    GAME_MANAGER.sendSaveNotesPacket(GAME_MANAGER.state.players.map((player) => {
                        return player.toString();
                    }).join(" - \n") + " - \n");
                }
            }
        break;
        case "yourCrossedOutOutlines":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.crossedOutOutlines = packet.crossedOutOutlines;
            break;
        case "yourDeathNote":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.deathNote = packet.deathNote ?? "";
        break;
        case "yourRoleState":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.roleState = packet.roleState;
            }
        break;
        case "yourSelection":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.targets = packet.playerIndices;
        break;
        case "yourVoting":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.voted = packet.playerIndex;
        break;
        case "yourJudgement":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.judgement = packet.verdict;
        break;
        case "yourVoteFastForwardPhase":
            if(GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.fastForward = packet.fastForward;
        break;
        case "yourForfeitVote":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.forfeitVote = packet.forfeit;
        break;
        case "yourPitchforkVote":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.pitchforkVote = packet.player;
        break;
        case "addChatMessages":
            if(GAME_MANAGER.state.stateType === "game" || GAME_MANAGER.state.stateType === "lobby"){
                GAME_MANAGER.state.chatMessages = GAME_MANAGER.state.chatMessages.concat(packet.chatMessages);

                for(let chatMessage of packet.chatMessages){
                    let audioSrc = chatMessageToAudio(chatMessage);
                    if(audioSrc)
                        AudioController.queueFile(audioSrc);
                }
            }
        break;
        case "nightMessages":
            if(GAME_MANAGER.state.stateType === "game" || GAME_MANAGER.state.stateType === "lobby"){

                if(ANCHOR_CONTROLLER?.getCoverCard()===null && packet.chatMessages.length!==0){
                    ANCHOR_CONTROLLER?.setCoverCard(<NightMessagePopup messages={packet.chatMessages}/>)
                }
            }
        break;
        case "addGrave":
            if(GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.graves = [...GAME_MANAGER.state.graves, packet.grave];
        break;
        case "gameOver":
            if(GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.ticking = false;
                switch(packet.reason) {
                    case "reachedMaxDay":
                    case "draw":
                        console.log("Game ended! (naturally)");
                    break;
                    default:
                        // alert("Game ended for an unknown reason!");
                        console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                        console.error(packet);
                    break;
                }
            }
        break;
        default:
            console.error(`incoming message response not implemented: ${(packet as any)?.type}`);
            console.error(packet);
        break;
    }

    GAME_MANAGER.invokeStateListeners(packet.type)
}


