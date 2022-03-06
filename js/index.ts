const { promisify } = require("util");

const binary = require('node-pre-gyp');
const path = require('path');
const binding_path = binary.find(path.resolve(path.join(__dirname,'../package.json')));
const { luaNew, luaClose, luaDoString, luaTableForEach, luaTableToString } = require(binding_path);

const luaDoStringAsync = promisify(luaDoString);

type LuaValue = null | string | number | LuaTable;

function convertLua(val: any): LuaValue {
    if (val == null) {
        return null;
    }
    
    // TODO: Handle other things than just tables...
    if (typeof val === 'object') {
        return new LuaTable(val);
    }

    return val as LuaValue;
}

class Lua {
    private lua: any;
    
    constructor() {
        this.lua = luaNew();
    } 

    async doString(code: string): Promise<LuaValue> {
        return convertLua(await luaDoStringAsync.call(this.lua, code));
    }

    close() {
        luaClose.call(this.lua);
    }
}

class LuaTable {
    table: any;

    constructor(table: any) {
        this.table = table;
    }
    
    forEach(callback: (key: LuaValue, value: LuaValue) => void) {
        luaTableForEach.call(this.table, (key: any, value: any) => {
            callback(convertLua(key), convertLua(value));
        });
    }

    toString(): string {
        return luaTableToString.call(this.table);
    }
}

module.exports = { Lua, LuaTable };
