# `IMP_sm` Interpreter

This is an interpreter for the `IMP_sm` language presented in the paper "Memory
Safety Isn’t All You Need: Code Reuse Attacks in Memory Safe Languages".

The interpreter is written in Rust and built using Cargo, which can be obtained
from <https://rust-lang.org/learn/get-started/>. Alternatively, you can use the
`Dockerfile` to create a container to build and run the interpreter:

```shell
docker build -t interpreter .
docker run -it --rm -p 8000:8000 -v .:/interpreter -w /interpreter interpreter
```

## Usage & Features

The interpreter implements the full syntax and operational semantics, including
observations and traces, and pretty printing of intermediate configurations and
final states. To run the interpreter:

```shell
cargo run
# or
cargo build
./target/debug/ism
```

which will output a usage message. To test if the interpreter is working
correctly, try

```shell
cargo run -- debug ./programs/example.ism
```

which will output the full execution trace of [`example.ism`] ending with

```text
[...]
Result:
4
```

The interpreter has three modes:

1. `compare`: Run the program on two sets of inputs and output a comparison of
   the final states, traces, and results.
1. `run`: Run the program on a single input set and output the final state,
   trace, and result.
1. `debug`: Run the program on a single input set, outputting all intermediate
   configurations as well as the final state, trace, and result.

[`example.ism`]: ./programs/example.ism

### Syntax

The syntax used by the interpreter is largely identical to the syntax presented
in the paper. There are, however, a few small differences due to the use of
characters that are nice when typeset but not easy to type. The differences are
listed below. See [`grammar.txt`] for full details.

| Paper | Interpreter |
| ----- | ----------- |
| `←`   | `<-`        |
| `⊥`   | `nil`       |
| `∧`   | `&&`        |
| `=`   | `==`        |
| `≠`   | `!=`        |

[`grammar.txt`]: ./grammar.txt

## Examples

The [`programs/`] directory contains runnable versions of all listings in the
paper as well as all canonical weakness patterns and gadget patterns from
Section 3 of the paper.

The examples can be run manually by following the instructions above or on
pre-specified inputs using `make` commands. Run just `make` to get a list of
available commands where the `demo_*` commands run the examples.

[`programs/`]: ./programs

## Peculiarities

- The parser requires spaces between all tokens.

  E.g., `x:=y` is rejected and must be written as `x := y`.

- Compound expressions must use parentheses.

  E.g., `a == b && c == d` cannot be parsed and must be written as
  `(a == b) && (c == d)`.

- The parser accepts a trailing `;` and a missing `;` in the middle of the
  program may be treated as the end of the program.

- The interpreter approximates the bijection relating heap locations between
  executions based on allocation order.

## Experimental Features

There are two experimental features:

1. A web-based version.

   An incomplete version of the interpreter that can be run in a browser is
   available. It only supports running programs along with a primitive
   visualization of configurations.

   Use `make web` to build the web version and start a local server hosting it.
   Open <http://localhost:8000/> in your browser to try it out.

1. Diagram of final state.

   Executing a program with the interpreter will produce `.mermaid` files with
   a diagram of the final state. This visualizes the store, heap, context, and
   observations. The color red is used to highlight shared memory.

   These diagrams can be viewed with, e.g., <https://mermaid-viewer.com/>.

## License

This software is available under the Apache License 2.0; see [LICENSE] for the
full license text.

[LICENSE]: ./LICENSE
