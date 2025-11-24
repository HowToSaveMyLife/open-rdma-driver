# Blue RDMA Driver 安装指南

本文档提供 Blue RDMA Driver 的快速安装步骤。详细技术细节和故障排除请参考 [detail](./detail/) 文件夹中的文档。

## 环境要求

- Linux 系统（支持 WSL2）
- Rust 工具链
- 内核版本 >= 6.6（WSL 需要自行编译内核模块）

## 安装步骤

### 1. 安装 Rust 工具链

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. 安装系统依赖

```bash
sudo apt install cmake libnl-3-dev libnl-route-3-dev libclang-dev libibverbs-dev
```

### 3. 克隆项目并初始化子模块

```bash
git clone --recursive https://github.com/bsbds/blue-rdma-driver.git
cd blue-rdma-driver

# 如果克隆时未使用 --recursive，可以手动初始化
git submodule update --init --recursive
```

**注意**：项目路径不宜过长，建议使用 `/home/user/blue-rdma-driver` 而非深层嵌套路径。详见：[路径长度问题](./detail/path-length-issue.md)

### 4. 编译并加载驱动模块

**WSL2 环境**需要先准备内核头文件，详细步骤请参考：[WSL2 内核头文件准备指南](./detail/wsl-kernel-headers.md)，或者可以使用 `make KBUILD_MODPOST_WARN=1` 跳过使用内核头文件，但有一定风险


**编译驱动**：
```bash
# 编译驱动
make

# 如果 BTF 生成失败（常见于 WSL），可使用：
# make KBUILD_MODPOST_WARN=1

# 加载驱动模块
make install
```

**验证驱动加载成功**：
```bash
lsmod | grep bluerdma
# 应显示：bluerdma
```

### 5. 分配大页内存

Blue RDMA Driver 需要使用大页内存。使用提供的脚本分配 2048 MB 大页：

```bash
./scripts/hugepages.sh alloc 2048
```

验证分配成功：
```bash
cat /proc/meminfo | grep Huge
```

### 6. 编译用户态库（dtld-ibverbs）

根据使用场景选择编译模式：

**Mock 模式（推荐用于开发测试）**：
```bash
cd dtld-ibverbs
cargo build --no-default-features --features mock
cd ..
```
- 不依赖真实硬件或仿真器
- 适合快速开发和功能测试
- 性能测试结果不真实

**Sim 模式（用于 RTL 仿真器调试）**：
```bash
cd dtld-ibverbs
cargo build --no-default-features --features sim
cd ..
```
- 需要先启动 RTL 仿真器
- 用于硬件逻辑验证
- 在运行测试前必须启动仿真器：
  ```bash
  # 启动仿真器（在单独的终端中运行）
  # <启动仿真器的具体命令>
  ```

**硬件模式（hw）**：
```bash
cd dtld-ibverbs
cargo build --no-default-features --features hw
cd ..
```
- ⚠️ **注意**：硬件模式尚未完全测试，可能存在问题
- 仅在有真实硬件设备时使用

### 7. 编译 rdma-core

```bash
cd dtld-ibverbs/rdma-core-55.0

# 基本编译
./build.sh

# 如需生成 compile_commands.json 用于调试：
# export EXTRA_CMAKE_FLAGS=-DCMAKE_EXPORT_COMPILE_COMMANDS=1
# ./build.sh

cd ../..
```

**常见问题**：如果编译在 81% 左右失败，提示 "size of unnamed array is negative"，这是路径过长导致的。请参考：[路径长度问题详解](./detail/path-length-issue.md)

### 8. 配置网络接口

为 Blue RDMA 虚拟网络接口分配 IP 地址：

```bash
sudo ip addr add 17.34.51.10/24 dev blue0
sudo ip addr add 17.34.51.11/24 dev blue1
```

验证配置：
```bash
ip addr show blue0
ip addr show blue1
```

### 9. 设置环境变量

**方法一：永久设置（推荐）**

直接将环境变量添加到 `~/.bashrc`，使其在每次打开终端时自动加载：

```bash
# 获取项目绝对路径
PROJECT_PATH=$(pwd)

# 添加到 .bashrc
cat >> ~/.bashrc << EOF

# Blue RDMA Driver Environment
export LD_LIBRARY_PATH=$PROJECT_PATH/dtld-ibverbs/target/debug:$PROJECT_PATH/dtld-ibverbs/rdma-core-55.0/build/lib:\${LD_LIBRARY_PATH}
EOF

# 立即生效
source ~/.bashrc
```

**方法二：临时设置（仅当前终端）**

