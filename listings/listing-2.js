// SPDX-License-Identifier: CC0-1.0

var PAYLOAD = "console.log('code reuse attack')"

// ---

var obj1 = {}, obj2 = {}

// ---

var i1 = "__proto__", i2 = "p", i3 = PAYLOAD
obj1[i1][i2] = i3
var x = obj2.p || "..."
eval(x)
