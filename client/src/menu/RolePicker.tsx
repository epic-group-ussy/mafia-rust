import React from "react";
import { RoleListEntry } from "../game/gameState.d";
import { ROLES } from "../game/gameState";

interface RolePickerProps {
    roleListEntry: RoleListEntry,
    onChange: (value: RoleListEntry) => void
}

// Can convert to function component
export default class RolePicker extends React.Component<RolePickerProps> {
    render() {
        let selectors: JSX.Element[] = [];
        
        if (this.props.roleListEntry.type === "any") {
            selectors = [
                <select 
                    key="faction" 
                    value="Any"
                    onChange={(e)=>{this.updateRolePicker("faction", e.target.value)}}
                > {
                    allFactions().map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>
            ];
        } else if (this.props.roleListEntry.type === "faction" ) {
            let faction = getFaction(this.props.roleListEntry);
            selectors = [
                <select
                    key="faction" 
                    value={faction}
                    onChange={(e)=>{this.updateRolePicker("faction", e.target.value)}}
                > {
                    allFactions().map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>,
                <select
                    key="alignment"
                    value={"Random"}
                    onChange={(e)=>{this.updateRolePicker("alignment", e.target.value)}}
                > {
                    allAlignments(faction).map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>
            ]
        } else if (this.props.roleListEntry.type === "factionAlignment") {
            let faction = getFaction(this.props.roleListEntry);
            let alignment = getAlignment(this.props.roleListEntry);
            selectors = [
                <select
                    key="faction" 
                    value={faction}
                    onChange={(e)=>{this.updateRolePicker("faction", e.target.value)}}
                > {
                    allFactions().map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>,
                <select
                    key="alignment"
                    value={alignment}
                    onChange={(e)=>{this.updateRolePicker("alignment", e.target.value)}}
                > {
                    allAlignments(faction).map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>,
                <select
                    key="exact"
                    value={"Random"}
                    onChange={(e)=>{this.updateRolePicker("exact", e.target.value)}}
                > {
                    allRoles(faction, alignment).map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>
            ]
        } else {
            let faction = getFaction(this.props.roleListEntry);
            let alignment = getAlignment(this.props.roleListEntry);
            selectors = [
                <select
                    key="faction"
                    value={faction}
                    onChange={(e)=>{this.updateRolePicker("faction", e.target.value)}}
                > {
                    allFactions().map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>,
                <select
                    key="alignment"
                    value={alignment}
                    onChange={(e)=>{this.updateRolePicker("alignment", e.target.value)}}
                > {
                    allAlignments(faction).map((alignment: string, key) => {
                        return <option key={key}> {alignment} </option>
                    })
                } </select>,
                <select
                    key="exact"
                    value={this.props.roleListEntry.role}
                    onChange={(e)=>{this.updateRolePicker("exact", e.target.value)}}
                > {
                    allRoles(faction, alignment).map((faction: string, key) => {
                        return <option key={key}> {faction} </option>
                    })
                } </select>
            ]
        }
        
        return <div className="lm-role-picker-container">
            {selectors}
        </div>
    }

    updateRolePicker(selector: "faction" | "alignment" | "exact", value: string) {
        let roleListEntry = this.props.roleListEntry;
        switch (selector) {
            case "faction":
                if (value === "Any") {
                    roleListEntry = {
                        type: "any"
                    }
                } else {
                    roleListEntry = {
                        type: "faction",
                        faction: value
                    }
                }
            break;
            case "alignment":
                if (value === "Random") {
                    roleListEntry = {
                        type: "faction",
                        faction: getFaction(this.props.roleListEntry)
                    }
                } else {
                    roleListEntry = {
                        type: "factionAlignment",
                        faction: getFaction(this.props.roleListEntry),
                        factionAlignment: getFaction(this.props.roleListEntry) + value
                    }
                }
            break;
            case "exact":
                if (value === "Random") {
                    roleListEntry = {
                        type: "factionAlignment",
                        faction: getFaction(this.props.roleListEntry),
                        factionAlignment: getFaction(this.props.roleListEntry) + getAlignment(this.props.roleListEntry)
                    }
                } else {
                    roleListEntry = {
                        type: "exact",
                        faction: getFaction(this.props.roleListEntry),
                        factionAlignment: getFaction(this.props.roleListEntry) + getAlignment(this.props.roleListEntry),
                        role: value
                    }
                }
            break;
        }

        this.props.onChange(roleListEntry);
    }
}

function getFaction(roleListEntry: RoleListEntry): string {
    if (roleListEntry.type === "any") {
        throw Error("Couldn't find a faction for Any")
    } else {
        return roleListEntry.faction;
    }
}

function getAlignment(roleListEntry: RoleListEntry): string {
    if (roleListEntry.type === "any" || roleListEntry.type === "faction") {
        throw Error("Couldn't find an alignment for " + roleListEntry);
    } else {
        let factionAlignment = roleListEntry.factionAlignment;
        return factionAlignment.replace(roleListEntry.faction, "");
    }
}

function allFactions(): string[] {
    let factions: string[] = [];
    for (let [, role] of ROLES) {
        let faction = role.faction;
        if (!factions.includes(faction)) {
            factions.push(faction);
        }
    }
    factions.push("Any");
    return factions;
}

function allAlignments(faction: string): string[] {
    let alignments: string[] = [];

    for (let [, role] of ROLES) {
        if (role.faction !== faction) continue;

        if (!alignments.includes(role.alignment)) {
            alignments.push(role.alignment);
        }
    }

    alignments.push("Random");
    return alignments;
}

function allRoles(faction: string, alignment: string): string[] {
    let roles = [];

    for (let [name, role] of ROLES) {
        if (role.faction !== faction) continue;
        if (role.alignment !== alignment && alignment !== "Random") continue;

        roles.push(name);
    }

    roles.push("Random");
    return roles;
}