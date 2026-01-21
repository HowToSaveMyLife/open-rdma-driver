# Base Test - RDMA 测试框架

这是一个重构后的 RDMA 测试框架，将功能库和测试用例分离，提供简洁、可维护的测试代码。

## 目录结构

```
base_test/
├── lib/          → 公共功能库
│   ├── rdma_common.*      - RDMA 基础操作 (设备、QP、Buffer)
│   ├── rdma_transport.*   - 传输层抽象 (TCP 连接、信息交换)
│   └── rdma_debug.*       - 调试工具 (内存对比、打印)
│
├── tests/        → 新测试用例（使用框架）
│   ├── loopback.c         - Loopback 测试
│   ├── send_recv.c        - Send/Recv 测试
│   ├── write_with_imm.c   - RDMA WRITE with Immediate
│   └── rdma_write.c       - RDMA WRITE 多轮测试
│
├── legacy/       → 旧版测试（保留用于对比）
│   ├── loopback.c
│   ├── send_recv.c
│   ├── rdma_client_server.c
│   └── write_imm*.c
│
└── build/        → 构建产物
    ├── obj/              - 库对象文件
    └── bin/              - 可执行文件
```

## 快速开始

### 构建

```bash
# 构建新测试（推荐）
make

# 构建所有测试（包括旧版）
make full

# 只构建旧版测试
make legacy

# 清理构建产物
make clean
```

### 查看可用测试

```bash
make list
```

输出示例：
```
========== Available Tests ==========

New tests (using framework):
  loopback
  send_recv
  write_with_imm
  rdma_write

Legacy tests (original versions):
  legacy_loopback
  legacy_send_recv
  ...
=====================================
```

### 运行测试

#### Loopback 测试
单设备上两个 QP 互相通信：
```bash
./build/bin/loopback 4096 5
# 参数：消息长度(bytes) 测试轮数
```

#### Send/Recv 测试
两个设备之间通信（需要两个终端）：

```bash
# 终端 1 (Server)
./build/bin/send_recv 4096

# 终端 2 (Client)
./build/bin/send_recv 4096 127.0.0.1
```

或使用 Makefile 便捷目标：
```bash
# 终端 1
make run-send-recv-server

# 终端 2
make run-send-recv-client
```

#### RDMA WRITE with Immediate 测试
测试 RDMA WRITE 操作并传递 immediate data：

```bash
# 终端 1 (Server)
./build/bin/write_with_imm 4096

# 终端 2 (Client)
./build/bin/write_with_imm 4096 127.0.0.1
```

#### RDMA WRITE 多轮测试
测试多轮 RDMA WRITE 操作（支持指定设备和轮数）：

```bash
# 终端 1 (Server, 设备1, 5轮测试)
./build/bin/rdma_write 8192 server 1 5

# 终端 2 (Client, 设备0, 5轮测试)
./build/bin/rdma_write 8192 client 127.0.0.1 0 5
```

## 框架优势

### 旧代码 vs 新框架

| 方面 | 旧代码 | 新框架 |
|------|--------|--------|
| 代码量 | 500-700 行/测试 | 150-250 行/测试 |
| 可维护性 | 重复代码多，难以维护 | 公共库统一管理 |
| 可读性 | 混杂底层操作和测试逻辑 | 清晰分离，易于理解 |
| 扩展性 | 需要大量复制粘贴 | 直接使用库，轻松扩展 |

### 代码量对比

```c
// 旧代码 (约 90 行)
void setup_ib(struct rdma_context *ctx, bool is_client, int msg_len) {
    struct ibv_device **dev_list = ibv_get_device_list(NULL);
    // ... 大量样板代码
    ctx->ctx = ibv_open_device(dev_list[dev_index]);
    ctx->pd = ibv_alloc_pd(ctx->ctx);
    // ... 更多重复代码
}

// 新框架 (5 行)
struct rdma_config config;
rdma_default_config(&config);
config.dev_index = 0;
config.buffer_size = msg_len;
rdma_init_context(&ctx, &config);
```

## 库 API 文档

### rdma_common - RDMA 基础操作

