import React from "react";
import translate from "../../../game/lang";
import GAME_MANAGER from "../../../index";
import "./playerListMenu.css"
import "./../gameScreen.css"
import { PhaseType, Player, PlayerIndex } from "../../../game/gameState.d";
import { ContentMenu, ContentTab } from "../GameScreen";
import { StateListener } from "../../../game/gameManager.d";
import StyledText from "../../../components/StyledText";
import { RoleState } from "../../../game/roleState.d";
import Icon from "../../../components/Icon";
import { Button } from "../../../components/Button";
import { Grave } from "../../../game/graveState";
import RoleSpecificSection from "../../../components/RoleSpecific";

type PlayerListMenuProps = {
}
type PlayerListMenuState = {
    players: Player[],
    phase: PhaseType | null,
    voted: PlayerIndex | null,
    targets: PlayerIndex[],
    roleState: RoleState | null,
    graveRolesStrings: (string | null)[],
    playerFilter: PlayerFilter,
    chatFilter: PlayerIndex | null,

    myIndex: PlayerIndex,
    roleSpecificOpen: boolean,
}
type PlayerFilter = "all"|"living"|"usable";


//indexed by player index, returns the role on the players grave
function getGraveRolesStrings(graves: Grave[], playerCount: number): (string | null)[] {


    let rolesStrings: (string | null)[] = [];
    
    for(let i = 0; i < playerCount; i++){
        rolesStrings.push(null);
    }

    graves.forEach((grave: Grave)=>{
        if (grave.information.type === "normal"){
            rolesStrings[grave.player] = "("+translate("role."+grave.information.role+".name")+")";
        } else {
            rolesStrings[grave.player] = "("+translate("obscured")+")";
        }
    });

    return rolesStrings;
}

export default class PlayerListMenu extends React.Component<PlayerListMenuProps, PlayerListMenuState> {
    listener: StateListener;
    updatePlayerFilter: () => void;

    constructor(props: PlayerListMenuProps) {
        super(props);

        
        if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
            this.state = {
                players: GAME_MANAGER.state.players,
                phase: GAME_MANAGER.state.phaseState.type,
                voted: GAME_MANAGER.state.clientState.voted,
                targets: GAME_MANAGER.state.clientState.targets,
                roleState: GAME_MANAGER.state.clientState.roleState,
                graveRolesStrings: getGraveRolesStrings(GAME_MANAGER.state.graves, GAME_MANAGER.state.players.length),
                playerFilter: "living",
                chatFilter: null,
                myIndex: GAME_MANAGER.state.clientState.myIndex??0,
                roleSpecificOpen: false
            };

        this.updatePlayerFilter = () => {
            if(GAME_MANAGER.state.stateType !== "game" || GAME_MANAGER.state.clientState.type !== "player"){
                return;
            }

            let playerFilter = this.state.playerFilter;
            if(
                (GAME_MANAGER.state.clientState.myIndex===null || GAME_MANAGER.state.players[GAME_MANAGER.state.clientState.myIndex].alive) && 
                playerFilter !== "all"
            ){
                if(GAME_MANAGER.state.phaseState.type === "night"){
                    playerFilter = "usable"
                }else if(GAME_MANAGER.state.phaseState.type === "obituary"){
                    playerFilter = "living";
                }
            }
            //if there are no usable players, switch to living
            if(playerFilter==="usable" && !GAME_MANAGER.state.players.some((player)=>{return Object.values(player.buttons).includes(true)})){
                playerFilter = "living";
            }
            //if there are no living players, switch to all
            if(playerFilter==="living" && !GAME_MANAGER.state.players.some((player)=>{return player.alive})){
                playerFilter = "all";
            }
            this.setState({
                playerFilter: playerFilter
            })
        };

        this.listener = (type)=>{
            if(GAME_MANAGER.state.stateType !== "game")
                return;
            
            switch (type) {
                case "filterUpdate":
                    if(GAME_MANAGER.state.clientState.type === "player")
                        this.setState({chatFilter: GAME_MANAGER.state.clientState.chatFilter});
                break;
                case "phase":
                    this.setState({ phase: GAME_MANAGER.state.phaseState.type });
                    if (GAME_MANAGER.state.phaseState.type === "night")
                        this.setState({ roleSpecificOpen: true });
                break;
                case "gamePlayers":
                case "yourButtons":
                case "playerAlive":
                case "yourPlayerTags":
                case "yourRoleLabels":
                case "playerVotes":
                    this.setState({ players: GAME_MANAGER.state.players });
                    this.setState({ graveRolesStrings: getGraveRolesStrings(GAME_MANAGER.state.graves, GAME_MANAGER.state.players.length) });
                break;
                case "yourVoting":
                    if(GAME_MANAGER.state.clientState.type === "player")
                        this.setState({ voted: GAME_MANAGER.state.clientState.voted });
                break;
                case "yourSelection":
                    if(GAME_MANAGER.state.clientState.type === "player")
                        this.setState({ targets: GAME_MANAGER.state.clientState.targets });
                break;
                case "yourRoleState":
                    if(GAME_MANAGER.state.clientState.type === "player")
                        this.setState({ roleState: GAME_MANAGER.state.clientState.roleState });
                break;
                case "addGrave":
                    this.setState({ graveRolesStrings: getGraveRolesStrings(GAME_MANAGER.state.graves, GAME_MANAGER.state.players.length) });
                break;
                case "yourPlayerIndex":
                    if(GAME_MANAGER.state.clientState.type === "player")
                        this.setState({ myIndex: GAME_MANAGER.state.clientState.myIndex??0 });
                break;
            }
            switch (type) {
                case "phase":
                case "gamePlayers":
                case "yourVoting":
                case "yourSelection":
                case "yourRoleState":
                    this.updatePlayerFilter();
                break;
            }
        };  
    }
    componentDidMount() {
        GAME_MANAGER.addStateListener(this.listener);
    }
    componentWillUnmount() {
        GAME_MANAGER.removeStateListener(this.listener);
    }

