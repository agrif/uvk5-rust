.PHONY : all clean

# python version is more lax than rust version
# which we need because our vendor svd is JANK
SVDTOOL=svd

all : src/lib.rs

clean :
	rm -f svd/dp32g030.svd
	rm -f lib.rs build.rs device.x
	rm -rf src/

svd/dp32g030.svd : svd/dp32g030.vendor.svd svd/dp32g030.yaml
	svd patch svd/dp32g030.yaml
	mv svd/dp32g030.vendor.svd.patched svd/dp32g030.svd

src/lib.rs : svd/dp32g030.svd
	svd2rust -i svd/dp32g030.svd
	rm -rf src
	form -i lib.rs -o src/
	rm lib.rs
	cargo fmt