#### 初始化和清理
```c
// 配置 RDMA 上下文
struct rdma_config config;
rdma_default_config(&config);  // 使用默认配置
config.dev_index = 0;           // 设备索引
config.buffer_size = 4096;      // Buffer 大小

// 初始化
struct rdma_context ctx;
rdma_init_context(&ctx, &config);

// 清理
rdma_destroy_context(&ctx);
```

#### QP 状态转换
```c
// 方式1: 逐步转换
rdma_qp_to_init(qp);
rdma_qp_to_rtr(qp, dest_qp_num);
rdma_qp_to_rts(qp);

// 方式2: 一步到位（推荐）
rdma_connect_qp(qp, dest_qp_num);
```

### rdma_transport - 传输层抽象

#### TCP Server
```c
struct tcp_transport transport;

// 初始化服务器
tcp_server_init(&transport, port);

// 等待客户端连接
tcp_server_accept(&transport);

// 交换 QP 信息
rdma_exchange_qp_info(transport.client_fd, &local_info, &remote_info);

// 同步
rdma_handshake(transport.client_fd);

// 清理
tcp_transport_close(&transport);
```

#### TCP Client
```c
struct tcp_transport transport;

// 连接服务器（带重试）
tcp_client_connect(&transport, server_ip, port, max_retries);

// 交换 QP 信息
rdma_exchange_qp_info(transport.sock_fd, &local_info, &remote_info);

// 清理
tcp_transport_close(&transport);
```

### rdma_debug - 调试工具

```c
// 内存对比（返回差异字节数）
size_t errors = rdma_memory_diff(expected, actual, length);

// 打印内存（十六进制 + ASCII）
rdma_print_memory_hex(buffer, length);

// 查找零字节范围
rdma_print_zero_ranges(buffer, length);

// 编译器屏障（防止优化）
COMPILER_BARRIER();
```

## 添加新测试

### 1. 创建测试文件

在 `tests/` 目录创建 `my_test.c`：

```c
#include "../lib/rdma_common.h"
#include "../lib/rdma_transport.h"
#include "../lib/rdma_debug.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
    // 1. 配置 RDMA
    struct rdma_context ctx;
    struct rdma_config config;
    rdma_default_config(&config);
    config.buffer_size = 4096;

    // 2. 初始化
    if (rdma_init_context(&ctx, &config) < 0) {
        return -1;
    }

    // 3. 测试逻辑
    // ...

    // 4. 清理
    rdma_destroy_context(&ctx);
    return 0;
}
```

### 2. 构建并运行

```bash
# Makefile 会自动发现新测试
make

# 运行
./build/bin/my_test
```

## 帮助命令

```bash
# 显示所有可用目标
make help

# 列出可用测试
make list

# 显示目录结构
make tree
```

## 环境变量

- `EXTRA_CFLAGS` - 额外的编译选项
  ```bash
  make EXTRA_CFLAGS="-DCOMPILE_FOR_RTL_SIMULATOR_TEST"
  ```

- `RDMA_BUFFER_SIZE` - 运行时 buffer 大小（部分测试支持）

## 故障排除

### 找不到设备
```
[ERROR] Device index 0 not available (found 0 devices)
```
**解决**: 确保 `LD_LIBRARY_PATH` 包含 libibverbs 和驱动库路径。

### 连接失败
```
[ERROR] Failed to connect after all retries
```
**解决**:
1. 检查服务器是否已启动
2. 检查 IP 地址是否正确
3. 检查防火墙设置

### 编译错误
```
fatal error: rdma_common.h: No such file or directory
```
**解决**: 确保测试文件使用 `#include "../lib/rdma_common.h"` 而不是 `#include "rdma_common.h"`

## 贡献

添加新测试时请遵循：
1. 使用新框架（`lib/` 中的库）
2. 保持代码简洁（< 250 行）
3. 添加充分的错误处理
4. 更新此 README

## 参考

- [REFACTORING.md](REFACTORING.md) - 详细的重构文档
- [rdma_common.h](lib/rdma_common.h) - API 定义
- [tests/loopback.c](tests/loopback.c) - 示例测试代码
