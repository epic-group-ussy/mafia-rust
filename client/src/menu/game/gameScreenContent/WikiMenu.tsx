import React from "react";
import GAME_MANAGER from "../../../index";
import GameScreen, { ContentMenus } from "../GameScreen";
import WikiSearch from "../../WikiSearch";
import "./wikiMenu.css"
import GameState from "../../../game/gameState.d";
import translate from "../../../game/lang";

interface WikiMenuProps {
}
interface WikiMenuState {
    gameState: GameState,
}


export default class WikiMenu extends React.Component<WikiMenuProps, WikiMenuState> {
    listener: () => void;
    
    constructor(props : WikiMenuProps) {
        super(props);

        this.state = {
            gameState : GAME_MANAGER.gameState,
        };
        this.listener = ()=>{
            this.setState({
                gameState: GAME_MANAGER.gameState,
            })
        };  
    }
    componentDidMount() {
        GAME_MANAGER.addStateListener(this.listener);
    }
    componentWillUnmount() {
        GAME_MANAGER.removeStateListener(this.listener);
    }


    render(){return(<div className="wiki-menu">
        {/* TODO: Use content-tab on other menus */}
        <div className="content-tab">
            <button onClick={()=>{GameScreen.instance.closeMenu(ContentMenus.WikiMenu)}}>{translate("menu.wiki.title")}</button>
        </div>
        <WikiSearch/>
    </div>)}
}