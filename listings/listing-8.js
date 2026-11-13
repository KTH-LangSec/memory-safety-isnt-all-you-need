// SPDX-License-Identifier: CC0-1.0

var obj1 = {}, obj2 = {}

// ---

var i1 = "__proto__", i2 = "p"
obj1[i1][i2] = () => "console.log('code reuse attack')"
var x = obj2.p()
eval(x)
