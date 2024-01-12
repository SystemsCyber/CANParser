default: ## Default command
	cargo build --release

all: check_cross ## Compile for multiple platforms
	# x86 windows
	cross build --release --target=i686-pc-windows-gnu
	# x86 linux
	cross build --release --target=i686-unknown-linux-gnu
	# raspberry pi or beaglebone
	cross build --release --target=armv7-unknown-linux-gnueabihf
	# mac
	cross build --release --target=x86_64-apple-darwin

check_cross: ## Check if 'cross' is installed, if not install it
	@if ! command -v cross &> /dev/null; then cargo install cross; fi

help: ## Show this help
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo ""
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
