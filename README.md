# BASIC AF
## A (Dartmouth) BASIC to Brainf**k Compiler

This is a compiler from a variant of [Dartmouth BASIC](https://en.wikipedia.org/wiki/Dartmouth_BASIC) (see notes below) to [Brainf**k](https://en.wikipedia.org/wiki/Brainfuck). It was inspired by Peter Norvig's [BASIC interpreter "pytude"](https://github.com/norvig/pytudes/blob/master/ipynb/BASIC.ipynb) -- what easier way to interpret a language than first compiling it into an esoteric one?

The compiler is written in Rust, and uses [`nom` parser combinators](https://github.com/Geal/nom), the [`clap` command line option parser](https://clap.rs/), and Saghm Rossi's [`unescape` crate](https://github.com/saghm/unescape-rs).

To compile a BASIC program:
```bash
basicaf input.db
```

To execute a Brainf**k program:
```bash
basicaf -e program.bf
```

For more options, see:
```bash
basicaf --help
```

### The BASIC variant

```basic
5  REM Compute the first 20 Fibonacci numbers 
10 DIM F(20)
15 LET F(0) = 0
20 LET F(1) = 1
25 FOR I = 2 TO 20
30 LET F(I) = F(I-1) + F(I-2)
35 NEXT I
40 FOR X = 0 TO 20
45 PRINT "F(", X, ") = ", F(X), "\n"
50 NEXT X
55 END
```

```brainfuck
[-]>[-]<[>+<-][-]>[>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>+>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]<[>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[-]>[>>>[-<<<<+>>>>]<[->+<]<[->+<]<[->+<]>-]>>>[-]<[->+<]<[[-<+>]<<<[->>>>+<<<<]>>-]<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<[-]+>[-]<[>+<-][-]+>[>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>+>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]<[>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[-]>[>>>[-<<<<+>>>>]<[->+<]<[->+<]<[->+<]>-]>>>[-]<[->+<]<[[-<+>]<<<[->>>>+<<<<]>>-]<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<[-]++<[-]>[<+>-]>[-]>[-]<>+++++[<++++>-]<>[-]<<[-]>>>[-]>[-]<<<<<[>>>+<<+<-]>[<+>-][-]>>[<[<+>>>+<<-]>>[<<+>>-]+<<<[>>>-<<-<[-]]>>>[>+<[-]]<-]>>[<<+>>-]<<[<<+>>-]<[<+>-]<[>[-]>[-]<<<[>>+>+<<<-]>>>[<<<+>>>-][-]<[>+<-][-]>>[-]<<<<[>>+>>+<<<<-]>>>>[<<<<+>>>>-][-]+[<<->>-][-]<<[>>+<<-][-]>>[>>>>>>>>>>>>>>>>>>>>>>>>>>>+>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[-]<<<[-]>[>>>[-<<<<+>>>>]<<[->+<]<[->+<]>-]>>>[-<+<<+>>>]<<<[->>>+<<<]>[[-<+>]>[-<+>]<<<<[->>>>+<<<<]>>-]<<>>>[<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<+>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>-]<<<<<<<<<<<<<<<<<<<<<<<<<<<<<[-]>[-]<<<<<[>>>>+>+<<<<<-]>>>>>[<<<<<+>>>>>-][-]++[<->-][-]<[>+<-][-]>[>>>>>>>>>>>>>>>>>>>>>>>>>>+>+<<<<<<<<<<<<<<<<<<<<<<<<<<<-]>>>>>>>>>>>>>>>>>>>>>>>>>>>>[-]<<<[-]>[>>>[-<<<<+>>>>]<<[->+<]<[->+<]>-]>>>[-<+<<+>>>]<<<[->>>+<<<]>[[-<+>]>[-<+>]<<<<[->>>>+<<<<]>>-]<<>>>[<<<<<<<<<<<<<<<<<<<<<<<<<<<<<+>>>>>>>>>>>>>>>>>>>>>>>>>>>>>-]<<<<<<<<<<<<<<<<<<<<<<<<<<<<[-]<[<<+>>>+<-]>[<+>-]<<[>>>>>>>>>>>>>>>>>>>>>>>>>>>>+>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]<[>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]>>>>>>>>>>>>>>>>>>>>>>>>>>>>[-]>[>>>[-<<<<+>>>>]<[->+<]<[->+<]<[->+<]>-]>>>[-]<[->+<]<[[-<+>]<<<[->>>>+<<<<]>>-]<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<+>>[-]>[-]<>+++++[<++++>-]<>[-]<<[-]>>>[-]>[-]<<<<<[>>>+<<+<-]>[<+>-][-]>>[<[<+>>>+<<-]>>[<<+>>-]+<<<[>>>-<<-<[-]]>>>[>+<[-]]<-]>>[<<+>>-]<<[<<+>>-]<[<+>-]<]>[-]<[-]>[<+>-]>[-]>[-]<>+++++[<++++>-]<>[-]<<[-]>>>[-]>[-]<<<<<[>>>+<<+<-]>[<+>-][-]>>[<[<+>>>+<<-]>>[<<+>>-]+<<<[>>>-<<-<[-]]>>>[>+<[-]]<-]>>[<<+>>-]<<[<<+>>-]<[<+>-]<[>[-]++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.------------------------------.[-]>[-]<<<[>>+>+<<<-]>>>[<<<+>>>-][-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]<<<<<<<<<<<<<<<[>+<-]>[>>+>+<<<-]>>>[<<<+>>>-]<<+>[<->[>++++++++++<[->-[>+>>]>[+[-<+>]>+>>]<<<<<]>[-]++++++++[<++++++>-]>[<<+>>-]>[<<+>>-]<<]>]<[->>++++++++[<++++++>-]]<[.[-]<]<<[-]+++++++++++++++++++++++++++++++++++++++++.---------.+++++++++++++++++++++++++++++.-----------------------------.[-]>[-]<<<[>>+>+<<<-]>>>[<<<+>>>-][-]<[>+<-][-]>[>>>>>>>>>>>>>>>>>>>>>>>>>>>+>+<<<<<<<<<<<<<<<<<<<<<<<<<<<<-]>>>>>>>>>>>>>>>>>>>>>>>>>>>>>[-]<<<[-]>[>>>[-<<<<+>>>>]<<[->+<]<[->+<]>-]>>>[-<+<<+>>>]<<<[->>>+<<<]>[[-<+>]>[-<+>]<<<<[->>>>+<<<<]>>-]<<>>>[<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<+>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>-]<<<<<<<<<<<<<<<<<<<<<<<<<<<<<[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]<<<<<<<<<<<<<<<[>+<-]>[>>+>+<<<-]>>>[<<<+>>>-]<<+>[<->[>++++++++++<[->-[>+>>]>[+[-<+>]>+>>]<<<<<]>[-]++++++++[<++++++>-]>[<<+>>-]>[<<+>>-]<<]>]<[->>++++++++[<++++++>-]]<[.[-]<]<<[-]++++++++++.<<+>>[-]>[-]<>+++++[<++++>-]<>[-]<<[-]>>>[-]>[-]<<<<<[>>>+<<+<-]>[<+>-][-]>>[<[<+>>>+<<-]>>[<<+>>-]+<<<[>>>-<<-<[-]]>>>[>+<[-]]<-]>>[<<+>>-]<<[<<+>>-]<[<+>-]<]
```

This variant of BASIC is very similar to [Norvig's](https://github.com/norvig/pytudes/blob/master/ipynb/BASIC.ipynb) and the original, with a few notable exceptions:

* Multi-letter variable names are allowed. As a result, `Ai` is a variable, not the `i`th element of the `A` array. To index an array, use `A(i)`.
* Commas are required between elements of a `PRINT` statement
* The `PRINT` statement does not automatically append a newline
* Semicolons are *not* allowed at the end of non-comment lines
* Non-reducible control flow graphs are not supported (not a problem for most code)
* No support for floating point or negative values. As a result, `TAN`, `COS`, `SIN`, `ABS`, etc. are unavailable.

Other than that, all major features are supported, including multi-dimensional arrays, `GOTO`, `GOSUB`, etc.

### Examples

See the `examples/` directory for sample BASIC (`.db`) inputs and Brainf**k (`.bf`) outputs. You can run the Brainf**k output online [here](https://copy.sh/brainfuck), or using `basicaf -e`. Note that the last two examples require you to select the 32-bit cell size option.

* [`game_of_life.db`](https://raw.githubusercontent.com/RyanMarcus/basicaf/master/examples/game_of_life.db) [(output)](https://raw.githubusercontent.com/RyanMarcus/basicaf/master/examples/game_of_life.bf) prints 10 generations of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life).
* [`fib.db`](https://raw.githubusercontent.com/RyanMarcus/basicaf/master/examples/fib.db) [(output)](https://raw.githubusercontent.com/RyanMarcus/basicaf/master/examples/fib.bf) computes the first 20 Fibonacchi numbers.
* [`collatz.db`](https://raw.githubusercontent.com/RyanMarcus/basicaf/master/examples/collatz.db) [(output)](https://raw.githubusercontent.com/RyanMarcus/basicaf/master/examples/collatz.bf) prints the first 25 [hailstone sequences](https://en.wikipedia.org/wiki/Collatz_conjecture).
