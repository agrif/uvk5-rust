.PHONY : all clean

# python version is more lax than rust version
# which we need because our vendor svd is JANK
SVDTOOLS=svd
CARGO=cargo
SVD2RUST=svd2rust
FORM=form

all : src/lib.rs svd/memorymap.txt

clean :
	rm -f svd/dp32g030.svd svd/dp32g030.vendor.svd.patched svd/memorymap.txt
	rm -f lib.rs build.rs device.x
	rm -rf src/

svd/dp32g030.svd : svd/dp32g030.vendor.svd svd/*.yaml
	${SVDTOOLS} patch svd/main.yaml
	mv svd/dp32g030.vendor.svd.patched svd/dp32g030.svd

svd/memorymap.txt : svd/dp32g030.svd
	${SVDTOOLS} mmap svd/dp32g030.svd > svd/memorymap.txt

src/lib.rs : svd/dp32g030.svd
	${SVD2RUST} -i svd/dp32g030.svd
	rm -rf src
	${FORM} -i lib.rs -o src/
	rm lib.rs
	${CARGO} fmt
