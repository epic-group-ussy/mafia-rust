import { ReactElement, useCallback, useContext, useState } from "react"
import translate from "../../game/lang"
import React from "react"
import StyledText from "../StyledText"
import { ROLE_SETS, RoleOutlineOption, getAllRoles, getRolesFromOutlineOption } from "../../game/roleListState.d";
import { Role, roleJsonData } from "../../game/roleState.d";
import { RoleOutlineOptionSelector } from "./OutlineSelector";
import "./disabledRoleSelector.css"
import Icon from "../Icon";
import { Button } from "../Button";
import { GameModeContext } from "./GameModesEditor";
import CheckBox from "../CheckBox";




export default function EnabledRoleSelector(props: Readonly<{
    disabled?: boolean,
    onDisableRoles: (role: Role[]) => void,
    onEnableRoles: (role: Role[]) => void,
    onIncludeAll: () => void
}>): ReactElement {
    const {enabledRoles} = useContext(GameModeContext);

    const [roleOutlineOption, setRoleOutlineOption] = useState<RoleOutlineOption>({ type: "roleSet", roleSet: "town" });

    const disableOutlineOption = (outline: RoleOutlineOption) => {
        props.onDisableRoles(getRolesFromOutlineOption(outline));
    }

    const enableOutlineOption = (outline: RoleOutlineOption) => {
        props.onEnableRoles(getRolesFromOutlineOption(outline));
    }

    const disableAll = () => {
        props.onDisableRoles(getAllRoles());
    }

    return <div className="role-specific-colors selector-section">
        <h2>{translate("menu.lobby.enabledRoles")}</h2>
        {(props.disabled !== true) && <div>
            <Button
                onClick={props.onIncludeAll}
                disabled={props.disabled}
                ><Icon>deselect</Icon> {translate("menu.enabledRoles.includeAll")}</Button>
            <Button
                onClick={disableAll}
                disabled={props.disabled}
            ><Icon>select_all</Icon> {translate("menu.enabledRoles.excludeAll")}</Button>

            <div className="disabled-role-selector-area">
                <Button
                    onClick={()=>{enableOutlineOption(roleOutlineOption)}}
                    disabled={props.disabled}
                >{translate("menu.enabledRoles.include")}</Button>
                <Button
                    onClick={()=>{disableOutlineOption(roleOutlineOption)}}
                    disabled={props.disabled}
                >{translate("menu.enabledRoles.exclude")}</Button>
                <RoleOutlineOptionSelector
                    excludeAny={true}
                    disabled={props.disabled}
                    roleOutlineOption={roleOutlineOption}
                    onChange={setRoleOutlineOption}
                />
            </div>
        </div>}

        <EnabledRolesDisplay 
            enabledRoles={enabledRoles}
            modifiable={!props.disabled}
            onDisableRoles={props.onDisableRoles}
            onEnableRoles={props.onEnableRoles}
            disabled={props.disabled}
        />
    </div>
}

type EnabledRolesDisplayProps = {
    enabledRoles: Role[],
} & (
    {
        modifiable: true,
        onDisableRoles: (role: Role[]) => void,
        onEnableRoles: (role: Role[]) => void,
        disabled?: boolean,
    } |
    {
        modifiable?: false,
    }
)

export function EnabledRolesDisplay(props: EnabledRolesDisplayProps): ReactElement {
    const isEnabled = useCallback((role: Role) => props.enabledRoles.includes(role), [props.enabledRoles]);

    const roleTextElement = (role: Role) => {

        return <StyledText 
            noLinks={props.modifiable ?? false}
            className={!isEnabled(role) ? "keyword-disabled" : undefined}
        >
            {translate("role."+role+".name")}
        </StyledText>
    }

    const [hideDisabled, setHideDisabled] = useState(true);

    return <div>
        {!props.modifiable && <label>
            {translate("hideDisabled")}
            <CheckBox
                checked={hideDisabled}
                onChange={checked => setHideDisabled(checked)}
            />
        </label>}
        <div>
            {getAllRoles()
                .filter(role => isEnabled(role) || !hideDisabled || props.modifiable)
                .sort((a, b) => (isEnabled(a) ? -1 : 1) - (isEnabled(b) ? -1 : 1))
                .sort((a, b) => ROLE_SETS.indexOf(roleJsonData()[a].mainRoleSet) - ROLE_SETS.indexOf(roleJsonData()[b].mainRoleSet))
                .map((role, i) => 
                    props.modifiable 
                        ? <Button key={i}
                            disabled={props.disabled}
                            onClick={() => (!isEnabled(role) ? props.onEnableRoles : props.onDisableRoles)([role])}
                        >
                            {roleTextElement(role)}
                        </Button> 
                        : <div key={i} className={"placard" + (!isEnabled(role) ? " disabled" : "")}>
                            {roleTextElement(role)}
                        </div>)}
        </div>
    </div>
}