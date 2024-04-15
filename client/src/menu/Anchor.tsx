import React, { JSXElementConstructor, MouseEventHandler, ReactElement, useRef } from "react";
import "../index.css";
import "./anchor.css";
import { switchLanguage } from "../game/lang";
import SettingsMenu, { DEFAULT_SETTINGS } from "./Settings";
import { loadSettings } from "../game/localStorage";
import LoadingScreen from "./LoadingScreen";
import { Theme } from "..";
import { ChatMessage } from "../components/ChatMessage";

type AnchorProps = {
    content: JSX.Element,
    onMount: () => void
}
type AnchorState = {
    mobile: boolean,
    content: JSX.Element,
    coverCard: JSX.Element | null,
    coverCardTheme: Theme,
    errorCard: JSX.Element | null,

    settings_menu: boolean,

    audio: HTMLAudioElement,

    touchStartX: number | null,
    touchCurrentX: number | null,
}

const MIN_SWIPE_DISTANCE = 40;

export default class Anchor extends React.Component<AnchorProps, AnchorState> {
    private static instance: Anchor;
    private static queueIsPlaying: boolean = false;
    private static audioQueue: Array<string> = [];

    swipeEventListeners: Array<(right: boolean) => void> = [];

    constructor(props: AnchorProps) {
        super(props);

        this.state = {
            mobile: false,
            content: this.props.content,
            coverCard: null,
            coverCardTheme: null,
            errorCard: null,

            settings_menu: false,

            audio: new Audio(),

            touchStartX: null,
            touchCurrentX: null,
        }
    }
    componentDidMount() {
        Anchor.instance = this;

        const settings = loadSettings();
        Anchor.instance.state.audio.volume = settings.volume ?? DEFAULT_SETTINGS.volume;
        switchLanguage(settings.language ?? "en_us")

        window.addEventListener("resize", Anchor.onResize);
        Anchor.onResize();

        this.props.onMount()
    }
    componentWillUnmount() {
        window.removeEventListener("resize", Anchor.onResize);
    }
    
    private static onResize() {
        const mobile = window.innerWidth <= 600;
        if (Anchor.instance.state.mobile && !mobile) {
            console.info("Switching to desktop layout");
        } else if (mobile && !Anchor.instance.state.mobile) {
            console.info("Switching to mobile layout");
        }
        Anchor.instance.setState({mobile});
    }


    static reload() {
        const content = Anchor.instance.state.content;
        Anchor.instance.setState({content: <LoadingScreen type="default"/>}, () => {
            Anchor.instance.setState({content});
        });

        const coverCard = Anchor.instance.state.coverCard;
        Anchor.instance.setState({coverCard: null}, () => {
            Anchor.instance.setState({coverCard});;
        });

        const errorCard = Anchor.instance.state.errorCard;
        Anchor.instance.setState({errorCard: null}, () => {
            Anchor.instance.setState({errorCard});;
        });
    }

    static queueAudioFile(src: string) {
        Anchor.audioQueue.push(src);

        if(!Anchor.queueIsPlaying) {
            Anchor.playAudioQueue();
        }
    }
    static playAudioQueue() {
        if(Anchor.audioQueue.length > 0) {

            Anchor.queueIsPlaying = true;
            Anchor.playAudioFile(Anchor.audioQueue[0], () => {
                Anchor.audioQueue = Anchor.audioQueue.slice(1)
                Anchor.playAudioQueue();
            });
        }else{
            Anchor.queueIsPlaying = false;
        }
    }
    static clearAudioQueue() {
        Anchor.audioQueue = [];
        Anchor.stopAudio();
    }
    static playAudioFile(src: string | null, onEnd?: () => void){
        Anchor.instance.state.audio.pause();
        if(src === null) return;
        Anchor.instance.state.audio.src = src;
        Anchor.instance.state.audio.load();

        const setStateCallback = () => {
            const onEnded = () => {
                if(onEnd !== undefined) onEnd();
                Anchor.instance.state.audio.removeEventListener("ended", onEnded);
            }

            Anchor.startAudio();
            Anchor.instance.state.audio.addEventListener("ended", onEnded);
        }

        Anchor.instance.setState({
            audio: Anchor.instance.state.audio
        }, setStateCallback);
    }
    static startAudio() {
        let playPromise = Anchor.instance.state.audio.play();
        playPromise.then(() => {

            Anchor.instance.state.audio.currentTime = 0;

            // Anchor.instance.state.audio.duration;
            // Anchor.instance.state.audio.currentTime = 45;
            Anchor.instance.state.audio.playbackRate = 1;
            // if(Anchor.instance.state.audio.duration !== Infinity && !Number.isNaN(Anchor.instance.state.audio.duration)){
            //     let startTime = Math.ceil(Anchor.instance.state.audio.duration - (timeLeftSeconds ?? 0));
            //     if (startTime > 0 && startTime < Anchor.instance.state.audio.duration) {
            //         console.log("Starting audio at " + startTime + " seconds")
            //         Anchor.instance.state.audio.currentTime = startTime;
            //     };
            // }
        }).catch((error) => {
            console.log("Audio failed to play: " + error);
        }); 
    }
    static stopAudio() {
        Anchor.instance.state.audio.pause();
    }



