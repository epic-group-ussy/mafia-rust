import React from "react";
import "./rolePicker.css";
import { 
    Faction, FactionAlignment, Role, RoleListEntry, FACTIONS,
    getAlignmentStringFromFactionAlignment, 
    getAllFactionAlignments, 
    getFactionFromFactionAlignment, getFactionAlignmentFromRole, getFactionFromRoleListEntry, 
    getFactionAlignmentFromRoleListEntry, getFactionFromRole
} from "../game/gameState.d";
import translate from "../game/lang";
import ROLES from "../resources/roles.json";

interface RolePickerProps {
    roleListEntry: RoleListEntry,
    onChange: (value: RoleListEntry) => void
}

// Can convert to function component
export default class RolePicker extends React.Component<RolePickerProps> {
    setAny(){
        this.props.onChange({
            type: "any"
        })
    }
    setFaction(faction: Faction){
        this.props.onChange({
            type: "faction",
            faction: faction
        })
    }
    setFactionAlignment(factionAlignment: FactionAlignment){
        this.props.onChange({
            type: "factionAlignment",
            factionAlignment: factionAlignment
        })
    }
    setExact(role: Role){
        this.props.onChange({
            type: "exact",
            role: role
        })
    }

    setFirstBox(e: { target: { selectedIndex: number; }; }){
        let selected = allFactionsAndAny()[e.target.selectedIndex];

        if(selected === "any"){
            this.setAny();
        } else {
            this.setFaction(selected);
        }
    }
    setSecondBox(e: { target: { selectedIndex: number; }; }){
        let currentFaction = getFactionFromRoleListEntry(this.props.roleListEntry);
        if(currentFaction === null)
            return;
        
        let selected = allFactionAlignmentsAndAny(currentFaction)[e.target.selectedIndex];

        if(selected === "any"){
            this.setFaction(currentFaction);
        } else {
            this.setFactionAlignment(selected);
        }
    }
    setThirdBox(e: { target: { selectedIndex: number; }; }){
        let currentFactionAlignment = getFactionAlignmentFromRoleListEntry(this.props.roleListEntry);
        if(currentFactionAlignment === null)
            return;

        let selected = allRolesAndAny(currentFactionAlignment)[e.target.selectedIndex];

        if(selected === "any"){
            this.setFactionAlignment(currentFactionAlignment);
        } else {
            this.setExact(selected);
        }
    }
    
    render() {
        let selectors: JSX.Element[] = [];
        
        switch(this.props.roleListEntry.type){

            case "any":
                selectors = [
                    <select 
                        key="faction" 
                        value={translate("any")}
                        onChange={(e)=>this.setFirstBox(e)}
                    > {
                        allFactionsAndAny().map((faction: Faction | "any", key) => {
                            if(faction === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate("faction."+faction)}</option>
                        })
                    } </select>
                ];
            break;
            case "faction":
                selectors = [
                    <select 
                        key="faction" 
                        value={translate("faction."+this.props.roleListEntry.faction)}
                        onChange={(e)=>this.setFirstBox(e)}
                    > {
                        allFactionsAndAny().map((faction: Faction | "any", key) => {
                            if(faction === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate("faction."+faction)}</option>
                        })
                    } </select>,
                    
                    <select
                        key="alignment"
                        value={translate("any")}
                        onChange={(e)=>this.setSecondBox(e)}
                    > {
                        allFactionAlignmentsAndAny(this.props.roleListEntry.faction).map((factionAlignment: FactionAlignment | "any", key) => {
                            if(factionAlignment === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate("alignment."+getAlignmentStringFromFactionAlignment(factionAlignment))}</option>
                        })
                    } </select>
                ]
            break;
            case "factionAlignment":
                selectors = [
                    <select
                        key="faction" 
                        value={translate("faction."+getFactionFromFactionAlignment(this.props.roleListEntry.factionAlignment))}
                        onChange={(e)=>this.setFirstBox(e)}
                    > {
                        allFactionsAndAny().map((faction: string, key) => {
                            if(faction === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate("faction."+faction)}</option>
                        })
                    } </select>,

                    <select
                        key="alignment"
                        value={translate("alignment."+getAlignmentStringFromFactionAlignment(this.props.roleListEntry.factionAlignment))}
                        onChange={(e)=>this.setSecondBox(e)}
                    > {
                        allFactionAlignmentsAndAny(getFactionFromFactionAlignment(this.props.roleListEntry.factionAlignment)).map((factionAlignment: string, key) => {
                            if(factionAlignment === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate("alignment."+getAlignmentStringFromFactionAlignment(factionAlignment as FactionAlignment))}</option>
                        })
                    } </select>,
                    <select
                        key="exact"
                        value={translate("any")}
                        onChange={(e)=>this.setThirdBox(e)}
                    > {
                        allRolesAndAny(this.props.roleListEntry.factionAlignment).map((role: string, key) => {
                            if(role === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate(`role.${role}.name`)}</option>
                        })
                    } </select>
                ]
            break;
            case "exact":
                selectors = [
                    <select
                        key="faction" 
                        value={translate("faction."+getFactionFromRole(this.props.roleListEntry.role))}
                        onChange={(e)=>this.setFirstBox(e)}
                    > {
                        allFactionsAndAny().map((faction: string, key) => {
                            if(faction === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate("faction."+faction)}</option>
                        })
                    } </select>,

                    <select
                        key="alignment"
                        value={translate("alignment."+getAlignmentStringFromFactionAlignment(getFactionAlignmentFromRole(this.props.roleListEntry.role)))}
                        onChange={(e)=>this.setSecondBox(e)}
                    > {
                        allFactionAlignmentsAndAny(getFactionFromRole(this.props.roleListEntry.role)).map((factionAlignment: string, key) => {
                            if(factionAlignment === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate("alignment."+getAlignmentStringFromFactionAlignment(factionAlignment as FactionAlignment))}</option>
                        })
                    } </select>,
                    <select
                        key="exact"
                        value={translate(`role.${this.props.roleListEntry.role}.name`)}
                        onChange={(e)=>this.setThirdBox(e)}
                    > {
                        allRolesAndAny(getFactionAlignmentFromRole(this.props.roleListEntry.role)).map((role: string, key) => {
                            if(role === "any")
                                return <option key={key}>{translate("any")}</option>
                            return <option key={key}>{translate(`role.${role}.name`)}</option>
                        })
                    } </select>
                ]
            break;
        }
        
        return <div className="role-picker">
            {selectors}
        </div>
    }

    
}

function allFactionsAndAny(): (Faction | "any")[] {
    return ["any" as (Faction | "any")].concat(FACTIONS);
}

function allFactionAlignmentsAndAny(faction: Faction): (FactionAlignment | "any")[] {
    return ["any" as (FactionAlignment | "any")].concat(getAllFactionAlignments(faction.toLowerCase() as Faction));
}

function allRolesAndAny(factionAlignment: FactionAlignment): (Role | "any")[] {
    let roles: (Role | "any")[] = ["any"];

    for(let role of Object.keys(ROLES)){
        if(getFactionAlignmentFromRole(role as Role) === factionAlignment)
            roles.push(role as Role);
    }
    

    return roles;
}