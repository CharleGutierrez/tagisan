# sec-kernel PoC Makefile
#
# Usage:
#   make              # Build all CVE PoCs
#   make cve_2016_5195  # Build single CVE PoC
#   make clean        # Clean all binaries
#   make magic        # Replace ELF magic (\x7fELF -> CVE\x00)
#
# Requirements:
#   - gcc with static linking support
#   - Common headers in common/
#   - Output to ../poc-bin/

.PHONY: all clean magic help

# Directories
POC_SRC_DIR := $(dir $(lastword $(MAKEFILE_LIST)))
POC_BIN_DIR := $(POC_SRC_DIR)../poc-bin/
COMMON_DIR := $(POC_SRC_DIR)common/

# Compiler settings
# MANDATORY: Static linking for cross-glibc compatibility
# PoC binaries must run on any Linux server regardless of glibc version
CC := gcc
CFLAGS := -O2 -Wall -I$(COMMON_DIR) -DPOC_NO_SYSTEM_MODIFY=1
LDFLAGS := -static

# Strip command
STRIP := strip

# Find all CVE directories (only those with poc.c source)
CVE_DIRS := $(wildcard $(POC_SRC_DIR)cve_*)
CVE_IDS := $(foreach dir,$(CVE_DIRS),$(if $(wildcard $(dir)/poc.c),$(notdir $(dir))))

# Binary targets
BINARIES := $(foreach cve,$(CVE_IDS),$(POC_BIN_DIR)$(cve).bin)

# Default target
all: clean_old binaries
	@echo "[DONE] Built $(words $(BINARIES)) PoC binary(ies)"

# Build all binaries
binaries: $(BINARIES)

# Build single CVE: auto-detect pthread requirement
# Files using pthread.h need -lpthread for static linking
# Note: -lpthread must come AFTER source file for static linking
$(POC_BIN_DIR)%.bin: $(POC_SRC_DIR)%/poc.c
	@mkdir -p $(POC_BIN_DIR)
	@echo "[INFO] Compiling: $(notdir $*) -> $(notdir $@)"
	@if grep -q 'pthread' $<; then \
		echo "[INFO] pthread detected, adding -lpthread"; \
		$(CC) $(CFLAGS) $< $(LDFLAGS) -lpthread -o $(POC_BIN_DIR)$(notdir $*).elf; \
	else \
		$(CC) $(CFLAGS) $< $(LDFLAGS) -o $(POC_BIN_DIR)$(notdir $*).elf; \
	fi
	@$(STRIP) --strip-all $(POC_BIN_DIR)$(notdir $*).elf
	@echo "[INFO] Masking ELF magic: \\x7fELF -> CVE\\x00"
	@python3 -c "import sys; data = open('$(POC_BIN_DIR)$(notdir $*).elf', 'rb').read(); data = b'CVE\x00' + data[4:]; open('$(POC_BIN_DIR)$(notdir $*).bin', 'wb').write(data)"
	@rm -f $(POC_BIN_DIR)$(notdir $*).elf
	@chmod -x $(POC_BIN_DIR)$(notdir $*).bin
	@echo "[PASS] Created: $(notdir $@)"

# Clean old docker/k8s binaries (container escape removed)
clean_old:
	@echo "[INFO] Cleaning old docker.bin and k8s.bin (container escape removed)..."
	@rm -f $(POC_BIN_DIR)*-docker.bin
	@rm -f $(POC_BIN_DIR)*-k8s.bin
	@echo "[CLEAN] Old container escape binaries removed"

# Clean all binaries
clean: clean_old
	@echo "[INFO] Cleaning all PoC binaries..."
	@rm -f $(POC_BIN_DIR)*.bin
	@rm -f $(POC_BIN_DIR)*.elf
	@echo "[CLEAN] All binaries removed"

# Replace ELF magic for all binaries
magic:
	@echo "[INFO] Masking ELF magic for all binaries..."
	@for bin in $(POC_BIN_DIR)*.bin; do \
		if [ -f "$$bin" ]; then \
			echo "  Masking: $$(basename $$bin)"; \
			python3 -c "import sys; data = open('$$bin', 'rb').read(); data = b'CVE\x00' + data[4:]; open('$$bin', 'wb').write(data)"; \
		fi; \
	done
	@echo "[PASS] ELF magic masked"

# Help
help:
	@echo "sec-kernel PoC Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make              Build all CVE PoCs"
	@echo "  make <cve_id>     Build single CVE PoC (e.g., make cve_2016_5195)"
	@echo "  make clean        Clean all binaries"
	@echo "  make magic        Mask ELF magic (\\x7fELF -> CVE\\x00)"
	@echo ""
	@echo "Targets:"
	@echo "  CVE IDs: $(CVE_IDS)"
	@echo ""
	@echo "Output:"
	@echo "  $(POC_BIN_DIR)"

# Individual CVE targets
$(CVE_IDS):
	@$(MAKE) $(POC_BIN_DIR)$@.bin
