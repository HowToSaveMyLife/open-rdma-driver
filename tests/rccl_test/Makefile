# RCCL Test Makefile for ROCm and Blue RDMA

# Compiler settings
CXX = mpicxx
HIPCC = hipcc

# ROCm paths
ROCM_HOME ?= /opt/rocm
DTK_HOME ?= /opt/dtk

# RCCL paths
RCCL_INCLUDE ?= $(DTK_HOME)/include
RCCL_LIB ?= $(DTK_HOME)/lib

# MPI paths
MPI_HOME = /usr/mpi/gcc/openmpi-4.1.7rc1
MPI_INCLUDE = $(MPI_HOME)/include
MPI_LIB = $(MPI_HOME)/lib

# Build directory
BUILD_DIR = build

# Compiler flags
CXXFLAGS = -std=c++11 -O2
HIPCCFLAGS = -std=c++11 -O2

# Include paths
INCLUDES = -I$(RCCL_INCLUDE) \
           -I$(MPI_INCLUDE) \
           -I$(ROCM_HOME)/include

# Library paths
LDFLAGS = -L$(RCCL_LIB) \
          -L$(MPI_LIB) \
          -L$(ROCM_HOME)/lib

# Libraries to link
LIBS = -lrccl -lmpi

# Targets
SIMPLE_TARGET = $(BUILD_DIR)/simple_test
SIMPLE_SOURCES = simple_test.cpp
NOMPI_TARGET = $(BUILD_DIR)/normal_test_nompi
NOMPI_SOURCES = normal_test_nompi.cpp

# hack lib
PWD=$(shell pwd)

HACK_LD_LIB=$(PWD)/../hack_libc/target/debug/libhack_libc.so

.PHONY: all clean rebuild info help simple nompi simple_rdma simple_rdma_force nompi_rank0 nompi_rank1 nompi_hack_rank0 nompi_hack_rank1

all: $(BUILD_DIR) $(SIMPLE_TARGET) $(NOMPI_TARGET)

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Compile the simple test program
$(SIMPLE_TARGET): $(SIMPLE_SOURCES) | $(BUILD_DIR)
	@echo "Compiling RCCL simple test program..."
	$(HIPCC) $(HIPCCFLAGS) $(INCLUDES) -o $(SIMPLE_TARGET) $(SIMPLE_SOURCES) $(LDFLAGS) $(LIBS)
	@echo "Build complete: $(SIMPLE_TARGET)"

# Compile the normal test program without MPI
$(NOMPI_TARGET): $(NOMPI_SOURCES) | $(BUILD_DIR)
	@echo "Compiling RCCL normal test program without MPI..."
	$(HIPCC) $(HIPCCFLAGS) -I$(RCCL_INCLUDE) -I$(ROCM_HOME)/include -o $(NOMPI_TARGET) $(NOMPI_SOURCES) -L$(RCCL_LIB) -L$(ROCM_HOME)/lib -lrccl
	@echo "Build complete: $(NOMPI_TARGET)"


test_alloc: test_alloc.cpp | $(BUILD_DIR)
	$(HIPCC) $(HIPCCFLAGS) -I$(RCCL_INCLUDE) -I$(ROCM_HOME)/include -o build/test_alloc test_alloc.cpp -L$(RCCL_LIB) -L$(ROCM_HOME)/lib -lrccl


# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)
	@echo "Clean complete"

# Rebuild everything
rebuild: clean all

# Run simple_test with RDMA (2 DCUs)
simple_rdma: $(SIMPLE_TARGET)
	@echo "=========================================="
	@echo "  Running Simple Test with RDMA (2 DCUs)"
	@echo "=========================================="
	@echo "Rank 0 -> blue0, Rank 1 -> blue1"
	@echo ""
	export PATH=$(MPI_HOME)/bin:$$PATH && \
	export LD_LIBRARY_PATH=$(MPI_LIB):$$LD_LIBRARY_PATH && \
	export NCCL_IB_DISABLE=0 && \
	export NCCL_NET_GDR_LEVEL=0 && \
	mpirun -np 2 \
	  -x UCX_NET_DEVICES=blue0,blue1 \
	  -x NCCL_IB_DISABLE=0 \
	  -x RUST_LOG=debug \
	  -mca coll ^hcoll \
	  $(SIMPLE_TARGET)

