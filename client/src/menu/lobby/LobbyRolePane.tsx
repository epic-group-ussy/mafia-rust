import React, { ReactElement, useEffect, useState } from "react";
import GAME_MANAGER from "../../index";
import "../../index.css";
import { StateListener } from "../../game/gameManager.d";
import translate from "../../game/lang";
import { RoleOutline } from "../../game/roleListState.d";
import RoleOutlineSelector from "../../components/RolePicker";

export default function LobbyRolePane(): ReactElement {

    const [roleList, setRoleList] = useState(
        GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game" ? GAME_MANAGER.state.roleList : []
    );
    const [host, setHost] = useState(
        GAME_MANAGER.getMyHost() ?? false
    );

    useEffect(() => {
        const listener: StateListener = (type) => {
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game"){
                switch (type) {
                    case "roleList":
                        setRoleList([...GAME_MANAGER.state.roleList]);
                        break;
                    case "roleOutline":
                        setRoleList([...GAME_MANAGER.state.roleList]);
                        break;
                    case "playersHost":
                        setHost(GAME_MANAGER.getMyHost() ?? false);
                        break;
                }
            }
        }

        if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game"){
            setRoleList([...GAME_MANAGER.state.roleList]);
            setHost(GAME_MANAGER.getMyHost() ?? false);
        }

        GAME_MANAGER.addStateListener(listener);
        return ()=>{GAME_MANAGER.removeStateListener(listener);}
    }, [setRoleList, setHost]);



    let onChangeRolePicker = (index: number, value: RoleOutline) => {
        let newRoleList = [...roleList];
        newRoleList[index] = value;
        setRoleList(newRoleList);
        GAME_MANAGER.sendSetRoleOutlinePacket(index, value);
    }

    return <section className="graveyard-menu-colors">
        <h2>{translate("menu.lobby.roleList")}</h2>
        <button disabled={!host}
            onClick={()=>{
                GAME_MANAGER.sendSimplifyRoleListPacket();
            }}>
            {translate("simplify")}
        </button>
        {roleList.map((outline, index) => {
            return <RoleOutlineSelector
                disabled={!host}
                roleOutline={outline}
                onChange={(value: RoleOutline) => {onChangeRolePicker(index, value);}}
                key={index}
            />
        })}
    </section>
    
}
