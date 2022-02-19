# Ideas

Simple language, with simple user-facing semantics.

Compiler is exposed as library, and can by imported in the build script to
accomplish much more complex stuffs. Default mode is to interpret the given
file, so that this stuff is easier to do.

Make compiler nice to work with as a library, so we can run experiments on what
kinds of patterns are helpful and what arent.

# Include in base language
- Primitives, pointers, slices
- Functions
- Control flow
- Type id's, runtime type information
- Dynamic memory
- Iterators (as macros? or as generators that behave like macros?)
- Scope begin/end directives, scoped feature enable/disable
- Simple compile-time constants (i.e. literals)
- Structs structs
- C ABI
- Simple enums
- Basic type inference
- Function pointers
- Allocators
- Implicit context
- defer, named continue/break
- Some kind of "throw error" thing, but not using stack unwinding
- Nullable checks: `a ?? b`, `a?.b`, `a?(`, `a?[`, etc.
- Some kind of "pass up this if it throws" thing, i.e. `could_error()!`

# IDK Yet
- Macros
- Generics
- Closures
- Complex enums
- Non-nullable types
- Overloading
- Operator overloading
- Interfaces/traits/etc.
- Anonymous structs
- Contracts/type requirements
- Compile-time constant evaluation
- Tuples
- async-await
- Modify AST
- Custom typechecking
- Inheritance using explicit type field? Call it closed and it can become an enum?

# Too Complex, use compiler API
- Python ABI
- Correctness checking
- Generate fuzzers, test harnesses
- Insert source code as string or nodes

# Too Complex, you're on your own
- Inheritance
- Garbage collection
- RAII
