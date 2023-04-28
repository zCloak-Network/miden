
## std::math::u256
| Procedure | Description |
| ----------- | ------------- |
| mul_unsafe | Performs addition of two unsigned 256 bit integers discarding the overflow.<br /><br />The input values are assumed to be represented using 32 bit limbs, but this is not checked.<br /><br />Stack transition looks as follows:<br /><br />[b7, b6, b5, b4, b3, b2, b1, b0, a7, a6, a5, a4, a3, a2, a1, a0, ...] -> [c7, c6, c5, c4, c3, c2, c1, c0, ...]<br /><br />where c = (a * b) % 2^256, and a0, b0, and c0 are least significant 32-bit limbs of a, b, and c respectively. |