    renderPhaseSpecific(){
        let phaseSpecificJSX = null;
        switch(this.state.phase){
            case "nomination":
                if(this.state.voted!=null){
                    phaseSpecificJSX = (<div>
                        <div><StyledText>{this.state.players[this.state.voted].toString()}</StyledText></div>
                        <button className="button gm-button" onClick={()=>{
                            GAME_MANAGER.sendVotePacket(null);
                        }}>{translate("menu.playerList.button.resetVote")}</button>
                    </div>);
                }
                else
                    phaseSpecificJSX = null;
                break;
            case "night":
                let targetStringList = this.state.targets.map((playerIndex: PlayerIndex)=>{
                    return this.state.players[playerIndex].toString();
                });

                if(targetStringList.length > 0){
                    phaseSpecificJSX = (<div>
                        <div><StyledText>{targetStringList.join(", ")+"."}</StyledText></div>
                        <button className="button gm-button" onClick={()=>{
                            GAME_MANAGER.sendTargetPacket([]);
                        }}>{translate("menu.playerList.button.resetTargets")}</button>
                    </div>);
                }
                else
                    phaseSpecificJSX =  null;
        }
        
        if(phaseSpecificJSX!==null){
            return <div className="phase-specific">{phaseSpecificJSX}</div>
        }
        return null;
    }

