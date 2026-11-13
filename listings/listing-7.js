// SPDX-License-Identifier: CC0-1.0

var obj = {}
var sink = () => console.log("sink reached")

// ---

var i1 = "toString"
var x = obj[i1]
if (x && x()) sink()
