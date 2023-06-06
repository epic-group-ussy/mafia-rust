import React from "react";
import GAME_MANAGER from "../index";
import { ChatMessage, NightInformation } from "./chatMessage";
import ROLES from "../resources/roles.json";
import { FactionAlignment, Player, getFactionFromFactionAlignment } from "./gameState.d";

let lang: ReadonlyMap<string, string>;
switchLanguage("en_us");

export function switchLanguage(language: string) {
    let json = require("../resources/lang/" + language + ".json");
    lang = new Map<string, string>(Object.entries(json))
}

export default function translate(langKey: string, ...valuesList: any[]): string {
    let out = lang.get(langKey);
    if(out===undefined){
        console.error("Attempted to use non existant lang key: "+langKey);
        return "ERROR: "+langKey;
    }
    for(let i = 0; i < valuesList.length; i++){
        out = out.replace("\\"+(i), valuesList[i]);
    }
    return out;
}

export function getChatElement(message: ChatMessage, key: number): JSX.Element {
    switch (message.type) {
        case "normal":
            if(message.messageSender.type === "player"){
                let playerIndex = message.messageSender.player;
                if(message.chatGroup !== "dead"){
                    return <span key={key}>{styleText(translate("chatmessage.normal",
                        GAME_MANAGER.gameState.players[playerIndex].toString(),
                        message.text
                    ), { 
                        indentStyle: { marginLeft: "2rem" } 
                    })}</span>;
                }else{
                    return <span key={key} style={{backgroundColor:"black", borderRadius: "5px"}}>{styleText(translate("chatmessage.normal",
                        GAME_MANAGER.gameState.players[playerIndex].toString(),
                        message.text
                    ), {
                        defaultStyle: { color: "grey" },
                        indentStyle: { marginLeft: "2rem" } 
                    })}</span>;
                }
            } else {
                //TODO, this only works because jailor and medium are the only options
                return <span key={key}>{styleText(translate("chatmessage.normal",
                    translate("role."+message.messageSender.type+".name"),
                    message.text
                ), {
                    defaultStyle: {color:"turquoise"}
                })}</span>;
            }
        case "whisper":
            return <span key={key}>{styleText(translate("chatmessage.whisper", 
                GAME_MANAGER.gameState.players[message.fromPlayerIndex].toString(),
                GAME_MANAGER.gameState.players[message.toPlayerIndex].toString(),
                message.text
            ), {
                defaultStyle: {color:"turquoise"}
            })}</span>;
        case "broadcastWhisper":
            return <span key={key}>{styleText(translate("chatmessage.broadcastWhisper",
                GAME_MANAGER.gameState.players[message.whisperer].toString(),
                GAME_MANAGER.gameState.players[message.whisperee].toString(),
            ), {
                defaultStyle: {color:"turquoise"}
            })}</span>;
        case "roleAssignment":
            let role = message.role;
            let name = translate("role."+role+".name")
            
            return <span key={key} style={{textAlign:"center"}}>{styleText(translate("chatmessage.roleAssignment", name), {
                defaultStyle: {color:"yellow"}
            })}</span>;
        case "playerDied":
            //TODO, role doesnt work properly
            let graveRoleString: string;
            if (message.grave.role.type === "role") {
                graveRoleString = translate(`role.${message.grave.role.role}.name`);
            } else {
                graveRoleString = translate(`grave.role.${message.grave.role.type}`);
            }

            let deathCause: string;
            if (message.grave.deathCause.type === "lynching") {
                deathCause = translate("grave.deathCause.lynching")
            } else {
                let killers: string[] = [];
                for (let killer of message.grave.deathCause.killers) {
                    if(killer.type === "role") {
                        killers.push(translate(`role.${killer.value}.name`))
                    }else if(killer.type === "faction") {
                        killers.push(translate(`faction.${killer.value}`))
                    }else{
                        killers.push(translate(`grave.killer.${killer.type}`))
                    }
                }
                deathCause = killers.join();
            }

            return <span key={key}>{styleText(translate("chatmessage.playerDied",
                GAME_MANAGER.gameState.players[message.grave.playerIndex].toString(),
                graveRoleString,
                deathCause,
                message.grave.will
            ), {
                defaultStyle: {color:"yellow"}
            })}</span>;
        case "phaseChange":
            return <span key={key} style={{textAlign:"center", backgroundColor:"var(--primary-color)"}}>{styleText(translate("chatmessage.phaseChange",
                translate("phase."+message.phase),
                message.dayNumber
            ), {
                defaultStyle: {color:"yellow", textDecoration:"underline"}
            })}</span >;
        case "trialInformation":
            return <span key={key}>{styleText(translate("chatmessage.trialInformation",
                message.requiredVotes,
                message.trialsLeft
            ), {
                defaultStyle: {color:"orange"}
            })}</span>;
        case "voted":
            if (message.votee !== null) {
                return <span key={key}>{styleText(translate("chatmessage.voted",
                    GAME_MANAGER.gameState.players[message.voter],
                    GAME_MANAGER.gameState.players[message.votee],
                ), {
                    defaultStyle: {color:"orange"}
                })}</span>;
            } else {
                return <span key={key}>{styleText(translate("chatmessage.voted.cleared",
                    GAME_MANAGER.gameState.players[message.voter],
                ), {
                    defaultStyle: {color:"orange"}
                })}</span>;
            }
        case "playerOnTrial":
            return <span key={key}>{styleText(translate("chatmessage.playerOnTrial",
                GAME_MANAGER.gameState.players[message.playerIndex],
            ), {
                defaultStyle: {color:"yellow"}
            })}</span>;
        case "judgementVote":
            return <span key={key}>{styleText(translate("chatmessage.judgementVote",
                GAME_MANAGER.gameState.players[message.voterPlayerIndex],
            ), {
                defaultStyle: {color:"orange"}
            })}</span>;
        case "judgementVerdict":
            return <span key={key}>{styleText(translate("chatmessage.judgementVerdict",
                GAME_MANAGER.gameState.players[message.voterPlayerIndex],
                translate("verdict."+message.verdict.toLowerCase())
            ), {
                defaultStyle: {color:"orange"}
            })}</span>;
        case "trialVerdict":
            return <span key={key}>{styleText(translate("chatmessage.trialVerdict",
                GAME_MANAGER.gameState.players[GAME_MANAGER.gameState.playerOnTrial!].toString(),
                message.innocent>=message.guilty?translate("verdict.innocent"):translate("verdict.guilty"),
                message.innocent,
                message.guilty
            ), {
                defaultStyle: {color:"yellow"}
            })}</span>;
        case "nightInformation":
            return <span key={key}>{styleText(getNightInformationString(message.nightInformation), {
                defaultStyle: {color:"green"}
            })}</span>;
        case "targeted":
            if (message.targets.length > 0) {
                return <span key={key}>{styleText(translate("chatmessage.targeted",
                    GAME_MANAGER.gameState.players[message.targeter],
                    message.targets.map((target) => GAME_MANAGER.gameState.players[target].toString()).join(", ")
                ), {
                    defaultStyle: {color:"orange"}
                })}</span>;
            } else {
                return <span key={key}>{styleText(translate("chatmessage.targeted.cleared",
                    GAME_MANAGER.gameState.players[message.targeter],
                ), {
                    defaultStyle: {color:"orange"}
                })}</span>;
            }
        case "mayorRevealed":
            return <span key={key}>{styleText(translate("chatmessage.mayorRevealed",
                GAME_MANAGER.gameState.players[message.playerIndex].toString(),
            ), {
                defaultStyle: {color:"violet"}
            })}</span>;
        case "jailedTarget":
            return <span key={key}>{styleText(translate("chatmessage.jailedTarget",
                GAME_MANAGER.gameState.players[message.playerIndex].toString(),
            ), {
                defaultStyle: {color:"violet"}
            })}</span>;
        case "jailedSomeone":
            return <span key={key}>{styleText(translate("chatmessage.jailedSomeone",
                GAME_MANAGER.gameState.players[message.playerIndex].toString()
            ), {
                defaultStyle: {color:"violet"}
            })}</span>;
        default:
            console.error("Unknown message type: "+message.type);
            console.error(message);
            return <span key={key}>{styleText(translate("chatmessage."+message))}</span>;
    }
}

