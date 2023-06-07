import { promisify } from "util";

const { find } = require('@mapbox/node-pre-gyp');
import { resolve, join } from 'path';
const binding_path = find(resolve(join(__dirname,'../package.json')));
const { 
    luaNew, luaClose, luaDoString, luaNewTable,
    luaTableGet, luaTableSet, luaTableForEach, luaTableToString,
} = require(binding_path);

const luaDoStringAsync = promisify(luaDoString);

export type LuaValue = null | string | number | LuaTable | Boolean;

// Handles checking output type of lua value and wraps tables in a class.
function wrapLua(val: any): LuaValue {
    if (val == null) {
        return null;
    }
    
    // TODO: Handle other things than just tables...
    if (typeof val === 'object') {
        return new LuaTable(val);
    }

    return val as LuaValue;
}

// Unwraps classes before sending lua values back to the Lua context.
function unwrapLua(val: LuaValue): any {
    if (val == null) {
        return null;
    }
    
    if (val instanceof LuaTable) {
       return val.table;         
    }

    return val;
}

export class Lua {
    private lua: any;
    
    constructor() {
        this.lua = luaNew();
    }

    async doString(code: string): Promise<LuaValue> {
        return wrapLua(await luaDoStringAsync.call(this.lua, code));
    }

    newTable(): LuaTable {
        return new LuaTable(luaNewTable.call(this.lua));
    }

    close() {
        luaClose.call(this.lua);
    }
}

export class LuaTable {
    table: any;

    constructor(table: null | any) {
        this.table = table;
    }

    get(key: LuaValue): LuaValue {
        return wrapLua(luaTableGet.call(this.table, unwrapLua(key)));
    }
    
    set(key: LuaValue, value: LuaValue): LuaValue {
        return wrapLua(luaTableSet.call(this.table, unwrapLua(key), unwrapLua(value)));
    }
    
    forEach(callback: (key: LuaValue, value: LuaValue) => void) {
        luaTableForEach.call(this.table, (key: any, value: any) => {
            callback(wrapLua(key), wrapLua(value));
        });
    }

    toString(): string {
        return luaTableToString.call(this.table);
    }
}
