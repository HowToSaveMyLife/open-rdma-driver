CC = gcc
CFLAGS = -Wall -Wextra -g
LDFLAGS =
LIBS = -libverbs

# Support for additional compiler flags
ifneq ($(EXTRA_CFLAGS),)
CFLAGS += $(EXTRA_CFLAGS)
endif

# Directories
LIB_DIR = lib
TEST_DIR = tests
LEGACY_DIR = legacy
BUILD_DIR = build
OBJ_DIR = $(BUILD_DIR)/obj
BIN_DIR = $(BUILD_DIR)/bin

# Library objects
LIB_OBJS = $(OBJ_DIR)/rdma_common.o \
           $(OBJ_DIR)/rdma_transport.o \
           $(OBJ_DIR)/rdma_debug.o

# Auto-discover test sources
TEST_SRCS = $(wildcard $(TEST_DIR)/*.c)
TEST_BINS = $(patsubst $(TEST_DIR)/%.c,$(BIN_DIR)/%,$(TEST_SRCS))

LEGACY_SRCS = $(wildcard $(LEGACY_DIR)/*.c)
LEGACY_BINS = $(patsubst $(LEGACY_DIR)/%.c,$(BIN_DIR)/legacy_%,$(LEGACY_SRCS))

# Default: build new tests
all: $(BIN_DIR) $(OBJ_DIR) $(LIB_OBJS) $(TEST_BINS)

# Build all (including legacy)
full: all $(LEGACY_BINS)

# Build only legacy tests
legacy: $(BIN_DIR) $(LEGACY_BINS)

# Create directories
$(BIN_DIR) $(OBJ_DIR):
	mkdir -p $@

# Compile library objects
$(OBJ_DIR)/%.o: $(LIB_DIR)/%.c $(LIB_DIR)/%.h | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# Compile new tests (with library)
$(BIN_DIR)/%: $(TEST_DIR)/%.c $(LIB_OBJS) | $(BIN_DIR)
	$(CC) $(CFLAGS) $< $(LIB_OBJS) -o $@ $(LDFLAGS) $(LIBS)

# Compile legacy tests (standalone)
$(BIN_DIR)/legacy_%: $(LEGACY_DIR)/%.c | $(BIN_DIR)
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS) $(LIBS)

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)

# Run test examples
run-loopback:
	$(BIN_DIR)/loopback 4096 5

run-send-recv-server:
	$(BIN_DIR)/send_recv 4096

run-send-recv-client:
	$(BIN_DIR)/send_recv 4096 127.0.0.1

# Hardware test with PCI device reset (legacy loopback)
run-loopback-hardware: $(BIN_DIR)/legacy_loopback
	@echo "Running hardware test with PCI device reset..."
	echo 1 > /sys/bus/pci/devices/0000\:01\:00.0/remove ; \
	echo 1 > /sys/bus/pci/rescan ; \
	setpci  -s 01:00.0 COMMAND=0x02 && \
	setpci  -s 01:00.0 98.b=0x16 && \
	setpci  -s 01:00.0 CAP_EXP+28.w=0x1000 && \
	export LD_LIBRARY_PATH=../dtld-ibverbs/target/debug:../dtld-ibverbs/rdma-core-55.0/build/lib && \
	RUST_LOG=debug $(BIN_DIR)/legacy_loopback 1

# List available tests
list:
	@echo "========== Available Tests =========="
	@echo ""
	@echo "New tests (using framework):"
	@if [ -d "$(BIN_DIR)" ]; then \
		for f in $(BIN_DIR)/*; do \
			[ -f "$$f" ] && ! echo "$$f" | grep -q "legacy_" && echo "  $$(basename $$f)"; \
		done | sort || echo "  (none built)"; \
	else \
		echo "  (none built)"; \
	fi
	@echo ""
	@echo "Legacy tests (original versions):"
	@if [ -d "$(BIN_DIR)" ]; then \
		for f in $(BIN_DIR)/legacy_*; do \
			[ -f "$$f" ] && echo "  $$(basename $$f)"; \
		done | sort || echo "  (none built)"; \
	else \
		echo "  (none built)"; \
	fi
	@echo ""
	@echo "====================================="

# Show directory structure
tree:
	@echo "Directory structure:"
	@tree -L 2 -I 'build' || (echo "lib/"; ls -1 lib/; echo "tests/"; ls -1 tests/; echo "legacy/"; ls -1 legacy/)

# Help target
help:
	@echo "Base Test Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make all          - Build new tests (default)"
	@echo "  make full         - Build all tests (new + legacy)"
	@echo "  make legacy       - Build only legacy tests"
	@echo "  make clean        - Remove build artifacts"
	@echo "  make list         - List available test binaries"
	@echo "  make tree         - Show directory structure"
	@echo "  make help         - Show this help"
	@echo ""
	@echo "Run targets:"
	@echo "  make run-loopback              - Run loopback test (new version)"
	@echo "  make run-loopback-hardware     - Run loopback with PCI reset (hardware test)"
	@echo "  make run-send-recv-server      - Run send/recv server"
	@echo "  make run-send-recv-client      - Run send/recv client"
	@echo ""
	@echo "Directory structure:"
	@echo "  lib/     - Common library code (rdma_common, rdma_transport, rdma_debug)"
	@echo "  tests/   - New test cases using the framework"
	@echo "  legacy/  - Original test implementations (for comparison)"
	@echo "  build/   - Build artifacts (obj/, bin/)"

.PHONY: all full legacy clean run-loopback run-loopback-hardware run-send-recv-server run-send-recv-client list tree help
