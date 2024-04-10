import GameState, { LobbyState, PhaseTimes, Player, PlayerGameState } from "./gameState.d"


export function defaultPhaseTimes(): PhaseTimes {
    return {
        briefing: 20,
        obituary: 10,
        discussion: 100,
        nomination: 60,
        testimony: 30,
        judgement: 30,
        finalWords: 7,
        dusk: 7,
        night: 45,
    }
}

export function createLobbyState(): LobbyState {
    return {
        stateType: "lobby",
        roomCode: 0,
        lobbyName: "Mafia Lobby",

        myId: null,

        roleList: [],
        excludedRoles: [],
        phaseTimes: defaultPhaseTimes(),

        players: {}
    }
}

export function createGameState(): GameState {
    return {
        stateType: "game",
        roomCode: 0,

        chatMessages : [],
        graves: [],
        players: [],
        
        phaseState: {type:"briefing"},
        timeLeftMs: 0,
        dayNumber: 1,

        fastForward: false,
        
        roleList: [],
        excludedRoles: [],
        phaseTimes: defaultPhaseTimes(),

        ticking: true,

        clientState: createPlayerGameState(),

    }
}

export function createPlayerGameState(): PlayerGameState {
    return {
        type: "player",

        myIndex: null,
        
        roleState: null,

        will: "",
        notes: "",
        crossedOutOutlines: [],
        chatFilter: null,
        deathNote: "",
        targets: [],
        voted: null,
        judgement: "abstain",

        sendChatGroups: [],
    }
}

export function createPlayer(name: string, index: number): Player {
    return{
        name: name,
        index: index,
        buttons: {
            dayTarget: false,
            target: false,
            vote: false,
        },
        numVoted: 0,
        alive: true,
        roleLabel: null,
        playerTags: [],
        host: false,

        toString() {
            return ""+(this.index+1)+": " + this.name;
        }
    }
}


