# Base Test - RDMA 测试框架

重构后的 RDMA 测试框架，将功能库和测试用例分离，提供简洁、可维护的测试代码。

## 目录结构

```
base_test/
├── lib/          → 公共功能库
│   ├── rdma_common.*      - RDMA 基础操作 (设备、QP、Buffer)
│   ├── rdma_transport.*   - 传输层抽象 (TCP 连接、信息交换)
│   └── rdma_debug.*       - 调试工具 (内存对比、打印)
├── tests/        → 测试用例
│   ├── loopback.c         - Loopback 测试
│   ├── send_recv.c        - Send/Recv 测试
│   ├── write_with_imm.c   - RDMA WRITE with Immediate
│   └── rdma_write.c       - RDMA WRITE 多轮测试
├── scripts/      → 自动化测试脚本（Sim 模式）
└── build/        → 构建产物
    ├── obj/      - 库对象文件
    └── bin/      - 可执行文件
```

## 快速开始

### 1. 编译测试

```bash
make              # 编译所有测试
make clean        # 清理构建产物
make list         # 列出可用测试
```

### 2. 运行测试

#### Mock 模式（不需要 RTL 仿真器）

直接运行编译好的测试程序：

```bash
# Loopback 测试
./build/bin/loopback 4096

# Send/Recv 测试（需要两个终端）
./build/bin/send_recv 4096              # 终端 1: Server
./build/bin/send_recv 4096 127.0.0.1    # 终端 2: Client

# RDMA WRITE 测试
./build/bin/rdma_write 8192 server 1 5              # 终端 1: Server
./build/bin/rdma_write 8192 client 127.0.0.1 0 5    # 终端 2: Client

# WRITE with Immediate
./build/bin/write_with_imm 4096              # 终端 1: Server
./build/bin/write_with_imm 4096 127.0.0.1    # 终端 2: Client
```

#### Sim 模式（需要 RTL 仿真器）

使用自动化脚本，会自动启动 RTL 仿真器、编译驱动和测试程序。

**环境准备：配置 RTL 路径**

将 `open-rdma-rtl` 仓库克隆到与 `open-rdma-driver` 同级目录：

```bash
# 在父目录下运行
cd /path/to/parent-directory
git clone https://github.com/open-rdma/open-rdma-rtl.git
```

目录结构：
```
parent-directory/
├── open-rdma-driver/
│   └── tests/base_test/  ← 当前目录
└── open-rdma-rtl/        ← RTL 仓库
```

或设置环境变量指定自定义路径：
```bash
export RTL_DIR=/path/to/your/open-rdma-rtl
```

**运行测试：**

```bash
cd scripts/

# 运行单个测试
./test_loopback_sim.sh 4096
./test_send_recv_sim.sh 4096
./test_rdma_write_sim.sh 4096 5
./test_write_imm_sim.sh 4096

# 运行所有测试
./run_all_tests.sh
```

测试日志保存在 `log/sim/` 目录：
```bash
cat log/sim/rtl-loopback.log                # Loopback 日志
cat log/sim/send_recv/server.log            # Send/Recv Server 日志
tail -f log/sim/rdma_write/client.log       # 实时查看 Client 日志
```

## 库 API 文档

### rdma_common - RDMA 基础操作

```c
// 初始化
struct rdma_context ctx;
struct rdma_config config;
rdma_default_config(&config);
config.dev_index = 0;
config.buffer_size = 4096;
rdma_init_context(&ctx, &config);

// QP 状态转换
rdma_connect_qp(qp, dest_qp_num);  // 一步到位

// 清理
rdma_destroy_context(&ctx);
```

### rdma_transport - 传输层抽象

```c
// Server 端
struct tcp_transport transport;
tcp_server_init(&transport, port);
tcp_server_accept(&transport);
rdma_exchange_qp_info(transport.client_fd, &local_info, &remote_info);

// Client 端
tcp_client_connect(&transport, server_ip, port, max_retries);
rdma_exchange_qp_info(transport.sock_fd, &local_info, &remote_info);

// 清理
tcp_transport_close(&transport);
```

### rdma_debug - 调试工具

```c
rdma_memory_diff(expected, actual, length);    // 内存对比
rdma_print_memory_hex(buffer, length);         // 打印内存
COMPILER_BARRIER();                            // 编译器屏障
```

## 添加新测试

在 `tests/` 目录创建新文件：

```c
#include "../lib/rdma_common.h"
#include "../lib/rdma_transport.h"
#include "../lib/rdma_debug.h"

int main(int argc, char *argv[]) {
    struct rdma_context ctx;
    struct rdma_config config;
    rdma_default_config(&config);
    config.buffer_size = 4096;

    rdma_init_context(&ctx, &config);
    // 测试逻辑...
    rdma_destroy_context(&ctx);
    return 0;
}
```

然后运行 `make` 会自动编译新测试。

## 故障排除

### RTL 目录未找到
```
Error: RTL directory not found
```
**解决**：确认 `open-rdma-rtl` 与 `open-rdma-driver` 在同级目录，或设置 `RTL_DIR` 环境变量

### 找不到设备
```
[ERROR] Device index 0 not available
```
**解决**：确保 `LD_LIBRARY_PATH` 包含 libibverbs 和驱动库路径

### 编译错误
```
fatal error: rdma_common.h: No such file or directory
```
**解决**：使用 `#include "../lib/rdma_common.h"` 而不是 `#include "rdma_common.h"`

## 参考

- [scripts/README.md](scripts/README.md) - 测试脚本使用指南
- [lib/rdma_common.h](lib/rdma_common.h) - API 定义
- [tests/loopback.c](tests/loopback.c) - 示例测试代码
