1.Your hello_world function is synchronous (regular fn), but Axum handlers must be async. Axum's router uses async functions for handlers.

2. axum = "0.7"  This only enables Axum's default features. Rust crates can contain optional code.
Instead of compiling everything always,
Rust lets crates expose: features

These are like:

optional modules
capability switches
compile-time toggles

Why features exist

Without features:
every project would compile:

unnecessary code
extra dependencies
bigger binaries
slower builds

Rust prefers:

minimal compilation
explicit enabling