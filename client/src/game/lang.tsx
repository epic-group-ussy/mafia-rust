
export let langMap: ReadonlyMap<string, string>;
export let langText: string;
export let langJson: any;

export const LANGUAGES = ["en_us", "broken_keyboard", "dyslexic", "funny"] as const;
export type Language = typeof LANGUAGES[number]
switchLanguage("en_us");

function getLangMapAndJson(language: Language): [ReadonlyMap<string, string>, any] {
    const langJson = require("../resources/lang/" + language + ".json");
    const langMap = new Map<string, string>(Object.entries(langJson));
    return [langMap, langJson]
}

export function switchLanguage(language: Language) {
    const [map, json] = getLangMapAndJson(language)
    langJson = json;
    langMap = map;
    langText = JSON.stringify(json, null, 1);
}

/// Returns the translated string with the given key, replacing the placeholders with the given values.
export default function translate(langKey: string, ...valuesList: (string | number)[]): string {
    const translation = translateChecked(langKey, ...valuesList);

    if (translation === null) {
        console.error("Attempted to use non existent lang key: "+langKey);
        return "ERROR: "+langKey;
    } 

    return translation;
}

export function translateAny(langKeys: string[], ...valuesList: (string | number)[]): string {
    return translateAnyWithLanguage(langMap, langKeys, ...valuesList);
}

function translateAnyWithLanguage(langMap: ReadonlyMap<string, string>, langKeys: string[], ...valuesList: (string | number)[]): string {
    for (const key of langKeys) {
        if (!langMap.has(key)) continue;

        const translation = fillTemplate(langMap.get(key)!, ...valuesList);

        if (translation !== null) {
            return translation;
        }
    }

    if (langMap.has('fallbackLanguage')) {
        const [newLangMap] = getLangMapAndJson(langMap.get('fallbackLanguage')! as Language)
        translateAnyWithLanguage(newLangMap, langKeys, ...valuesList)
    }

    console.error("Attempted to use non existent lang key: "+langKeys.at(-1));
    return "ERROR: "+langKeys.at(-1);
}

export function translateChecked(langKey: string, ...valuesList: (string | number)[]): string | null {
    return translateCheckedWithLanguage(langMap, langKey, ...valuesList)
}

function translateCheckedWithLanguage(langMap: ReadonlyMap<string, string>, langKey: string, ...valuesList: (string | number)[]): string | null {
    const out = langMap.get(langKey);
    if(out === undefined){
        if (!langMap.has('fallbackLanguage')) return null;

        const [newLangMap] = getLangMapAndJson(langMap.get('fallbackLanguage')! as Language)
        return translateCheckedWithLanguage(newLangMap, langKey, ...valuesList)
    }
    return fillTemplate(out, ...valuesList);
}

function fillTemplate(template: string, ...valuesList: (string | number)[]): string {
    for(let i = 0; i < valuesList.length; i++){
        template = template.replace("\\"+(i), valuesList[i] as string);
    }
    return template;
}

export function languageName(language: Language): string {
    const json = require("../resources/lang/" + language + ".json");
    return json.language;
}