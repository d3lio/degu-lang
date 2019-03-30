# degu-lang

A take at a scripting language with an ML (Meta Language) family syntax. Heavily inspired by F#, OCaml and at some extent - Rust.

### Setting up the project

You need to have a local LLVM sdk. [The llvm cmake guide](https://llvm.org/docs/CMake.html) will help you as well as [the llvm-sys crate](https://crates.io/crates/llvm-sys) since that is what the project depends on.

What I've found useful is to pass a parallel build flag to `cmake --build`. For Windows' MSVC this is `/maxcpucount:<n>` and would look something like `cmake --build . --config MinSizeRel -- /maxcpucount:4`.

The current required version of LLVM is 6 which will change when I get to recompile LLVM again.

Other than just run `cargo run -p compiler`.

### Example output

main.dg
```f#
let number_ops a b c =
    print_number a
    print_number b
    print_number c
    a - 2 + b - 3 + c * 10 + (a < b)

let main _ = print_number (number_ops 1 2 3)
```

stdout
```
1
2
3
29
```