    renderPlayer(player: Player){

        let roleString = (()=>{
            if(player.index === this.state.myIndex){
                return ("("+translate("role."+this.state.roleState?.type+".name")+")");
            }

            if(player.alive && player.roleLabel != null){
                return ("("+translate("role."+player.roleLabel+".name")+")");
            }
            
            if (!player.alive && this.state.graveRolesStrings[player.index] != null){
                return this.state.graveRolesStrings[player.index];
            }

            return "";
        })();

        return(<div className="player" key={player.index}>
            <div className="top">
                {(
                    player.numVoted !==null &&
                    player.numVoted !==0 &&
                    this.state.phase === "nomination"
                ) ?
                    <span className="keyword-player-number">
                        {player.numVoted}
                    </span>
                : null}
                <button className="whisper" onClick={()=>{GAME_MANAGER.prependWhisper(player.index)}}>
                    <h4>
                        <StyledText>{(player.alive?"":" "+translate("dead.icon")+"")}</StyledText>
                    </h4>
                    <StyledText>{player.toString()}</StyledText>
                    {roleString!==null&&<StyledText>{roleString}</StyledText>}
                </button>
                <div className="playerTags">
                    <StyledText>{player.playerTags.map((tag)=>{return translate("tag."+tag)})}</StyledText>
                </div>
                {(() => {

                    const filter = player.index;
                    const isFilterSet = this.state.chatFilter === filter;
                    
                    return <Button 
                        className={"filter"} 
                        highlighted={isFilterSet}
                        onClick={() => {
                            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                                GAME_MANAGER.state.clientState.chatFilter = isFilterSet ? null : filter;
                                GAME_MANAGER.invokeStateListeners("filterUpdate");
                            }
                            this.setState({})
                            return true;
                        }}
                        pressedChildren={result => <Icon>{result ? "done" : "warning"}</Icon>}
                        aria-label={translate("menu.playerList.button.filter")}
                    >
                        <Icon>filter_alt</Icon>
                    </Button>
                })()}
            </div>
            

            <div className="buttons">
                <div className="day-target">
                    {player.buttons.dayTarget && <Button 
                        highlighted={
                            (this.state.roleState?.type === "jailor" && this.state.roleState.jailedTargetRef === player.index)
                            || 
                            (this.state.roleState?.type === "medium" && this.state.roleState.seancedTarget === player.index)
                            || 
                            (this.state.roleState?.type === "journalist" && this.state.roleState.interviewedTarget === player.index)
                            || 
                            (
                                this.state.roleState?.type === "marksman" && 
                                this.state.roleState.state.type === "marks" &&
                                (
                                    (this.state.roleState.state.marks.type === "one" && this.state.roleState.state.marks.a === player.index) ||
                                    (this.state.roleState.state.marks.type === "two" && (
                                        this.state.roleState.state.marks.a === player.index || 
                                        this.state.roleState.state.marks.b === player.index
                                    ))
                                )
                            )
                        } 
                        onClick={()=>GAME_MANAGER.sendDayTargetPacket(player.index)}
                    >
                        {translate("role."+this.state.roleState?.type+".dayTarget")}
                    </Button>}
                </div>
                <div className="target">
                    {((player) => {
                        if(player.buttons.target) {
                            return <button onClick={() => {
                                if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                                    GAME_MANAGER.sendTargetPacket([...GAME_MANAGER.state.clientState.targets, player.index])
                            }}>
                                {translate("role."+this.state.roleState?.type+".target")}
                            </button>
                        } else if (GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player" && this.state.phase === "night" && this.state.targets.includes(player.index)) {
                            let newTargets = [...GAME_MANAGER.state.clientState.targets];
                            newTargets.splice(newTargets.indexOf(player.index), 1);
                            return <Button highlighted={true} onClick={() => GAME_MANAGER.sendTargetPacket(newTargets)}>
                                {translate("cancel")}
                            </Button>
                        }
                    })(player)}
                </div>
                <div className="vote">
                    {player.buttons.vote && <button 
                        onClick={()=>GAME_MANAGER.sendVotePacket(player.index)}
                    >{translate("menu.playerList.button.vote")}</button>}
                </div>
            </div>            
        </div>)
    }
    renderPlayers(players: Player[]){
        return <div className="player-list">{
            players.filter((player: Player) => {
                switch(this.state.playerFilter){
                    case "all": return true;
                    case "living": return player.alive;
                    case "usable": return Object.values(player.buttons).includes(true);
                    default: return false;
                }
            }).map(player => this.renderPlayer(player))
        }</div>
    }

    renderFilterButton(filter: PlayerFilter) {
        return <Button 
            highlighted={this.state.playerFilter === filter}
            onClick={()=>this.setState({playerFilter: filter})}
        >
            {translate("menu.playerList.button." + filter)}
        </Button>
    }

    render(){return(<div className="player-list-menu player-list-menu-colors">
        <ContentTab close={ContentMenu.PlayerListMenu} helpMenu={"standard/playerList"}>{translate("menu.playerList.title")}</ContentTab>

        <details className="role-specific-colors small-role-specific-menu" open={this.state.roleSpecificOpen}>
            <summary
                onClick={(e)=>{
                    e.preventDefault();
                    this.setState({roleSpecificOpen: !this.state.roleSpecificOpen});
                }}
            >{translate("role."+this.state.roleState?.type+".name")}</summary>
            <RoleSpecificSection/>
        </details>
        
        <div>
            {this.renderFilterButton("all")}
            {this.renderFilterButton("living")}
            {this.renderFilterButton("usable")}
        </div>
        {this.renderPhaseSpecific()}
        {this.renderPlayers(this.state.players)}
    </div>)}
}
