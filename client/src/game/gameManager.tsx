import Anchor from "./../menu/Anchor";
import StartMenu from "./../menu/main/StartMenu";
import GAME_MANAGER from "./../index";
import messageListener from "./messageListener";
import CONFIG from "./../resources/config.json"
import React from "react";
import { Modifier, Phase, PhaseTimes, PlayerID, Verdict } from "./gameState.d";
import { GameManager, Server, StateListener } from "./gameManager.d";
import { ToClientPacket, ToServerPacket } from "./packet";
import { RoleOutline } from "./roleListState.d";
import translate from "./lang";
import PlayMenu from "../menu/main/PlayMenu";
import { createGameState, createLobbyState } from "./gameState";
import LobbyMenu from "../menu/lobby/LobbyMenu";
import GameScreen from "../menu/game/GameScreen";
import LoadingScreen from "../menu/LoadingScreen";
import { Role } from "./roleState.d";
import DUMMY_NAMES from "../resources/dummyNames.json";
export function createGameManager(): GameManager {

    console.log("Game manager created.");
    
    let gameManager: GameManager = {
        setDisconnectedState() {
            Anchor.stopAudio();
            GAME_MANAGER.server.close();

            GAME_MANAGER.state = {
                stateType: "disconnected"
            };
            Anchor.setContent(<StartMenu/>);
        },
        setLobbyState() {
            GAME_MANAGER.state = createLobbyState();
            Anchor.setContent(<LobbyMenu/>);
            Anchor.playAudioFile("/audio/01. Calm Before The Storm.mp3");
        },
        setGameState() {
            let roomCode = null;
            if (GAME_MANAGER.state.stateType === "lobby") {
                roomCode = GAME_MANAGER.state.roomCode;
            }


            Anchor.stopAudio();
            GAME_MANAGER.state = createGameState();
            Anchor.setContent(GameScreen.createDefault());
            if (roomCode !== null) {
                GAME_MANAGER.state.roomCode = roomCode;
            }
        },
        async setOutsideLobbyState() {
            Anchor.stopAudio();

            Anchor.setContent(<LoadingScreen type="default" />);
            // GAME_MANAGER.server.close();
            if (!GAME_MANAGER.server.ws?.OPEN) {
                await GAME_MANAGER.server.open();
            }

            GAME_MANAGER.state = {
                stateType: "outsideLobby",
                selectedRoomCode: null,
                lobbies: new Map<number, [PlayerID, string][]>()
            };

            Anchor.setContent(<PlayMenu />);
        },

        saveReconnectData(roomCode, playerId) {
            localStorage.setItem(
                "reconnectData",
                JSON.stringify({
                    "roomCode": roomCode,
                    "playerId": playerId,
                    "lastSaveTime": Date.now()
                })
            );
        },
        deleteReconnectData() {
            localStorage.removeItem("reconnectData");
        },
        loadReconnectData() {
            let data = localStorage.getItem("reconnectData");
            // localStorage.removeItem("reconnectData");
            if (data) {
                return JSON.parse(data);
            }
            return null;
        },
        saveSettings(volume) {
            localStorage.setItem("settings", JSON.stringify({
                "volume": volume,
            }));
        },
        loadSettings() {
            let data = localStorage.getItem("settings");
            if (data) {
                return JSON.parse(data);
            }
            return null;
        },

        state: {
            stateType: "disconnected"
        },

        getMyName() {
            if (gameManager.state.stateType === "lobby")
                return gameManager.state.players.get(gameManager.state.myId!)?.name;
            if (gameManager.state.stateType === "game")
                return gameManager.state.players[gameManager.state.myIndex!]?.name;
            return undefined;
        },
        getMyHost() {
            if (gameManager.state.stateType === "lobby")
                return gameManager.state.players.get(gameManager.state.myId!)?.host;
            if (gameManager.state.stateType === "game")
                return gameManager.state.players[gameManager.state.myIndex!]?.host;
            return undefined;
        },
        getPlayerNames(): string[] {
            switch (GAME_MANAGER.state.stateType) {
                case "game":
                    return GAME_MANAGER.state.players.map((player) => player.toString());
                default:
                    return DUMMY_NAMES;
            }
        },


        server: createServer(),

        listeners: [],

        addStateListener(listener) {
            gameManager.listeners.push(listener);
        },
        removeStateListener(listener) {
            let index = gameManager.listeners.indexOf(listener);
            if (index !== -1)
                gameManager.listeners.splice(index, 1);
        },
        invokeStateListeners(type) {
            for (let i = 0; i < gameManager.listeners.length; i++) {
                if (typeof (gameManager.listeners[i]) === "function") {
                    gameManager.listeners[i](type);
                }
            }
        },


        leaveGame() {
            if (this.state.stateType !== "disconnected") {
                this.server.sendPacket({ type: "leave" });
            }
            this.deleteReconnectData();
            this.setOutsideLobbyState();
            // Set URL to main menu and refresh
            // window.history.replaceState({}, document.title, window.location.pathname);
            // window.location.reload();
        },

        sendLobbyListRequest() {
            this.server.sendPacket({ type: "lobbyListRequest" });
        },
        sendHostPacket() {
            this.server.sendPacket({ type: "host" });
        },
        sendRejoinPacket(roomCode: string, playerId: number) {
            let completePromise: (success: boolean) => void;
            let promise = new Promise<boolean>((resolver) => {
                completePromise = resolver;
            });
            let onJoined: StateListener = (type) => {
                if (type === "acceptJoin") {
                    completePromise(true);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "rejectJoin") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                }
            };
            GAME_MANAGER.addStateListener(onJoined);

            this.server.sendPacket({
                type: "reJoin",
                roomCode: parseInt(roomCode, 18),
                playerId: playerId
            });


            return promise;
        },
        sendJoinPacket(roomCode: string) {
            let completePromise: (success: boolean) => void;
            let promise = new Promise<boolean>((resolver) => {
                completePromise = resolver;
            });
            let onJoined: StateListener = (type) => {
                if (type === "acceptJoin") {
                    completePromise(true);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "rejectJoin") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                }
            };
            GAME_MANAGER.addStateListener(onJoined);

            let actualCode: number = parseInt(roomCode, 18);

            this.server.sendPacket({
                type: "join",
                roomCode: isNaN(actualCode) ? 0 : actualCode
            });

            return promise;
        },
        sendKickPlayerPacket(playerId: number) {
            this.server.sendPacket({
                type: "kick",
                playerId: playerId
            });
        },

        sendSetNamePacket(name) {
            this.server.sendPacket({
                type: "setName",
                name: name
            });
        },
        sendStartGamePacket() {
            this.server.sendPacket({
                type: "startGame"
            });
        },
        sendSetPhaseTimePacket(phase: Phase, time: number) {
            if (isValidPhaseTime(time)) {
                this.server.sendPacket({
                    type: "setPhaseTime",
                    phase: phase,
                    time: time
                });
            }
        },
        sendSetModifiersPacket(modifier: Modifier[]) {
            this.server.sendPacket({
                type: "setModifiers",
                modifiers: modifier
            });
        },
        sendSetPhaseTimesPacket(phaseTimeSettings: PhaseTimes) {
            this.server.sendPacket({
                type: "setPhaseTimes",
                phaseTimeSettings
            });
        },
        sendSetRoleListPacket(roleListEntries: RoleOutline[]) {
            this.server.sendPacket({
                type: "setRoleList",
                roleList: roleListEntries
            });
        },
        sendSetRoleOutlinePacket(index: number, roleOutline: RoleOutline) {
            this.server.sendPacket({
                type: "setRoleOutline",
                index,
                roleOutline
            });
        },
        sendSimplifyRoleListPacket() {
            this.server.sendPacket({
                type: "simplifyRoleList"
            });
        },

        sendJudgementPacket(judgement: Verdict) {
            this.server.sendPacket({
                type: "judgement",
                verdict: judgement
            });
        },
        sendVotePacket(voteeIndex) {
            this.server.sendPacket({
                type: "vote",
                playerIndex: voteeIndex
            });
        },
        sendTargetPacket(targetIndexList) {
            this.server.sendPacket({
                type: "target",
                playerIndexList: targetIndexList
            });
        },
        sendDayTargetPacket(targetIndex) {
            this.server.sendPacket({
                type: "dayTarget",
                playerIndex: targetIndex
            });
        },

        sendSaveWillPacket(will) {
            this.server.sendPacket({
                type: "saveWill",
                will: will
            });
        },
        sendSaveNotesPacket(notes) {
            this.server.sendPacket({
                type: "saveNotes",
                notes: notes
            });
        },
        sendSaveCrossedOutOutlinesPacket(crossedOutOutlines) {
            this.server.sendPacket({
                type: "saveCrossedOutOutlines",
                crossedOutOutlines: crossedOutOutlines
            });
        },
        sendSaveDeathNotePacket(notes) {
            this.server.sendPacket({
                type: "saveDeathNote",
                deathNote: notes.trim().length === 0 ? null : notes
            });
        },
        sendSendMessagePacket(text) {
            this.server.sendPacket({
                type: "sendMessage",
                text: text
            });
        },
        sendSendWhisperPacket(playerIndex, text) {
            this.server.sendPacket({
                type: "sendWhisper",
                playerIndex: playerIndex,
                text: text
            });
        },
        sendExcludedRolesPacket(roles) {
            this.server.sendPacket({
                type: "setExcludedRoles",
                roles: roles
            });
        },

        sendSetDoomsayerGuess(guesses) {
            this.server.sendPacket({
                type: "setDoomsayerGuess",
                guesses: guesses
            });
        },
        sendSetAmnesiacRoleOutline(roleOutline) {
            this.server.sendPacket({
                type: "setAmnesiacRoleOutline",
                roleOutline: roleOutline
            });
        },
        sendSetJournalistJournal(journal: string) {
            this.server.sendPacket({
                type: "setJournalistJournal",
                journal: journal,
            });
        },
        sendSetJournalistJournalPublic(isPublic: boolean) {
            this.server.sendPacket({
                type: "setJournalistJournalPublic",
                public: isPublic,
            });
        },
        sendSetConsortOptions(
            roleblock: boolean,
            youWereRoleblockedMessage: boolean,
            youSurvivedAttackMessage: boolean,
            youWereProtectedMessage: boolean,
            youWereTransportedMessage: boolean,
            youWerePossessedMessage: boolean,
            yourTargetWasJailedMessage: boolean
        ): void {
            this.server.sendPacket({
                type: "setConsortOptions",
                roleblock: roleblock,

                youWereRoleblockedMessage: youWereRoleblockedMessage === undefined || youWereRoleblockedMessage === null ? false : youWereRoleblockedMessage,
                youSurvivedAttackMessage: youSurvivedAttackMessage === undefined || youSurvivedAttackMessage === null ? false : youSurvivedAttackMessage,
                youWereProtectedMessage: youWereProtectedMessage === undefined || youWereProtectedMessage === null ? false : youWereProtectedMessage,
                youWereTransportedMessage: youWereTransportedMessage === undefined || youWereTransportedMessage === null ? false : youWereTransportedMessage,
                youWerePossessedMessage: youWerePossessedMessage === undefined || youWerePossessedMessage === null ? false : youWerePossessedMessage,
                yourTargetWasJailedMessage: yourTargetWasJailedMessage === undefined || yourTargetWasJailedMessage === null ? false : yourTargetWasJailedMessage
            });
        },
        sendSetForgerWill(role: Role | null, will: string) {
            this.server.sendPacket({
                type: "setForgerWill",
                role: role,
                will: will
            });
        },

        messageListener(serverMessage) {
            messageListener(serverMessage);
        },

        lastPingTime: 0,
        pingCalculation: 0,
        tick(timePassedMs) {
            if (gameManager.state.stateType !== "disconnected") {
                if(gameManager.lastPingTime + (30 * 1000) < Date.now()){
                    gameManager.lastPingTime = Date.now();
                    this.server.sendPacket({
                        type: "ping"
                    });
                }
            }
            if (gameManager.state.stateType === "game") {
                if (!gameManager.state.ticking) return;

                const newTimeLeft = gameManager.state.timeLeftMs - timePassedMs;
                if (Math.floor(newTimeLeft / 1000) < Math.floor(gameManager.state.timeLeftMs / 1000)) {
                    gameManager.invokeStateListeners("tick");
                }
                gameManager.state.timeLeftMs = newTimeLeft;
                if (gameManager.state.timeLeftMs < 0) {
                    gameManager.state.timeLeftMs = 0;
                }
            }
        },
    }
    return gameManager;
}
function createServer(){

    let Server: Server = {
        ws: null,

        open : () => {
            let address = CONFIG.address;
            Server.ws = new WebSocket(address);

            let completePromise: () => void;
            let promise = new Promise<void>((resolver) => {
                completePromise = resolver;
            });

            Server.ws.onopen = (event: Event)=>{
                completePromise();
                console.log("Connected to server.");
                
                Anchor.setContent(<PlayMenu/>);
            };
            Server.ws.onclose = (event: CloseEvent)=>{
                console.log("Disconnected from server.");
                if (Server.ws === null) return; // We closed it ourselves

                Anchor.pushError(translate("notification.connectionFailed"), "");
                Anchor.setContent(<StartMenu/>);
            };
            Server.ws.onmessage = (event: MessageEvent<string>)=>{
                GAME_MANAGER.messageListener(
                    JSON.parse(event.data) as ToClientPacket
                );
            };
            Server.ws.onerror = (event: Event) => {
                Server.close();
                Anchor.pushError(translate("notification.connectionFailed"), translate("notification.serverNotFound"));
                Anchor.setContent(<StartMenu/>);
            };
            
            return promise;
        },

        sendPacket : (packet: ToServerPacket)=>{
            if (Server.ws === null) {
                console.error("Attempted to send packet to null websocket!");
            } else {
                Server.ws.send(JSON.stringify(packet));
            }
        },

        close : ()=>{
            if(Server.ws === null) return;
            
            Server.ws.close();
            Server.ws = null;
        }
        
    }
    return Server;
}

export function isValidPhaseTime(time: number) {
    return Number.isSafeInteger(time) && time <= 1000 && 0 <= time;
}

export type { GameManager, Server } from "./gameManager.d";
