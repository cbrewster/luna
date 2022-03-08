# @cbrewster/luna

Node.js bindings to Lua.

```javascript
const { Lua } = require('@cbrewster/luna');

(async () => {
    const lua = new Lua();
    const result = await lua.doString("return 1 + 2");
    console.log(result); // 3
})()
```
