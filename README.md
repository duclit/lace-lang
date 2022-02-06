Lace is an efficient, modern and predictable procedural programming language written in [rust](https://www.rust-lang.org/). It takes advantage of the compact and write-once-run-anywhere nature of bytecode, while eliminating an entire class of errors by using static typing (a dynamic type is still available, but it's use is discouraged unless required). 
* Easy to debug - Type-safety, null-safety, and prevention of unintended mutations help you write bug free code.
* Blazingly fast - Static-typing and a rust-like GC allows lace to be faster than most interpreted languages.

> This project is under development, and the above points might not be true yet.

## Optimizations
Lace has three main features that make it fast:
* Static Typing
* Rust-like GC

### Static Typing
Lace requires programmers to specify the type of each variable, and if the type is not `any`, the compiler can perform compile-time optimizations to make any operations (addition, multiplication, etc) on that variable faster.

### Rust-like GC
Lace uses a rust-like garbage collector, where all variables in a function are freed when it returns. Since garbage collection does not happen inside functions, a `free` keyword is available to free variables. Lace also provides a `collect` macro, which expands to a bunch of free statements that free any variables that are not used after the macro call.
