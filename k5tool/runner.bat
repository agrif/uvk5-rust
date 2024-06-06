:; # -*- mode: sh -*-
:;
:; # This file is a POSIX sh and Windows CMD script, in one.
:; # It's job is to run `cargo run -- rest of arguments` in this directory,
:; # to be used as a cargo runner, because cargo -C is unstable.
:;
:; # Nothing valued is here, what is here is dangerous and repulsive to us.
:;
:; cd -P -- "$(dirname -- "$0")"
:; cargo run -- "$@"; exit $?

@ECHO OFF
pushd "%~dp0"
cargo run -- %*
popd
