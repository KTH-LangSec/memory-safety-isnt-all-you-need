// SPDX-License-Identifier: CC0-1.0

var PAYLOAD = "console.log('code reuse attack')"

// ---

var obj = {}

// ---

Object.prototype.p = { q: PAYLOAD }
var i1 = "p"
var x = obj[i1]
if (x) { var y = x.q; eval(y) }