// TODO make night information message union type (& make an interface) and make this a method
export function getNightInformationString(info: NightInformation){
    switch (info.type) {
        case "roleBlocked":
            return translate("chatmessage.night.roleBlocked" + (info.immune ? ".immune" : ""));
        case "sheriffResult":
            return translate("chatmessage.night.sheriffResult." + (info.suspicious ? "suspicious" : "innocent"));
        case "lookoutResult":
            return translate("chatmessage.night.lookoutResult", (info.players.map((playerIndex) => GAME_MANAGER.gameState.players[playerIndex].toString()).join(", ")));
        case "playerRoleAndWill":
            return translate("chatmessage.night.playersRoleAndWill", translate("role."+info.role+".name"), info.will);
        default:
            return translate("chatmessage.night."+info.type);
    }
}


function styleSubstrings(
    string: string, 
    stringsToStyle: {
        string: string, 
        style: React.CSSProperties,
        className:string|undefined,
    }[], 
    styleOverride: {
        defaultStyle?: React.CSSProperties, 
        indentStyle?: React.CSSProperties,
    } = {}
): JSX.Element[]{

    let defaultStyle = styleOverride.defaultStyle !== undefined ? styleOverride.defaultStyle : {};
    let indentStyle = styleOverride.indentStyle !== undefined ? styleOverride.indentStyle : {};

    type StyledOrNot = {
        type: "string"
        string: string 
    } | {
        type: "styled"
        string: string
        style: React.CSSProperties
    } | {
        type: "br"
    }

    let finalOutList: StyledOrNot[] = [];

    //add in br
    string.split("\n").forEach((v, i, a) => {
        finalOutList.push({type: "string", string: v});
        if(i !== a.length-1) 
            finalOutList.push({type: "br"});
    });


    for(let i in stringsToStyle){
        for(let j in finalOutList){

            let current = finalOutList[j];
            if(current === undefined){
                continue;
            }
            if(current.type !== "string"){
                continue;
            }

            
            const regEscape = (v: string) => v.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, '\\$&');

            let currentStringSplit = current.string.split(RegExp(regEscape(stringsToStyle[i].string), "gi"));


            let currentOutList: StyledOrNot[] = []; 

            for(let str of currentStringSplit){
                if(str !== "")
                    currentOutList.push({
                        type: "string",
                        string: str
                    });

                currentOutList.push({
                    type: "styled",
                    string: stringsToStyle[i].string, 
                    style: stringsToStyle[i].style
                });
            }
            currentOutList.pop();

            //inject outlist into finaloutlist at position j, without using splice
            finalOutList = 
                finalOutList.slice(0, Number(j))
                .concat(currentOutList)
                .concat(finalOutList.slice(Number(j)+1));
        }
    }

    //turn into jsx
    let outJsxList = [];
    let lines = 0;
    for(let [i, current] of finalOutList.entries()){
        if(current.type === "br"){
            lines++;
            outJsxList.push(<br key={i}/>);
        }else if(current.type === "string"){
            outJsxList.push(
            <span key={i} style={lines === 0 ? defaultStyle : {...defaultStyle, ...indentStyle}}>
                {current.string}
            </span>);
        }else if(current.type === "styled"){
            outJsxList.push(
            <span key={i}
                style={lines === 0 ? current.style : {...current.style, ...indentStyle}}
            >
                {current.string}
            </span>);
        }
    }

    return outJsxList;
}

