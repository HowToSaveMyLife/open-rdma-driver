#!/bin/bash

# 设置目录路径
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
DRIVER_DIR=$(cd "$SCRIPT_DIR/../../.." && pwd)

# 设置日志目录
mkdir -p $SCRIPT_DIR/../log/sim
LOG_DIR=$(cd "$SCRIPT_DIR/../log/sim" && pwd)

# Source 共同函数库
source $SCRIPT_DIR/../../common/test_common.sh

# 设置信号处理
setup_signal_handler

# 打印测试开始信息
print_test_start "send_recv"

# 初始化测试环境
init_test_environment

# 编译 Rust 驱动
build_rust_driver "sim"

# 启动 RTL 模拟器（2个实例）
start_rtl_simulators 2 "send_recv"

# 编译测试程序
build_test_program "$SCRIPT_DIR/.."

# 设置运行时环境
setup_runtime_environment

# 运行 send_recv 测试，参数是消息长度
MSG_LEN=${1:-4096}  # 默认 4096 字节

echo "Running send_recv test with MSG_LEN=$MSG_LEN"

cd $SCRIPT_DIR/..

# 先启动 server
echo "Starting server..."
sudo env RUST_LOG=info LD_LIBRARY_PATH="$LD_LIBRARY_PATH" ./build/send_recv $MSG_LEN &> $LOG_DIR/send_recv-server.log &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"

# 等待 server 启动
sleep 3

# 启动 client，连接到 localhost
echo "Starting client..."
sudo env RUST_LOG=info LD_LIBRARY_PATH="$LD_LIBRARY_PATH" ./build/send_recv $MSG_LEN 127.0.0.1 &> $LOG_DIR/send_recv-client.log &
CLIENT_PID=$!

echo "Client PID: $CLIENT_PID"

# 只等待测试程序，不等待 RTL 进程
wait $SERVER_PID $CLIENT_PID

# 打印测试结束信息
print_test_end "send_recv"

