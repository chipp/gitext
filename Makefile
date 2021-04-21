.PHONY: archive install

install:
	cargo test --release
	cargo install --path gitbucket --force --locked
	install -m 755 $(wildcard gitbucket/wrappers/git-*) ~/.cargo/bin/

clean:
	cargo clean --release

build:
	cargo build --release

clean_archive:
	rm -rf archive

archive: clean build clean_archive
	mkdir archive
	cp -r gitbucket/wrappers archive/
	cp target/release/gitbucket install/Makefile archive
	pushd archive; tar -zcvf ../gitbucket.tgz .; popd
