TARGETS := x86_64-unknown-linux-musl aarch64-unknown-linux-musl
NAME := els

.PHONY: release clean $(TARGETS)

release: clean $(TARGETS)

$(TARGETS):
	cross build --release --target $@
	mkdir -p dist
	cp target/$@/release/$(NAME) dist/$(NAME)-$@

clean:
	cargo clean
	rm -rf dist
