## Field operations
Miden assembly provides a set of instructions which can perform operations with raw field elements. These instructions are described in the tables below.

While most operations place no restrictions on inputs, some operations expect inputs to be binary values, and fail if executed with non-binary inputs.

For instructions where one or more operands can be provided as immediate parameters (e.g., `add` and `add.b`), we provide stack transition diagrams only for the non-immediate version. For the immediate version, it can be assumed that the operand with the specified name is not present on the stack.

### Assertions and tests

| Instruction                     | Stack_input | Stack_output  | Notes                                                            |
| ------------------------------- | ----------- | ------------- | ---------------------------------------------------------------- |
| assert <br> - *(1 cycle)*       | [a, ...]    | [...]         | If $a = 1$, removes it from the stack. <br> Fails if $a \ne 1$   |
| assertz <br> - *(2 cycles)*     | [a, ...]    | [...]         | If $a = 0$, removes it from the stack, <br> Fails if $a \ne 0$   |
| assert_eq <br> - *(2 cycles)*   | [b, a, ...] | [...]         | If $a = b$, removes them from the stack. <br> Fails if $a \ne b$ |
| assert_eqw <br> - *(11 cycles)* | [B, A, ...] | [...]         | If $A = B$, removes them from the stack. <br> Fails if $A \ne B$ |


### Arithmetic and Boolean operations

| Instruction                                                                    | Stack_input | Stack_output  | Notes                                                                                                        |
| ------------------------------------------------------------------------------ | ----------- | ------------- | ------------------------------------------------------------------------------------------------------------ |
| add <br> - *(1 cycle)*  <br> add.*b* <br> - *(1-2 cycle)*                      | [b, a, ...] | [c, ...]      | $c \leftarrow (a + b) \mod p$                                                                                |
| sub <br> - *(2 cycles)*  <br> sub.*b* <br> - *(2 cycles)*                      | [b, a, ...] | [c, ...]      | $c \leftarrow (a - b) \mod p$                                                                                |
| mul <br> - *(1 cycle)*  <br> mul.*b* <br> - *(2 cycles)*                       | [b, a, ...] | [c, ...]      | $c \leftarrow (a \cdot b) \mod p$                                                                            |
| div <br> - *(2 cycles)*  <br> div.*b* <br> - *(2 cycles)*                      | [b, a, ...] | [c, ...]      | $c \leftarrow (a \cdot b^{-1}) \mod p$ <br> Fails if $b = 0$                                                 |
| neg <br> - *(1 cycle)*                                                         | [a, ...]    | [b, ...]      | $b \leftarrow -a \mod p$                                                                                     |
| inv <br> - *(1 cycle)*                                                         | [a, ...]    | [b, ...]      | $b \leftarrow a^{-1} \mod p$ <br> Fails if $a = 0$                                                           |
| pow2 <br> - *(16 cycles)*                                                      | [a, ...]    | [b, ...]      | $b \leftarrow 2^a$ <br> Fails if $a > 63$                                                                    |
| exp.*uxx* <br> - *(9 + xx cycles)*  <br> exp.*b* <br> - *(9 + log2(b) cycles)* | [b, a, ...] | [c, ...]      | $c \leftarrow a^b$ <br> Fails if xx is outside [0, 63) <br> exp is equivalent to exp.u64 and needs 73 cycles |
| not <br> - *(1 cycle)*                                                         | [a, ...]    | [b, ...]      | $b \leftarrow 1 - a$ <br> Fails if $a > 1$                                                                   |
| and <br> - *(1 cycle)*                                                         | [b, a, ...] | [c, ...]      | $c \leftarrow a \cdot b$ <br> Fails if $max(a, b) > 1$                                                       |
| or <br> - *(1 cycle)*                                                          | [b, a, ...] | [c, ...]      | $c \leftarrow a + b - a \cdot b$ <br> Fails if $max(a, b) > 1$                                               |
| xor <br> - *(7 cycles)*                                                        | [b, a, ...] | [c, ...]      | $c \leftarrow a + b - 2 \cdot a \cdot b$ <br> Fails if $max(a, b) > 1$                                       |

### Comparison operations

| Instruction                                                | Stack_input | Stack_output   | Notes                                                                                                                        |
| ---------------------------------------------------------- | ----------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| eq <br> - *(1 cycle)*  <br> eq.*b* <br> - *(1-2 cycles)*   | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a=b \\ 0, & \text{otherwise}\ \end{cases}$                                       |
| neq <br> - *(2 cycle)*  <br> neq.*b* <br> - *(2-3 cycles)* | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a \ne b \\ 0, & \text{otherwise}\ \end{cases}$                                   |
| lt <br> - *(17 cycles)*                                    | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a < b \\ 0, & \text{otherwise}\ \end{cases}$                                     |
| lte <br> - *(18 cycles)*                                   | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a \le b \\ 0, & \text{otherwise}\ \end{cases}$                                   |
| gt <br> - *(18 cycles)*                                    | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a > b \\ 0, & \text{otherwise}\ \end{cases}$                                     |
| gte <br> - *(19 cycles)*                                   | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a \ge b \\ 0, & \text{otherwise}\ \end{cases}$                                   |
| is_odd <br> - *(5 cycles)*                                 | [a, ...]    | [b, ...]       | $b \leftarrow \begin{cases} 1, & \text{if}\ a \text{ is odd} \\ 0, & \text{otherwise}\ \end{cases}$                          |
| eqw <br> - *(15 cycles)*                                   | [A, B, ...] | [c, A, B, ...] | $c \leftarrow \begin{cases} 1, & \text{if}\ a_i = b_i \; \forall i \in \{0, 1, 2, 3\} \\ 0, & \text{otherwise}\ \end{cases}$ |

### Extension Field Operations

| Instruction                        | Stack Input | Stack Output | Notes                                                        |
| ---------------------------------- | ----------- | ------------ | ------------------------------------------------------------ |
| ext2add <br> - *(5 cycles)*  <br>  | [b, a, ...] | [c, ...]     | $c \leftarrow (a + b) \mod q$                                |
| ext2sub <br> - *(7 cycles)*  <br>  | [b, a, ...] | [c, ...]     | $c \leftarrow (a - b) \mod q$                                |
| ext2mul <br> - *(3 cycles)*  <br>  | [b, a, ...] | [c, ...]     | $c \leftarrow (a \cdot b) \mod q$                            |
| ext2div <br> - *(11 cycles)*  <br> | [b, a, ...] | [c, ...]     | $c \leftarrow (a \cdot b^{-1}) \mod q$ <br> Fails if $b = 0$ |
| ext2neg <br> - *(4 cycles)*  <br>  | [a, ...]    | [b, ...]     | $b \leftarrow -a \mod q$                                     |
| ext2inv <br> - *(8 cycles)*  <br>  | [a, ...]    | [b, ...]     | $b \leftarrow a^{-1} \mod q$ <br> Fails if $a = 0$           |

where $q$ is an irreducible polynomial $x^2 - x + 2$ over $F_p$ for $p = 2^{64} - 2^{32} + 1$