    static closeSettings() {
        Anchor.instance.setState({settings_menu: false});
    }
    static openSettings() {
        Anchor.instance.setState({settings_menu: true});
    }

    static addSwipeEventListener(listener: (right: boolean) => void) {
        Anchor.instance.swipeEventListeners = [...Anchor.instance.swipeEventListeners, listener];
    }
    static removeSwipeEventListener(listener: (right: boolean) => void) {
        Anchor.instance.swipeEventListeners = Anchor.instance.swipeEventListeners.filter((l) => l !== listener);
    }

    onTouchStart(e: React.TouchEvent<HTMLDivElement>) {
        this.setState({
            touchStartX: e.targetTouches[0].clientX,
            touchCurrentX: e.targetTouches[0].clientX
        });
    }
    onTouchMove(e: React.TouchEvent<HTMLDivElement>) {
        this.setState({
            touchCurrentX: e.targetTouches[0].clientX
        });
    }
    onTouchEnd(e: React.TouchEvent<HTMLDivElement>) {

        if(this.state.touchStartX !== null && this.state.touchCurrentX !== null){
            if(this.state.touchStartX - this.state.touchCurrentX > MIN_SWIPE_DISTANCE) {
                for(let listener of this.swipeEventListeners) {
                    listener(false);
                }
            }else if(this.state.touchStartX - this.state.touchCurrentX < -MIN_SWIPE_DISTANCE) {
                for(let listener of this.swipeEventListeners) {
                    listener(true);
                }
            }
        }

        this.setState({
            touchStartX: null,
            touchCurrentX: null
        });
    }
    

    render(){
        return <div
            className="anchor"
            onTouchStart={(e) => {this.onTouchStart(e)}}
            onTouchMove={(e) => {this.onTouchMove(e)}}
            onTouchEnd={(e) => {this.onTouchEnd(e)}}
        >
            <button className="material-icons-round settings-button" onClick={() => {
                this.setState({settings_menu: !this.state.settings_menu});
            }}>menu</button>
            {this.state.settings_menu && <SettingsMenu 
                onVolumeChange={(volume) => {
                    Anchor.instance.state.audio.volume = volume;
                    this.setState({
                        audio: this.state.audio
                    });
                }}
                onClickOutside={() => this.setState({settings_menu: false})}
            />}
            {this.state.content}
            {this.state.coverCard && <CoverCard 
                theme={this.state.coverCardTheme}
                onClickOutside={() => this.setState({coverCard: null})}
            >{this.state.coverCard}</CoverCard>}
            {this.state.errorCard}
        </div>
    }

    public static setContent(content: JSX.Element){
        Anchor.instance.setState({content : content});
    }
    public static contentType(): string | JSXElementConstructor<any> {
        return Anchor.instance.state.content.type;
    }
    public static setCoverCard(coverCard: JSX.Element, theme?: Theme, callback?: () => void){
        const coverCardTheme = theme === undefined ? null : theme;
        Anchor.instance.setState({ coverCard, coverCardTheme }, callback);
    }
    public static pushError(title: string, body: string) {
        Anchor.instance.setState({errorCard: <ErrorCard
            onClose={() => Anchor.instance.setState({ errorCard: null })}
            error={{title, body}}
        />});
    }
    public static clearCoverCard() {
        Anchor.instance.setState({coverCard: null, coverCardTheme: null});
    }

    public static isMobile(): boolean {
        return Anchor.instance.state.mobile;
    }
}

function CoverCard(props: { children: React.ReactNode, theme: Theme, onClickOutside: MouseEventHandler<HTMLDivElement> }): ReactElement {
    const ref = useRef<HTMLDivElement>(null);
    return <div 
        className={`anchor-cover-card-background-cover ${props.theme ?? ""}`} 
        onClick={e => {
            if (e.target === ref.current) props.onClickOutside(e)
        }}
        ref={ref}
    >
        <div className="anchor-cover-card">
            <button className="material-icons-round close-button" onClick={()=>{
                Anchor.clearCoverCard()
            }}>
                close
            </button>
            <div className="anchor-cover-card-content">
                {props.children}
            </div>
        </div>
    </div>
}

type Error = {
    title: string,
    body: string
}

function ErrorCard(props: { error: Error, onClose: () => void }) {
    return <div className="error-card" onClick={() => props.onClose()}>
        <header>
            {props.error.title}
            <button className="close">✕</button>
        </header>
        <div>{props.error.body}</div>
    </div>
}


export type AudioFile = 
    "church_bell.mp3" | 
    "alarm.mp3" | 
    "vine_boom.mp3" | 
    "sniper_shot.mp3" | 
    "normal_message.mp3" | 
    "whisper_broadcast.mp3";

export type AudioFilePath = `audio/${AudioFile}`;

export function chatMessageToAudio(msg: ChatMessage): AudioFilePath | null {
    let file: AudioFile|null = null;

    switch(msg.variant.type){
        case "normal":
        case "voted":
            file = "normal_message.mp3";
            break;
        case "broadcastWhisper":
            file = "whisper_broadcast.mp3";
            break;
        case "playerDied": 
            file = "church_bell.mp3";
            break;
        case "deputyKilled": 
            file = "sniper_shot.mp3";
            break;
    }

    if(file){
        return `audio/${file}`;
    }else{
        return null
    }
}