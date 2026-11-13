// SPDX-License-Identifier: CC0-1.0

var PAYLOAD = "console.log('code reuse attack')"

// ---

var obj1 = {}, obj2 = {}

// ---

Object.prototype.p = true
Object.prototype.q = PAYLOAD
var i1 = "p", i2 = "q"
var x = obj1[i1]
if (x) { var y = obj2[i2]; eval(y) }