```bash
# 使用提供的脚本
source ./scripts/setup-env.sh

# 或手动设置
export LD_LIBRARY_PATH=$PWD/dtld-ibverbs/target/debug:$PWD/dtld-ibverbs/rdma-core-55.0/build/lib
```

### 10. 验证安装

编译示例程序：

```bash
cd examples
make
```

**运行示例程序**：

根据编译时选择的模式运行：

**Mock 模式**：
```bash
RUST_LOG=debug ./loopback 8192
```

**Sim 模式**：
```bash
# 1. 先在单独的终端启动仿真器
# <启动仿真器的命令>

# 2. 然后运行测试
RUST_LOG=debug ./loopback 8192
```

成功运行将显示 RDMA 操作的调试日志。

双端测试请使用 send_recv 程序，同时请分别启用两个不同的仿真器程序

## 快速命令总结

```bash
# 1. 环境准备
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install cmake libnl-3-dev libnl-route-3-dev libclang-dev libibverbs-dev

# 2. 克隆项目
git clone --recursive https://github.com/bsbds/blue-rdma-driver.git
cd blue-rdma-driver

# 3. 编译并加载驱动（WSL2 需要先准备内核头文件）

make && make install

# 4. 分配大页
./scripts/hugepages.sh alloc 2048

# 5. 编译用户态库（选择模式：mock/sim/hw）
# Mock 模式（推荐）：
cd dtld-ibverbs && cargo build --no-default-features --features mock && cd ..
# Sim 模式（需要先启动仿真器）：
# cd dtld-ibverbs && cargo build --no-default-features --features sim && cd ..
# 硬件模式（未测试）：
# cd dtld-ibverbs && cargo build --no-default-features --features hw && cd ..

# 6. 编译 rdma-core
cd dtld-ibverbs/rdma-core-55.0 && ./build.sh && cd ../..

# 7. 配置网络
sudo ip addr add 17.34.51.10/24 dev blue0
sudo ip addr add 17.34.51.11/24 dev blue1

# 8. 设置环境变量（永久）
PROJECT_PATH=$(pwd)
cat >> ~/.bashrc << EOF

# Blue RDMA Driver Environment
export LD_LIBRARY_PATH=$PROJECT_PATH/dtld-ibverbs/target/debug:$PROJECT_PATH/dtld-ibverbs/rdma-core-55.0/build/lib:\${LD_LIBRARY_PATH}
EOF
source ~/.bashrc

# 9. 运行示例
cd examples && make && RUST_LOG=debug ./loopback 8192
```

## 常见问题

### Q1: rdma-core 编译在 81% 时失败
**原因**：项目路径过长，导致 Unix socket 路径超出限制。
**解决**：将项目移至较短路径（如 `/home/user/blue-rdma-driver`）。
**详见**：[路径长度问题](./detail/path-length-issue.md)

### Q2: 找不到 `infiniband/verbs_api.h`
**原因**：缺少 `libibverbs-dev` 包。
**解决**：`sudo apt install libibverbs-dev`

### Q3: WSL 下驱动编译失败，提示找不到内核头文件
**原因**：WSL 默认不提供内核头文件。
**解决**：参考步骤 3，编译 WSL2 内核并链接头文件。
**详见**：[WSL2 内核头文件准备指南](./detail/wsl-kernel-headers.md)

### Q4: 运行示例时没有发现RDMA devices，或者提示找不到共享库
**原因**：可能是未设置 `LD_LIBRARY_PATH`。
**解决**：执行 `source ./scripts/setup-env.sh`

### Q5: OFED 与 vanilla RDMA 冲突
**详见**：[切换到 vanilla RDMA](./detail/switch-to-vanilla-rdma.md)

### Q6: 如何选择编译模式（mock/sim/hw）？
**Mock 模式**：推荐用于开发和功能测试，不需要硬件或仿真器
**Sim 模式**：用于 RTL 仿真验证，需要先启动仿真器
**硬件模式**：仅在有真实硬件设备时使用，⚠️ 目前尚未完全测试

### Q7: Sim 模式下示例程序无法运行
**原因**：未启动 RTL 仿真器
**解决**：
1. 在单独的终端启动仿真器
2. 确保仿真器正常运行后再执行测试程序

## 相关文档

- [WSL2 内核头文件准备指南](./detail/wsl-kernel-headers.md)
- [路径长度问题详解](./detail/path-length-issue.md)
- [OFED 符号版本修复](./detail/ofed-symbol-version-fix.md)
- [OFED RoCE 注册问题](./detail/ofed-roce-registration-issue.md)
- [切换到 vanilla RDMA](./detail/switch-to-vanilla-rdma.md)


