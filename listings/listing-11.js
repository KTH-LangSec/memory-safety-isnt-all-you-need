// SPDX-License-Identifier: CC0-1.0

var PAYLOAD = "console.log('code reuse attack')"

// ---

var obj1 = {}, obj2 = {}

// ---

Object.prototype.p = "q"
Object.prototype.q = PAYLOAD
var i1 = "p"
var x = obj1[i1]
var y = obj2[x]
eval(y)