export function styleText(
    string: string, 
    styleOverride: {
        defaultStyle?: React.CSSProperties, 
        indentStyle?: React.CSSProperties
    } = {}
): JSX.Element[]{
    let stringsToStyle: {string: string, style: React.CSSProperties, className:string|undefined}[] = [];


    stringsToStyle = stringsToStyle.concat(
        GAME_MANAGER.gameState.players.map((player: Player)=>{
            return {string:player.toString(), style:{
                fontStyle: "italic",
                fontWeight: "bold"
            }, className: undefined};
        })
    );

    for(let role in ROLES){
        let roleObject = ROLES[role as keyof typeof ROLES];

        switch(getFactionFromFactionAlignment(roleObject.factionAlignment as FactionAlignment)){
            case "coven":
                stringsToStyle.push({string:translate("role."+role+".name"), style:{
                    color: "magenta"
                }, className: undefined});
                break;
            case "town":
                stringsToStyle.push({string:translate("role."+role+".name"), style:{
                    color: "lime"
                }, className: undefined});
                break;
            case "mafia":
                stringsToStyle.push({string:translate("role."+role+".name"), style:{
                    color: "red"
                }, className: undefined});
                break;
            case "neutral":
                stringsToStyle.push({string:translate("role."+role+".name"), style:{
                    color: "orange"
                }, className: undefined});
                break;
        }
    }

    stringsToStyle = stringsToStyle.concat([
        {string:translate("verdict.guilty"), style:{color:"red"}, className:undefined},
        {string:translate("verdict.innocent"), style:{color:"lime"}, className:undefined},
        {string:translate("verdict.abstain"), style:{color:"cyan"}, className:undefined},

        {string:translate("grave.role.cleaned"), style:{fontStyle: "italic", fontWeight: "bold"}, className:undefined},
        {string:translate("grave.role.petrified"), style:{fontStyle: "italic", fontWeight: "bold"}, className:undefined},
        {string:translate("suspicious"), style:{color:"red"}, className:undefined},

        {string:translate("faction.town"), style:{color:"lime"}, className:undefined},
        {string:translate("faction.mafia"), style:{color:"red"}, className:undefined},
        {string:translate("faction.neutral"), style:{color:"orange"}, className:undefined},
        {string:translate("faction.coven"), style:{color:"magenta"}, className:undefined},

        {string:translate("alignment.killing"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.investigative"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.protective"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.support"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.deception"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.evil"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.chaos"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.utility"), style:{color:"lightblue"}, className:undefined},
        {string:translate("alignment.power"), style:{color:"lightblue"}, className:undefined},

        {string:translate("any"), style:{color:"lightblue"}, className:undefined},
        {string:translate("none"), style:{color:"lightblue"}, className:undefined},
        {string:translate("basic"), style:{color:"lightblue"}, className:undefined},
        {string:translate("powerful"), style:{color:"lightblue"}, className:undefined},
        {string:translate("unstoppable"), style:{color:"lightblue"}, className:undefined},
        {string:translate("invincible"), style:{color:"lightblue"}, className:undefined},
        {string:translate("dead"), style:{fontStyle: "italic", color:"gray"}, className:undefined},

        {string:translate("menu.wiki.abilities"), style:{color:"lightblue"}, className:undefined},
        {string:translate("menu.wiki.attributes"), style:{color:"lightblue"}, className:undefined},

        {string:translate("grave.killer.suicide"), style:{color:"lightblue"}, className:undefined}
    ]);

    return styleSubstrings(string, stringsToStyle, styleOverride);
}
