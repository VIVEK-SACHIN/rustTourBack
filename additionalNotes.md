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


3.Think like this:
JS:
filesystem = module system
Rust:
filesystem helps organize
BUT compiler uses declared module tree so mod.rs in every folder is required 
Why Rust designed this way?

Main reasons:

explicitness
compile-time safety
faster compilation reasoning
avoids accidental imports
clearer visibility rules