# Run simple_test with RDMA forcing network transport
simple_rdma_force: $(SIMPLE_TARGET)
	@echo "=========================================="
	@echo "  Simple Test - Force IB Network"
	@echo "=========================================="
	@echo "Disabling P2P and SHM to force IB network usage"
	@echo ""
	export PATH=$(MPI_HOME)/bin:$$PATH && \
	export LD_LIBRARY_PATH=$(MPI_LIB):$$LD_LIBRARY_PATH && \
	export NCCL_DEBUG=INFO && \
	export NCCL_IB_DISABLE=0 && \
	export NCCL_P2P_DISABLE=1 && \
	export NCCL_SHM_DISABLE=1 && \
	export NCCL_NET_GDR_LEVEL=0 && \
	export NCCL_IB_HCA=bluerdma0,bluerdma1 && \
	mpirun -np 2 \
	  -x UCX_NET_DEVICES=blue0,blue1 \
	  -x NCCL_DEBUG=INFO \
	  -x NCCL_DEBUG_SUBSYS=INIT,NET \
	  -x NCCL_IB_DISABLE=0 \
	  -x NCCL_IB_HCA=bluerdma0,bluerdma1 \
	  -x NCCL_P2P_DISABLE=1 \
	  -x NCCL_SHM_DISABLE=1 \
	  -x NCCL_NET_GDR_LEVEL=0 \
	  -x RUST_LOG=debug \
	  -mca coll ^hcoll \
	  $(SIMPLE_TARGET)

# Run normal_test_nompi as rank 0 (server)
nompi_rank0: $(NOMPI_TARGET)
	@echo "=========================================="
	@echo "  Normal Test No-MPI - Rank 0 (Server)"
	@echo "=========================================="
	@echo "Using RDMA device: bluerdma0 only"
	@echo "Run 'make nompi_rank1' in another terminal"
	@echo ""
	export NCCL_DEBUG=INFO && \
	export NCCL_IB_DISABLE=0 && \
	export NCCL_IB_HCA=bluerdma0 && \
	export UCX_NET_DEVICES=blue0 && \
	export NCCL_NET_GDR_LEVEL=0 && \
	export NCCL_P2P_DISABLE=1 && \
	export NCCL_SHM_DISABLE=1 && \
	$(NOMPI_TARGET) 0

# Run normal_test_nompi as rank 1 (client)
nompi_rank1: $(NOMPI_TARGET)
	@echo "=========================================="
	@echo "  Normal Test No-MPI - Rank 1 (Client)"
	@echo "=========================================="
	@echo "Using RDMA device: bluerdma1 only"
	@echo "Connecting to rank 0..."
	@echo ""
	export NCCL_DEBUG=INFO && \
	export NCCL_IB_DISABLE=0 && \
	export NCCL_IB_HCA=bluerdma1 && \
	export UCX_NET_DEVICES=blue1 && \
	export NCCL_NET_GDR_LEVEL=0 && \
	export NCCL_P2P_DISABLE=1 && \
	export NCCL_SHM_DISABLE=1 && \
	$(NOMPI_TARGET) 1

# Run normal_test_nompi as rank 0 (server)
nompi_hack_rank0: $(NOMPI_TARGET)
	@echo "=========================================="
	@echo "  Normal Test No-MPI - Rank 0 (Server)"
	@echo "=========================================="
	@echo "Using RDMA device: bluerdma0 only"
	@echo "Run 'make nompi_rank1' in another terminal"
	@echo ""
	export NCCL_DEBUG=INFO && \
	export NCCL_IB_DISABLE=0 && \
	export NCCL_IB_HCA=bluerdma0 && \
	export UCX_NET_DEVICES=blue0 && \
	export NCCL_NET_GDR_LEVEL=0 && \
	export NCCL_P2P_DISABLE=1 && \
	export NCCL_SHM_DISABLE=1 && \
	export LD_PRELOAD=$(HACK_LD_LIB) && \
	$(NOMPI_TARGET) 0

