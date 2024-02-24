.PHONY: archive install

install:
	cargo test --release
	cargo install --path . --force --locked
	cargo clean --release

clean:
	cargo clean --release

build:
	cargo build --release

clean_archive:
	rm -rf gitext.zip
	rm -rf archive

archive: build clean_archive clean
	mkdir archive
	cp target/release/gitext install/Makefile archive
	codesign -s 4CF30DCF7945FCED0B59096BA61A2DD0364AA19F -o runtime -v archive/gitext
	pushd archive; zip -r ../gitext.zip .; popd

notarize: archive
	xcrun notarytool submit gitext.zip --keychain-profile "Burdukov"
