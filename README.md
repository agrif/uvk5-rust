Rust on the UV-K5
=================

Not much here, but to get started:

~~~~
# install tools and toolchain
pip install --upgrade --user svdtools
cargo install svd2rust form cargo-binutils
rustup target add thumbv6m-none-eabi
rustup component add llvm-tools

# build tools, generate device crate, build firmware
( cd k5tool && cargo build )
make -C dp32g030
( cd k5firmware && cargo build --release )
rust-objcopy -O binary k5firmware/target/thumbv6m-none-eabi/release/k5firmware k5firmware.bin
( cd k5tool && cargo run -- pack '*0.0test' ../k5firmware.bin ../k5firmware.packed )
~~~~

Still todo:

 * make sure none of this hoses everything forever.