# Run normal_test_nompi as rank 1 (client)
nompi_hack_rank1: $(NOMPI_TARGET)
	@echo "=========================================="
	@echo "  Normal Test No-MPI - Rank 1 (Client)"
	@echo "=========================================="
	@echo "Using RDMA device: bluerdma1 only"
	@echo "Connecting to rank 0..."
	@echo ""
	export NCCL_DEBUG=INFO && \
	export NCCL_IB_DISABLE=0 && \
	export NCCL_IB_HCA=bluerdma1 && \
	export UCX_NET_DEVICES=blue1 && \
	export NCCL_NET_GDR_LEVEL=0 && \
	export NCCL_P2P_DISABLE=1 && \
	export NCCL_SHM_DISABLE=1 && \
	export LD_PRELOAD=$(HACK_LD_LIB) && \
	$(NOMPI_TARGET) 1





# Show configuration
info:
	@echo "==================================="
	@echo "  Build Configuration"
	@echo "==================================="
	@echo "ROCM_HOME:     $(ROCM_HOME)"
	@echo "DTK_HOME:      $(DTK_HOME)"
	@echo "RCCL_INCLUDE:  $(RCCL_INCLUDE)"
	@echo "RCCL_LIB:      $(RCCL_LIB)"
	@echo "MPI_HOME:      $(MPI_HOME)"
	@echo "MPI_INCLUDE:   $(MPI_INCLUDE)"
	@echo "MPI_LIB:       $(MPI_LIB)"
	@echo "BUILD_DIR:     $(BUILD_DIR)"
	@echo ""
	@echo "Compiler:      $(HIPCC)"
	@echo "Flags:         $(HIPCCFLAGS)"
	@echo "Includes:      $(INCLUDES)"
	@echo "Libs:          $(LIBS)"
	@echo ""
	@echo "RDMA Devices:"
	@rdma link show 2>/dev/null || echo "  No RDMA devices found"
	@echo ""
	@echo "DCU Devices:"
	@rocm-smi --showid 2>/dev/null || echo "  No DCU devices found"
	@echo ""

# Help target
help:
	@echo "================================================"
	@echo "  RCCL Test Suite for ROCm and Blue RDMA"
	@echo "================================================"
	@echo ""
	@echo "Build Targets:"
	@echo "  all          - Build all test programs (default)"
	@echo "  clean        - Remove build directory and artifacts"
	@echo "  rebuild      - Clean and rebuild"
	@echo ""
	@echo "Run Targets (Simple Test):"
	@echo "  simple             - Run simple test with RDMA (2 DCUs) - default"
	@echo "  simple_rdma        - Run simple test with RDMA (2 DCUs)"
	@echo "  simple_rdma_force  - Force IB network (disable P2P/SHM)"
	@echo ""
	@echo "Run Targets (Normal Test - No MPI):"
	@echo "  nompi_rank0  - Run rank 0 with blue0 (run in terminal 1)"
	@echo "  nompi_rank1  - Run rank 1 with blue1 (run in terminal 2)"
	@echo ""
	@echo "Info Targets:"
	@echo "  info         - Show build configuration"
	@echo "  help         - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make                    # Build all programs"
	@echo "  make info               # Show configuration"
	@echo "  make simple_rdma        # Run simple test with RDMA"
	@echo ""
	@echo "  # Run without MPI (in two terminals):"
	@echo "  Terminal 1: make nompi_rank0"
	@echo "  Terminal 2: make nompi_rank1"
	@echo ""
	@echo "Environment Variables (optional):"
	@echo "  NCCL_DEBUG=INFO     - Enable RCCL debug output"
	@echo "  NCCL_IB_DISABLE=0   - Enable InfiniBand/RDMA"
	@echo "  UCX_NET_DEVICES     - Specify network device"
	@echo ""
