
import React from "react";
import GAME_MANAGER from "../../index";
import { RoleOutline, sortRoleOutlines, translateRoleOutline } from "../../game/roleListState.d";
import "../../index.css";
import { StateListener } from "../../game/gameManager.d";
import translate from "../../game/lang";
import RolePicker from "../../components/RolePicker";
import StyledText from "../../components/StyledText";
import ROLES from "./../../resources/roles.json";
import { Role } from "../../game/roleState.d";
import EXCLUDED_ROLE_PRESETS from "./../../resources/excludedRolePresets.json";

type ExcludedRolesState = {
    excludedRoles: RoleOutline[],
    roleOutline: RoleOutline,
    selectedExcludedRolePreset: string,
    host: boolean
}

export default class LobbyExcludedRoles extends React.Component<{}, ExcludedRolesState> {
    listener: StateListener;

    constructor(props: {}){
        super(props);

        if(GAME_MANAGER.state.stateType === "lobby")
            this.state = {
                excludedRoles: GAME_MANAGER.state.excludedRoles,
                roleOutline: {type:"any"},
                selectedExcludedRolePreset: Object.keys(EXCLUDED_ROLE_PRESETS)[0],
                host: GAME_MANAGER.getMyHost() ?? false
            }

        this.listener = (type) => {
            if(GAME_MANAGER.state.stateType === "lobby")
                this.setState({
                    excludedRoles: GAME_MANAGER.state.excludedRoles,
                    host: GAME_MANAGER.getMyHost() ?? false
                });
        };
    }
    componentDidMount() {
        GAME_MANAGER.addStateListener(this.listener);
    }
    componentWillUnmount() {
        GAME_MANAGER.removeStateListener(this.listener);
    }

    includeRole(role: RoleOutline){
        let roles = [...this.state.excludedRoles];
        roles = roles.filter((value)=>value !== role);
        GAME_MANAGER.sendExcludedRolesPacket(roles);
    }
    excludeRole(){
        let roles = [...this.state.excludedRoles];

        if(this.state.roleOutline.type !== "any"){
            roles.push(this.state.roleOutline);
        }else{
            for(let role in ROLES){
                roles.push({
                    type: "exact",
                    role: role as Role,
                });
            }
        }

        GAME_MANAGER.sendExcludedRolesPacket(roles);
    }
    
    handleExcludedRolePreset(){
        let new_exclusions = this.state.excludedRoles;
        let preset = EXCLUDED_ROLE_PRESETS[this.state.selectedExcludedRolePreset as keyof typeof EXCLUDED_ROLE_PRESETS] as RoleOutline[];
        for(let outline of preset){
            new_exclusions.push(outline);
        }

        GAME_MANAGER.sendExcludedRolesPacket(new_exclusions);
    }
    handleIncludeAll(){
        GAME_MANAGER.sendExcludedRolesPacket([]);
    }
    

    render(){return(<section className="excluded-roles role-specific-colors">
        <header>
            <h2>{translate("menu.lobby.excludedRoles")}</h2>
        </header>
        <div className="exclusion-preset">
            <select
                onChange={(e)=>this.setState({selectedExcludedRolePreset: e.target.options[e.target.selectedIndex].value})}
                disabled={!this.state.host}
            >
                {
                    Object.keys(EXCLUDED_ROLE_PRESETS).map((value, i)=>{
                        return <option key={i} value={value}>{translate("menu.excludedRoles."+value)}</option>
                    })
                }
            </select>
        </div>
            <button 
                onClick={(e)=>this.handleExcludedRolePreset()}
                disabled={!this.state.host}
            >{translate("menu.excludedRoles.exclude")}</button>
            <button 
                onClick={(e)=>this.handleIncludeAll()}
                disabled={!this.state.host}
            >{translate("menu.excludedRoles.includeAll")}</button>
        <div>
            <RolePicker
                disabled={!this.state.host}
                roleOutline={this.state.roleOutline}
                onChange={(value: RoleOutline) => {
                    this.setState({
                        roleOutline: value
                    })
                }}
            />
            <button 
                disabled={!this.state.host}
                onClick={()=>{this.excludeRole()}}
            >{translate("menu.excludedRoles.exclude")}</button>
        </div>
        <div>
            {sortRoleOutlines(Array.from(this.state.excludedRoles.values())).map((value, i)=>{
                return <button key={i} 
                    disabled={!this.state.host}
                    onClick={()=>{this.includeRole(value)}}
                >
                    <StyledText noLinks={true}>
                        {translateRoleOutline(value) ?? ""}
                    </StyledText>
                </button>
            })}
        </div>
    </section>)}
}
