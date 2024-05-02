// This file is here so that cargo commands (like cargo metadata)
// still run, even if the src directory has not yet been generated. In
// particular, cargo make will try to run cargo metadata, but we need
// cargo make to work to generate src to begin with.
//
// https://github.com/sagiegurari/cargo-make/discussions/1076
fn main() {}
