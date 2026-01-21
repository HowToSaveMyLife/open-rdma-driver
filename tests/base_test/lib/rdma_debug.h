#ifndef RDMA_DEBUG_H
#define RDMA_DEBUG_H

#include <stddef.h>
#include <stdint.h>

// ANSI color codes
#define ANSI_COLOR_RED     "\x1b[31m"
#define ANSI_COLOR_GREEN   "\x1b[32m"
#define ANSI_COLOR_YELLOW  "\x1b[33m"
#define ANSI_COLOR_BLUE    "\x1b[34m"
#define ANSI_COLOR_MAGENTA "\x1b[35m"
#define ANSI_COLOR_CYAN    "\x1b[36m"
#define ANSI_COLOR_RESET   "\x1b[0m"

// Compiler barrier
#define COMPILER_BARRIER() asm volatile("" ::: "memory")

// Memory comparison and visualization
size_t rdma_memory_diff(const char *buf1, const char *buf2, size_t length);
void rdma_print_memory_hex(const void *start_addr, size_t length);
void rdma_print_zero_ranges(const char *buffer, size_t length);

#endif // RDMA_DEBUG_H
