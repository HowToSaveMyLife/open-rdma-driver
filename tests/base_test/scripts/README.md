# Base Test Scripts

这个目录包含用于运行 RDMA 基础测试的脚本，主要用于 RTL 模拟器环境。

## 脚本清单

### 核心脚本

#### `run_dual_sim_test.sh`
通用的双端 RTL 模拟器测试框架，用于运行需要 server/client 模式的测试程序。

**用法**:
```bash
./run_dual_sim_test.sh <test_program> [args...]
```

**示例**:
```bash
./run_dual_sim_test.sh send_recv 4096
./run_dual_sim_test.sh rdma_write 8192 10
```

**功能**:
- 自动启动 RTL 模拟器
- 编译 Rust 驱动和测试程序
- 启动 server 进程（使用传入的参数）
- 启动 client 进程（自动添加 `127.0.0.1` 作为服务器地址）
- 收集日志到 `../log/sim/<test_program>/` 目录

---

### 单测试脚本

#### `test_loopback_sim.sh`
运行 loopback 测试（单端测试，无需 server/client）。

**用法**:
```bash
./test_loopback_sim.sh [msg_len]
```

**参数**:
- `msg_len`: 消息长度（字节），默认 4096

**示例**:
```bash
./test_loopback_sim.sh          # 使用默认 4096 字节
./test_loopback_sim.sh 8192     # 测试 8KB 消息
```

---

#### `test_send_recv_sim.sh`
运行 Send/Recv 测试。

**用法**:
```bash
./test_send_recv_sim.sh [msg_len]
```

**参数**:
- `msg_len`: 消息长度（字节），默认 4096

**示例**:
```bash
./test_send_recv_sim.sh          # 使用默认 4096 字节
./test_send_recv_sim.sh 16384    # 测试 16KB 消息
```

---

#### `test_rdma_write_sim.sh`
运行 RDMA WRITE 测试，支持多轮重复测试。

**用法**:
```bash
./test_rdma_write_sim.sh [msg_len] [num_rounds]
```

**参数**:
- `msg_len`: 消息长度（字节），默认 4096
- `num_rounds`: 测试轮数，默认 5

**示例**:
```bash
./test_rdma_write_sim.sh                # 4096 字节，5 轮
./test_rdma_write_sim.sh 8192 10        # 8192 字节，10 轮
```

**数据验证**: 使用顺序字节模式 `(i & 0xFF)`

---

#### `test_write_imm_sim.sh`
运行 RDMA WRITE with Immediate 测试（单次操作）。

**用法**:
```bash
./test_write_imm_sim.sh [msg_len]
```

**参数**:
- `msg_len`: 消息长度（字节），默认 4096

**示例**:
```bash
./test_write_imm_sim.sh              # 4096 字节
./test_write_imm_sim.sh 2048         # 2048 字节
```

**数据验证**: 使用固定字符 `'W'` 填充

---

#### `test_write_imm_single_sim.sh`
运行单次 RDMA WRITE with Immediate 测试，支持零长度测试。

**用法**:
```bash
./test_write_imm_single_sim.sh [msg_len]
```

**参数**:
- `msg_len`: 消息长度（字节），默认 0（零长度测试）

**示例**:
```bash
./test_write_imm_single_sim.sh       # 零长度测试（只传输 immediate 数据）
./test_write_imm_single_sim.sh 4096  # 4KB 数据 + immediate
```

**用途**: 特别用于测试零长度 WRITE_WITH_IMM 操作（只传输 immediate 值，无数据负载）

---

#### `run_all_tests.sh`
运行所有测试的测试套件，自动执行所有测试并汇总结果。

**用法**:
```bash
./run_all_tests.sh
```

**测试内容**:
1. Loopback (4KB)
2. Send/Recv (4KB)
3. RDMA WRITE (4KB, 5 轮)
4. WRITE with IMM (4KB, 10 次操作)
5. WRITE with IMM Single (零长度)
6. WRITE with IMM Single (4KB)

**输出示例**:
```
==========================================
          Test Suite Summary
==========================================

PASS: Loopback (4KB)
PASS: Send/Recv (4KB)
PASS: RDMA WRITE (4KB, 5 rounds)
FAIL: WRITE with IMM (4KB, 10 ops)
PASS: WRITE with IMM Single (zero-length)
PASS: WRITE with IMM Single (4KB)

==========================================
Total:  6
Passed: 5
Failed: 1
==========================================
```

---

## 测试程序参数格式

