.PHONY : all clean

# python version is more lax than rust version
# which we need because our vendor svd is JANK
SVDTOOLS=svd
CARGO=cargo
SVD2RUST=svd2rust
FORM=form

SVDFILE=svd/dp32g030.vendor.svd

all : src/lib.rs svd/memorymap.txt

clean :
	rm -f svd/dp32g030.vendor.svd.patched svd/memorymap.txt
	rm -f lib.rs build.rs device.x
	rm -rf src/

${SVDFILE}.patched : ${SVDFILE} svd/*.yaml
	${SVDTOOLS} patch svd/main.yaml

svd/memorymap.txt : ${SVDFILE}.patched
	${SVDTOOLS} mmap ${SVDFILE}.patched > svd/memorymap.txt

src/lib.rs : ${SVDFILE}.patched
	${SVD2RUST} -c svd2rust.toml -i ${SVDFILE}.patched
	rm -rf src
	${FORM} -i lib.rs -o src/
	rm lib.rs
	${CARGO} fmt -- --config-path="rustfmt.toml"
