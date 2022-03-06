const { Lua, LuaTable } = require('../');

(async () => {
    const lua = new Lua();
    const table = await lua.doString("x = {}; x.self = x; return x");
    console.log(table instanceof LuaTable);
    table.forEach((key, val) => {
        console.log(`key: ${key}, val: ${val}`);
        console.log(val instanceof LuaTable);
        console.log("table equalness", table === val);
        val.forEach((key, val) => {
            console.log(`key: ${key}, val: ${val}`);
        });
    });

    lua.close();
})();
