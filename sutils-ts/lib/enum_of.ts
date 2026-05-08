export type ConstStringArray = readonly [..._: string[]]

export type IEnum<Keys extends ConstStringArray> = {
    [K in Keys[number]]: number
}

export type EnumCombine<Keys extends ConstStringArray> = Enum<Keys> & IEnum<Keys>

export class Enum<Keys extends ConstStringArray> {
    ".keys": Keys
    private constructor(keys: Keys) {
        this[".keys"] = keys
        keys.forEach((key: keyof IEnum<Keys>, idx) => (this as IEnum<Keys>)[key] = idx)
    }
    static Of = <Keys extends ConstStringArray>(keys: Keys) =>
        new Enum(keys) as EnumCombine<Keys>
}