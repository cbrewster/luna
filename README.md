# @cbrewster/luna

Node.js bindings to Lua.

```javascript
import { Lua, LuaTable } from '@cbrewster/luna';

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
    table.forEach((key, val) => {
      console.log(`key: ${key} val: ${val}`);
    });
  }
  
  const selfReference = await lua.doString("x = {} \n x.self = x \n return x");
  if (selfReference instanceof LuaTable) {
    selfReference.forEach((key, val) => {
      console.log(`key: ${key} val: ${val}`);
      if (val?.toString() === selfReference.toString()) {
        console.log("we have a cycle!");
        console.log("Table: ", selfReference.toString());
      }
    });
  }

  lua.close();
})()
```
