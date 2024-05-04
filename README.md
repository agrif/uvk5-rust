Rust on the UV-K5
=================

Not much here, but to get started:

~~~~
# install tools and toolchain
pip install --upgrade --user svdtools
cargo install svd2rust@^0.33.1 form@^0.10.0 cargo-make@^0.37.12
rustup target add thumbv6m-none-eabi

# build tools, generate device crate, build firmware
( cd k5tool && cargo build )
cargo make --cwd dp32g030
( cd k5firmware && cargo build --release )

# flash to radio
( cd k5tool && cargo run -- flash ../k5firmware/target/thumbv6m-none-eabi/release/k5firmware )
~~~~

Still todo:

 * make sure none of this hoses everything forever.
 * it probably does.
