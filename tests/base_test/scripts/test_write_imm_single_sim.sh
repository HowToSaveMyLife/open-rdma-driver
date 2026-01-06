#!/bin/bash

# write_imm_single 测试脚本 - 测试单次 WRITE_WITH_IMM 操作（支持可调长度，包括零长度）
# 用法: ./test_write_imm_single_sim.sh [msg_len]
# 例如: ./test_write_imm_single_sim.sh 0      # 零长度测试
#       ./test_write_imm_single_sim.sh 4096   # 4KB 数据测试

# 设置目录路径
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

# 默认消息长度（0 表示零长度测试）
MSG_LEN=${1:-0}

echo "Running write_imm_single test with msg_len=$MSG_LEN"

# 调用通用的双端测试脚本
exec "$SCRIPT_DIR/run_dual_sim_test.sh" write_imm_single $MSG_LEN
