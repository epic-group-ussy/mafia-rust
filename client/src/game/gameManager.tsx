import { createGameState } from "./gameState";
import Anchor from "./../menu/Anchor";
import StartMenu from "./../menu/main/StartMenu";
import GAME_MANAGER from "./../index";
import messageListener from "./messageListener";
import CONFIG from "./../resources/config.json"
import React from "react";
import { Phase, PhaseTimes, RoleListEntry, Verdict } from "./gameState.d";
import { GameManager, Server, StateEventType, StateListener } from "./gameManager.d";
import { ToClientPacket, ToServerPacket } from "./packet";

export function createGameManager(): GameManager {

    console.log("Game manager created.");
    
    let gameManager: GameManager = {
        roomCode: null,

        gameState : createGameState(),

        server : createServer(),

        listeners : {},

        addStateListener(type: StateEventType | StateEventType[], listener) {
            (typeof type === "string" ? [type] : type).forEach(type => {
                gameManager.listeners[type] = gameManager.listeners[type] ?? []
                gameManager.listeners[type]!.push(listener);
            })
        },
        removeStateListener(type: StateEventType | StateEventType[], listener) {
            (typeof type === "string" ? [type] : type).forEach(type => {
                let index = gameManager.listeners[type]?.indexOf(listener);
                if(index !== undefined && index !== -1)
                    gameManager.listeners[type]!.splice(index, 1);
            })
        },
        invokeStateListeners(type: StateEventType) {
            (gameManager.listeners[type] ?? [])
                .map(listener => listener())
        },

        async tryJoinGame(roomCode: string) {
            GAME_MANAGER.roomCode = roomCode;
            
            GAME_MANAGER.server.close();
            await GAME_MANAGER.server.open();
            
            await GAME_MANAGER.sendJoinPacket();
        },

        leaveGame() {
            if (this.gameState.inGame) {
                // Let the server know it can disconnect us immediately. No need for a timer.
                this.server.sendPacket({type: "leave"});
            }
            // This is kind of lazy. It basically resets the URL to the "main menu" state and refreshes.
            // Clear query parameters from visible URL
            window.history.replaceState({}, document.title, window.location.pathname);
            window.location.reload();
        },

        sendHostPacket() {
            this.server.sendPacket({type: "host"});
        },
        sendJoinPacket() {
            let completePromise: () => void;
            let promise = new Promise<void>((resolver) => {
                completePromise = resolver;
            });
            
            let onJoined: StateListener = () => {
                completePromise();
                // This listener shouldn't stick around
                GAME_MANAGER.removeStateListener("acceptJoin", onJoined);
            };
            GAME_MANAGER.addStateListener("acceptJoin", onJoined);

            let actualCode: number = parseInt(gameManager.roomCode!, 18);

            this.server.sendPacket({
                type: "join",
                roomCode: isNaN(actualCode) ? 0 : actualCode
            });

            return promise;
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
        sendSetPhaseTimesPacket(phaseTimeSettings: PhaseTimes) {
            // No need for validity checks here - json should be valid.
            this.server.sendPacket({
                type: "setPhaseTimes",
                phaseTimeSettings
            });
        },
        sendSetRoleListPacket(roleListEntries: RoleListEntry[]) {
            this.server.sendPacket({
                type: "setRoleList",
                roleList: roleListEntries
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
        sendExcludedRolesPacket(roles){
            this.server.sendPacket({
                type:"setExcludedRoles",
                roles:roles
            })
        },
        
        messageListener(serverMessage) {
            messageListener(serverMessage);
        },
    
        tick(timePassedms) {
            gameManager.gameState.secondsLeft = Math.round(gameManager.gameState.secondsLeft - timePassedms/1000)
            if(gameManager.gameState.secondsLeft < 0)
                gameManager.gameState.secondsLeft = 0;
            gameManager.invokeStateListeners("tick");
        },
    }
    return gameManager;
}
function createServer(){

    let Server: Server = {
        ws: null,

        openListener : ()=>{
            //Server.ws.send("Hello to Server");
        },
        closeListener : ()=>{
            Anchor.setContent(<StartMenu/>);
        },
        messageListener: (event)=>{
            GAME_MANAGER.messageListener(
                JSON.parse(event.data) as ToClientPacket
            );
        },

        open : () => {
            let address = CONFIG.address;
            Server.ws = new WebSocket(address);

            let completePromise: () => void;
            let promise = new Promise<void>((resolver) => {
                completePromise = resolver;
            });

            Server.ws.addEventListener("open", (event: Event)=>{
                completePromise();
                Server.openListener(event);
            });
            Server.ws.addEventListener("close", (event: CloseEvent)=>{
                Server.closeListener(event);
            });
            Server.ws.addEventListener("message", (event: MessageEvent<string>)=>{
                Server.messageListener(event);
            });
            Server.ws.addEventListener("error", (event: Event) => {
                Anchor.queueError("Failed to connect", "Contact an admin to see if the server is online.");
                Anchor.setContent(<StartMenu/>);
            })
            
            return promise;
        },
        sendPacket : (packet: ToServerPacket)=>{
            if (Server.ws === null) {
                console.log("Attempted to send packet to null websocket!");
            } else {
                Server.ws.send(JSON.stringify(packet));
            }
        },
        close : ()=>{
            if(Server.ws==null) return;
            
            Server.ws.close();
            Server.ws.removeEventListener("close", Server.closeListener);
            Server.ws.removeEventListener("message", Server.messageListener);
            Server.ws.removeEventListener("open", Server.openListener);
            Server.ws = null;
        }
        
    }
    return Server;
}

export function isValidPhaseTime(time: number) {
    return Number.isSafeInteger(time) && time <= 1000 && 0 <= time;
}

export type { GameManager, Server } from "./gameManager.d";
