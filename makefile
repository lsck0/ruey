.DEFAULT_GOAL := default
.PHONY: default
default:
	@echo "Available tasks:"
	@echo "  - make run"
	@echo "  - make build"
	@echo "  - make ci"

.PHONY: run
run:
	cargo run

.PHONY: build
build:
	rm -rf ./dist/
	mkdir -p dist

	cargo build --release --target x86_64-unknown-linux-gnu
	cargo build --release --target x86_64-pc-windows-gnu

	cp ./target/x86_64-unknown-linux-gnu/release/ruey ./dist/ruey-x86_64_linux
	cp ./target/x86_64-pc-windows-gnu/release/ruey.exe ./dist/ruey-x86_64_windows-unsigned.exe

	# even if this is just self-singend, without signature browsers will just block the download and call it malware
	osslsigncode sign \
		-pkcs12 ./assets/ruey-cert.pfx \
		-pass "super_secure_password_for_code_signing" \
		-t http://timestamp.digicert.com \
		-h sha256 \
		-in ./dist/ruey-x86_64_windows-unsigned.exe \
		-out ./dist/ruey-x86_64_windows.exe \

	rm ./dist/ruey-x86_64_windows-unsigned.exe

.PHONY: ci
ci:

.PHONY: ci
ci:
	cargo deny check \
		--allow unlicensed \
		--allow license-not-encountered \
		--allow duplicate \
		--allow unmaintained \
