install:
	# cargo test --release
	cargo install --path gitbucket --force
	cp -n gitbucket/wrappers/* ~/.cargo/bin/ 2>/dev/null || :
