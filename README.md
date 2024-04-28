Rust on the UV-K5
=================

Not much here, but to get started:

~~~~
# install tools and toolchain
pip install --upgrade --user svdtools
cargo install svd2rust form
rustup target add thumbv6m-none-eabi

# build tools, generate device crate, build firmware
( cd k5tool && cargo build )
make -C dp32g030
( cd k5firmware && cargo build --release )
( cd k5tool && cargo run -- pack ../k5firmware/target/thumbv6m-none-eabi/release/k5firmware ../k5firmware.packed )
~~~~

Still todo:

 * make sure none of this hoses everything forever.
