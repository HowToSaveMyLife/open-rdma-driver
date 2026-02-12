# Base Test Scripts

用于运行 RDMA 基础测试的自动化脚本，主要用于 RTL 模拟器环境（Sim 模式）。

## 环境准备

### RTL 仿真器路径配置

测试脚本需要访问 RTL 仿真器代码（`open-rdma-rtl` 仓库）。

**配置方式 1：默认路径（推荐）**

将 `open-rdma-rtl` 克隆到与 `open-rdma-driver` 同级目录：

```bash
cd /path/to/parent-directory
git clone https://github.com/open-rdma/open-rdma-rtl.git
```

目录结构：
```
parent-directory/
├── open-rdma-driver/
└── open-rdma-rtl/
```

**配置方式 2：自定义路径**

设置 `RTL_DIR` 环境变量：

```bash
export RTL_DIR=/path/to/your/open-rdma-rtl
# 或在运行时指定
RTL_DIR=/custom/path ./test_loopback_sim.sh
```

## 快速使用

### 运行单个测试

```bash
./test_loopback_sim.sh [msg_len]
./test_send_recv_sim.sh [msg_len]
./test_rdma_write_sim.sh [msg_len] [rounds]
./test_write_imm_sim.sh [msg_len]
```

**示例**：
```bash
./test_loopback_sim.sh 4096              # Loopback，4KB 消息
./test_send_recv_sim.sh 8192             # Send/Recv，8KB 消息
./test_rdma_write_sim.sh 4096 10         # RDMA Write，4KB，10 轮
./test_write_imm_sim.sh 0                # Write with Imm，零长度
```

### 运行所有测试

```bash
./run_all_tests.sh
```

输出示例：
```
==========================================
          Test Suite Summary
==========================================
PASS: Loopback (4KB)
PASS: Send/Recv (4KB)
PASS: RDMA WRITE (4KB, 5 rounds)
PASS: WRITE with IMM (4KB)
==========================================
Total:  4
Passed: 4
Failed: 0
==========================================
```

## 脚本详细说明

### test_loopback_sim.sh
单端回环测试，一个设备上两个 QP 互相通信。

**参数**：
- `msg_len`：消息长度（字节），默认 4096

### test_send_recv_sim.sh
双端 Send/Recv 测试。

**参数**：
- `msg_len`：消息长度（字节），默认 4096

### test_rdma_write_sim.sh
双端 RDMA WRITE 多轮测试。

**参数**：
- `msg_len`：消息长度（字节），默认 4096
- `rounds`：测试轮数，默认 5

### test_write_imm_sim.sh
双端 RDMA WRITE with Immediate 测试。

**参数**：
- `msg_len`：消息长度（字节），默认 4096
  - 可以设置为 0 进行零长度测试（只传输 immediate 值）

### run_dual_sim_test.sh
通用的双端测试框架，其他脚本基于此实现。

**用法**：
```bash
./run_dual_sim_test.sh <test_program> [args...]
```

## 测试日志

日志保存在 `../log/sim/` 目录：

```
log/sim/
├── rtl-loopback.log           # Loopback RTL 日志
├── send_recv/
│   ├── server.log             # Server 应用日志
│   ├── client.log             # Client 应用日志
│   ├── rtl-server.log         # Server RTL 日志
│   └── rtl-client.log         # Client RTL 日志
└── rdma_write/
    └── ...
```

**查看日志**：
```bash
cat ../log/sim/rtl-loopback.log               # 查看日志
tail -f ../log/sim/send_recv/server.log       # 实时查看
```

## 脚本功能

所有测试脚本会自动执行以下操作：

1. **初始化环境**：设置 DRIVER_DIR、RTL_DIR 路径
2. **编译 Rust 驱动**：使用 sim 特性编译 dtld-ibverbs
3. **启动 RTL 仿真器**：自动启动所需数量的 RTL 实例
4. **编译测试程序**：编译 base_test 测试程序
5. **运行测试**：启动 server/client 进程（双端测试）
6. **收集日志**：所有输出保存到日志文件
7. **清理资源**：测试结束后自动停止 RTL 仿真器

## 环境变量

### RTL_DIR
RTL 仓库路径（可选，默认为 `../../../open-rdma-rtl`）

```bash
export RTL_DIR=/path/to/open-rdma-rtl
```

### RUST_LOG
Rust 驱动日志级别（默认 `info`）

```bash
RUST_LOG=debug ./test_loopback_sim.sh
```

可选值：`trace`, `debug`, `info`, `warn`, `error`

## 故障排查

### RTL 目录未找到
```
Error: RTL directory not found: /path/to/open-rdma-rtl
```
**解决**：
- 确认 RTL 仓库已克隆
- 检查目录结构或设置 `RTL_DIR` 环境变量

### RTL 启动失败
```
Error: RTL process failed to start or died
```
**解决**：
- 查看 RTL 日志：`cat ../log/sim/rtl-*.log`
- 确保 RTL 仓库完整（包含子模块）

### 测试超时
**解决**：
- 检查应用日志：`tail ../log/sim/<test>/server.log`
- 检查 RTL 日志是否有错误

### 数据验证失败
测试会自动显示字节级差异，检查：
- 日志中的详细差异信息
- RTL 仿真器是否正常工作

## 参考

- [../README.md](../README.md) - 测试框架总览
- [../../common/test_common.sh](../../common/test_common.sh) - 公共测试函数库
