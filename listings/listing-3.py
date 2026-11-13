# SPDX-License-Identifier: CC0-1.0

PAYLOAD = "print('code reuse attack')"

# ---

class A:
  def __init__(self):
    pass

class B:
  p = "print('benign')"
  def __init__(self):
    pass

o = A()

# ---

i1, i2, i3, i4, i5, i6 = "__class__", "__init__", "__globals__", "B", "p", PAYLOAD
setattr(getattr(getattr(getattr(o, i1), i2), i3)[i4], i5, i6)
x = B.p
eval(x)
