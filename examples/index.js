const { Lua, LuaTable } = require('../');

(async () => {
  const lua = new Lua();

  console.log(await lua.doString("x = 1")); // null
  console.log(await lua.doString("return x")); // 1

  try {
    await lua.doString("this is a syntax error"); // Error!
  } catch (e) {
    console.log(e); 
  }
  
  console.log(await lua.doString("return 1 + 2")); // 3
  console.log(await lua.doString(`return "abc"`)); // "abc"

  const table = await lua.doString(`return {key = "value", foo = "bar"}`);
  if (table instanceof LuaTable) {
    console.log("foo is: " + table.get("foo"));
    table.forEach((key, val) => {
      console.log(`key: ${key} val: ${val}`);
    });

    console.log("Set foo = baz");
    
    table.set("foo", "baz");
    table.forEach((key, val) => {
      console.log(`key: ${key} val: ${val}`);
    });

    console.log("Set foo = {a = b}")
    const a = lua.newTable();
    a.set("a", "b");
    table.set("foo", a);
    table.forEach((key, val) => {
      console.log(`key: ${key} val: ${val}`);
    });
  }
  
  const selfReference = await lua.doString("x = {} \n x.self = x \n return x");
  if (selfReference instanceof LuaTable) {
    console.log("self is: " + selfReference.get("self"));
    selfReference.forEach((key, val) => {
      console.log(`key: ${key} val: ${val}`);
      if (val?.toString() === selfReference.toString()) {
        console.log("we have a cycle!");
      }
    });
  }

  lua.close();
})()
