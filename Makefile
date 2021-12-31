.PHONY: archive install

install:
	cargo test --release
	cargo install --path gitext --force --locked
	install -m 755 $(wildcard gitext/wrappers/git-*) ~/.cargo/bin/

clean:
	cargo clean --release

build:
	cargo build --release

clean_archive:
	rm -rf archive

archive: build clean_archive
	mkdir archive
	cp -r gitext/wrappers archive/
	cp target/release/gitext install/Makefile archive
	pushd archive; tar -zcvf ../gitext.tgz .; popd
