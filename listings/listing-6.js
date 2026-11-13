// SPDX-License-Identifier: CC0-1.0

var obj1 = {}, obj2 = {}, obj3 = {}
var sink = () => console.log("sink reached")

// ---

var i1 = "toString", i2 = "__proto__", i3 = "p"
var y = obj1[i1]
obj2[i2][i3] = y
var x = obj3.p
if (x && x()) sink()
