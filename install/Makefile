WRAPPERS=$(wildcard wrappers/git-*)
install:
	install -m 755 -C gitbucket $(WRAPPERS) /usr/local/bin
	xattr -d com.apple.quarantine /usr/local/bin/gitbucket
