# Makefile for CH32H417EVT dual-core Zephyr application
#
# Build both V3F waker and V5F application images, merge them into a single
# binary, and flash the board.
#
# Requirements:
#   - Zephyr workspace at ~/zephyrproject (override with ZEPHYR_ROOT=...)
#   - west inside ~/zephyrproject/.venv (override with WEST=...)
#   - WCH-LinkE debug probe
#   - On macOS: wlink (recommended) or OpenOCD with WCH support
#   - screen / minicom for the serial terminal

ZEPHYR_ROOT ?= $(HOME)/zephyrproject
WEST        ?= $(ZEPHYR_ROOT)/.venv/bin/west

BOARD_V3F := ch32h417evt/ch32h417/v3f
BOARD_V5F := ch32h417evt/ch32h417/v5f

BUILD_V3F := $(abspath build/v3f)
BUILD_V5F := $(abspath build/v5f)

SRC_V3F := $(abspath v3f)
SRC_V5F := $(abspath v5f)

# Vendored zephyr-lang-rust module providing Rust language support (CONFIG_RUST,
# rust_cargo_application) for the V5F application.
RUST_MODULE := $(abspath third_party/zephyr-lang-rust)

OPENOCD_CFG ?= $(ZEPHYR_ROOT)/zephyr/boards/wch/ch32h417evt/support/openocd.cfg
SERIAL_PORT ?= /dev/ttyACM0

MERGED_BIN := ch32h417_dual.bin

# Auto-select flash tool: wlink on macOS, OpenOCD elsewhere.
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
  FLASH_TOOL ?= wlink
else
  FLASH_TOOL ?= openocd
endif

.PHONY: all help build build-v3f build-v5f merge flash flash-wlink flash-openocd serial clean

all: build merge

help:
	@echo "Targets:"
	@echo "  build-v3f     Build the V3F waker image"
	@echo "  build-v5f     Build the V5F application image"
	@echo "  build         Build both images"
	@echo "  merge         Merge V3F + V5F binaries into $(MERGED_BIN)"
	@echo "  flash         Build, merge and flash (auto: wlink on macOS, OpenOCD elsewhere)"
	@echo "  flash-wlink   Build, merge and flash using wlink"
	@echo "  flash-openocd Build, merge and flash using OpenOCD"
	@echo "  serial        Open serial terminal on $(SERIAL_PORT) @ 115200"
	@echo "  clean         Remove build outputs and merged binary"
	@echo ""
	@echo "Variables:"
	@echo "  ZEPHYR_ROOT=<path>   (current: $(ZEPHYR_ROOT))"
	@echo "  WEST=<path>          (current: $(WEST))"
	@echo "  FLASH_TOOL=wlink|openocd   (current: $(FLASH_TOOL))"
	@echo "  SERIAL_PORT=<port>"

build-v3f:
	cd $(ZEPHYR_ROOT) && $(WEST) build -p always -b $(BOARD_V3F) -d $(BUILD_V3F) $(SRC_V3F)

build-v5f:
	cd $(ZEPHYR_ROOT) && LIBCLANG_PATH=$${LIBCLANG_PATH:-/opt/homebrew/opt/llvm/lib} \
		$(WEST) build -p always -b $(BOARD_V5F) -d $(BUILD_V5F) $(SRC_V5F) \
		-- -DZEPHYR_EXTRA_MODULES=$(RUST_MODULE)

build: build-v3f build-v5f

merge: build
	python3 scripts/merge_dual.py \
		$(BUILD_V3F)/zephyr/zephyr.bin \
		$(BUILD_V5F)/zephyr/zephyr.bin \
		$(MERGED_BIN)

flash: merge
ifeq ($(FLASH_TOOL),wlink)
	$(MAKE) flash-wlink
else
	$(MAKE) flash-openocd
endif

flash-wlink: merge
	wlink flash --address 0x08000000 $(MERGED_BIN)

flash-openocd: merge
	openocd -f $(OPENOCD_CFG) \
		-c init \
		-c halt \
		-c "program $(MERGED_BIN) 0x00000000 verify" \
		-c reset \
		-c shutdown

serial:
	screen $(SERIAL_PORT) 115200

clean:
	rm -rf build $(MERGED_BIN)
