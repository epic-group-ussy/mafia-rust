import React from "react";
import translate from "../../../game/lang";
import GAME_MANAGER from "../../../index";
import GameScreen, { ContentMenus } from "../GameScreen";
import "./willMenu.css"
import { StateListener } from "../../../game/gameManager.d";


type FieldType = "will" | "notes";
type Fields = { [key in FieldType]: string };

interface WillMenuState {
    syncedFields : Fields
    localFields: Fields
}

export default class WillMenu extends React.Component<{}, WillMenuState> {
    listener: StateListener

    constructor(props: {}) {
        super(props);

        let gameStateFields = {
            will: GAME_MANAGER.gameState.will,
            notes: GAME_MANAGER.gameState.notes
        };

        this.state = {
            syncedFields: gameStateFields,
            localFields: gameStateFields
        };
        this.listener = () => {
            this.setState({
                syncedFields: {
                    will: GAME_MANAGER.gameState.will,
                    notes: GAME_MANAGER.gameState.notes,
                }
            })
        };  
    }
    componentDidMount() {
        GAME_MANAGER.addStateListener("yourWill", this.listener);
        GAME_MANAGER.addStateListener("yourNotes", this.listener);
    }
    componentWillUnmount() {
        GAME_MANAGER.removeStateListener("yourWill", this.listener);
        GAME_MANAGER.removeStateListener("yourNotes", this.listener);
    }
    send(type: FieldType) {
        this.save(type);
        GAME_MANAGER.sendSendMessagePacket('\n' + this.state.localFields[type])
    }
    save(type: FieldType) {
        if (type === "will")
            GAME_MANAGER.sendSaveWillPacket(this.state.localFields[type])
        else if (type === "notes")
            GAME_MANAGER.sendSaveNotesPacket(this.state.localFields[type])
    }
    renderInput(type: FieldType) {
        return (<div className="textarea-section">
            {translate("menu.will." + type)}
            <button 
                className={this.state.syncedFields[type] !== this.state.localFields[type] ? "highlighted" : undefined}
                onClick={() => this.save(type)}
            >
                {translate("menu.will.save")}
            </button>
            <button onClick={() => this.send(type)}>
                {translate("menu.will.post")}
            </button>
            <textarea
                value={this.state.localFields[type]}
                onChange={(e) => {
                    let fields = this.state.localFields;
                    fields[type] = e.target.value;
                    this.setState({ localFields: fields });
                }}
                onKeyDown={(e) => {
                    if (e.ctrlKey) {
                        if (e.key === 's') {
                            // Prevent the Save dialog from opening
                            e.preventDefault();
                            this.save(type);
                        } else if (e.key === "Enter") {
                            this.send(type);
                        }
                    }
                }}>
            </textarea>
        </div>)
    }
    render() {return (<div className="will-menu">
        <button onClick={()=>{GameScreen.instance.closeMenu(ContentMenus.WillMenu)}}>
            {translate("menu.will.title")}
        </button>
        <section>
            {this.renderInput("will")}
            {this.renderInput("notes")}
        </section>
    </div>);}
}