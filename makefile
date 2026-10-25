FIRMWARE = -p presence-firmware --target riscv32imc-unknown-none-elf

# Which board to flash comes from .env (PRESENCE_PROBE, the VID:PID:serial
# from `probe-rs list`). The serial is the board's MAC, so it lives there;
# with one board plugged in it can stay empty.
PROBE = $(shell grep '^PRESENCE_PROBE=' .env 2>/dev/null | cut -d= -f2-)
PROBE_ENV = $(if $(PROBE),PROBE_RS_PROBE=$(PROBE))

# plain `make` lists the targets rather than flashing a board
help:
	@echo "  make run     flash the board and stream its logs (Ctrl-C to stop)"
	@echo "  make test    core tests on the Mac"
	@echo "  make check   clippy and fmt"

run:
	$(PROBE_ENV) cargo run $(FIRMWARE)

test:
	cargo test -p presence-core

check:
	cargo clippy -p presence-core --all-targets -- -D warnings
	cargo clippy -p presence-core --target riscv32imc-unknown-none-elf -- -D warnings
	cargo clippy $(FIRMWARE) -- -D warnings
	cargo fmt --check