所有测试程序统一遵循以下参数格式：

### Server 模式（无 IP 地址参数）
```bash
<program> [msg_len] [other_args...]
```

### Client 模式（包含 IP 地址参数）
```bash
<program> [msg_len] <server_ip> [other_args...]
```

**自动检测规则**: 如果第二个参数包含 `.` 或 `:` 字符，则判断为 client 模式。

### 各程序参数详情

| 程序 | Server 参数 | Client 参数 | 默认值 |
|------|------------|------------|--------|
| `loopback` | `[msg_len]` | N/A（单端测试） | msg_len=4096 |
| `send_recv` | `[msg_len]` | `[msg_len] <server_ip>` | msg_len=4096 |
| `rdma_write` | `[msg_len] [rounds]` | `[msg_len] [rounds] <server_ip>` | msg_len=4096, rounds=5, dev=1(server)/0(client) 固定 |
| `write_with_imm` | `[msg_len]` | `[msg_len] <server_ip>` | msg_len=4096 |

---

## 日志输出

所有测试日志保存在 `../log/sim/<test_name>/` 目录下：

```
log/sim/
├── send_recv/
│   ├── server.log
│   └── client.log
├── rdma_write/
│   ├── server.log
│   └── client.log
└── write_with_imm/
    ├── server.log
    └── client.log
```

**查看日志**:
```bash
# 实时查看 server 日志
tail -f ../log/sim/rdma_write/server.log

# 查看 client 日志
cat ../log/sim/rdma_write/client.log
```

---

## 环境变量

### `RUST_LOG`
控制 Rust 驱动的日志级别。

**默认值**: `info`

**可选值**: `trace`, `debug`, `info`, `warn`, `error`

**示例**:
```bash
RUST_LOG=debug ./test_send_recv_sim.sh 4096
```

### `RDMA_BUFFER_SIZE`
设置 RDMA 缓冲区大小（可选）。

**示例**:
```bash
RDMA_BUFFER_SIZE=524288 ./test_rdma_write_sim.sh 8192
```

---

## 数据验证

所有测试都使用统一的数据验证框架（位于 `lib/rdma_debug.h`）：

### 验证函数
- `rdma_verify_data()`: 统一数据验证接口
- `rdma_generate_pattern()`: 自动生成期望数据模式

### 数据模式

| 测试 | 数据模式 | 说明 |
|------|---------|------|
| `loopback` | 顺序字节 `(i & 0xFF)` | 0x00, 0x01, ..., 0xFF, 0x00, ... |
| `send_recv` | 固定字符 `'c'` (0x63) | 全部填充字符 'c' |
| `rdma_write` | 顺序字节 `(i & 0xFF)` | 0x00, 0x01, ..., 0xFF, 0x00, ... |
| `write_with_imm` | 固定字符 `'W'` (0x57) | 全部填充字符 'W' |

### 验证失败输出

当数据不匹配时，验证函数会自动打印彩色差异：
- **红色**: 期望值
- **绿色**: 实际值
- 显示差异字节的前后文（每行 16 字节）

**示例输出**:
```
0x00000100: 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f     ................
0x00000110: 10 11 12 ff 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f     ................  (期望)
0x00000110: 10 11 12 aa 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f     ................  (实际)
              ^^        ^^
0x00000120: 20 21 22 23 24 25 26 27 28 29 2a 2b 2c 2d 2e 2f     !"#$%&'()*+,-./
```

---

## 故障排查

### 编译失败
```bash
# 清理并重新编译
cd ..
make clean
make
```

### RTL 模拟器启动失败
检查 `test_common.sh` 中的 RTL 路径配置：
```bash
# 查看共同函数库
cat ../../common/test_common.sh
```

### 测试卡住/超时
检查日志文件：
```bash
# 查看 server 日志
tail -20 ../log/sim/<test_name>/server.log

# 查看 client 日志
tail -20 ../log/sim/<test_name>/client.log
```

### 数据验证失败
- 日志会自动显示详细的字节差异
- 检查测试程序的数据模式是否正确
- 确认 RTL 模拟器正常工作

---

## 贡献

添加新测试时，请遵循以下规范：

1. **统一参数格式**: Server 不带 IP，Client 带 IP
2. **使用统一验证**: 调用 `rdma_verify_data()` 函数
3. **创建包装脚本**: 参考 `test_*_sim.sh` 格式
4. **更新文档**: 在本 README 中添加说明

---

## 许可证

与主项目相同的许可证。
