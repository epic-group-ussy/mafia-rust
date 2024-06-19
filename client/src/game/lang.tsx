export let langMap: ReadonlyMap<string, string>;
export let langText: string;
export let langJson: any;

export const LANGUAGES = ["en_us", "broken_keyboard", "dyslexic"] as const;
export type Language = typeof LANGUAGES[number];

import en_us from "../resources/lang/en_us.json";
import broken_keyboard from "../resources/lang/broken_keyboard.json";
import dyslexic from "../resources/lang/dyslexic.json";

switchLanguage("en_us");

// Because vite cannot automatically import
function getLanguageJson(language: Language) {
    let langJson: any = null;
    switch(language) {
        case "en_us":
            langJson = en_us;
            break;
        case "broken_keyboard":
            langJson = broken_keyboard;
            break;
        case "dyslexic":
            langJson = dyslexic;
            break;
    }

    return langJson;
}

export function switchLanguage(language: Language) {
    langJson = getLanguageJson(language);;
    langMap = new Map<string, string>(Object.entries(langJson));
    langText = JSON.stringify(langJson, null, 1);
}

/// Returns the translated string with the given key, replacing the placeholders with the given values.
export default function translate(langKey: string, ...valuesList: (string | number)[]): string {
    let out = langMap.get(langKey);
    if(out===undefined){
        console.error("Attempted to use non existent lang key: "+langKey);
        return "ERROR: "+langKey;
    }
    for(let i = 0; i < valuesList.length; i++){
        out = out.replace("\\"+(i), valuesList[i] as string);
    }
    return out;
}

export function translateChecked(langKey: string, ...valuesList: (string | number)[]): string | null {
    let out = langMap.get(langKey);
    if(out===undefined){
        return null;
    }
    for(let i = 0; i < valuesList.length; i++){
        out = out.replace("\\"+(i), valuesList[i] as string);
    }
    return out;
}

export function languageName(language: Language): string {
    const json = getLanguageJson(language);
    return json.language;
}