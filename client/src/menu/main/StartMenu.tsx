import React, { ReactElement, useContext } from "react";
import GAME_MANAGER from "../../index";
import "../../index.css"
import "./startMenu.css"
import translate from "../../game/lang";
import { AnchorControllerContext } from "../Anchor";
import PlayMenu from "./PlayMenu";
import LoadingScreen from "../LoadingScreen";
import GameModesEditor from "../../components/gameModeSettings/GameModesEditor";
import Icon from "../../components/Icon";
import SettingsMenu from "../Settings";
import StandaloneWiki from "./StandaloneWiki";
import { Button } from "../../components/Button";
import Credits from "./Credits";
import StyledText from "../../components/StyledText";

export default function StartMenu(): ReactElement {
    const { setContent: setAnchorContent, setCoverCard } = useContext(AnchorControllerContext)!;
    return <div className="sm">
        <main>
            <h1>
                <StyledText noLinks={true}>{translate("menu.start.title")}</StyledText>
            </h1>
            <div>
                <Button onClick={async () => {
                    setAnchorContent(<LoadingScreen type="default"/>);
                    if (await GAME_MANAGER.setOutsideLobbyState()) {
                        setAnchorContent(<PlayMenu/>);
                    } else {
                        setAnchorContent(<StartMenu/>);
                    }
                }}>
                    <Icon>play_arrow</Icon> {translate("menu.start.button.play")}
                </Button>
                <Button onClick={() => setCoverCard(<SettingsMenu />)}>
                    <Icon>settings</Icon> {translate("menu.settings.title")}
                </Button>
                <Button onClick={() => setCoverCard(<GameModesEditor/>)}>
                    <Icon>edit</Icon> {translate("menu.globalMenu.gameSettingsEditor")}
                </Button>
                <Button onClick={() => setAnchorContent(<StandaloneWiki/>)}>
                    <Icon>menu_book</Icon> {translate("menu.wiki.title")}
                </Button>
                <Button onClick={()=>{
                    window.open("https://discord.gg/Vxw7gFPfJj", "_blank")
                }}>
                    <Icon>public</Icon> Discord
                </Button>
            </div>
        </main>
        <footer>
            <nav>
                <ul>
                    <li>
                        <a href="https://www.github.com/ItsSammyM/mafia-rust">Github</a>
                    </li>
                    <li>
                        <Button onClick={()=>{
                            window.open("https://discord.gg/Vxw7gFPfJj", "_blank")
                        }}>
                            <Icon>public</Icon> Discord
                        </Button>
                    </li>
                    <li>
                        <Button onClick={()=>{setAnchorContent(<Credits/>)}}>{translate("credits")}</Button>
                    </li>
                    <li><a href="https://mafia.dev.jackpapel.com">Dev (Experimental)</a></li>
                </ul>
            </nav>
        </footer>
    </div>
}
