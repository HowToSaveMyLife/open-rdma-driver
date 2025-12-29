#!/bin/bash

# write_imm 测试脚本 - 使用通用的双端 sim 测试框架
# 用法: ./test_write_imm_sim.sh
# 注意: nccl_pattern_test 不接受参数

# 设置目录路径
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

# 默认消息长度
MSG_LEN=${1:-4096}

# 调用通用的双端测试脚本
# 注意: write_imm 测试实际运行的是 nccl_pattern_test 程序，它不需要 msg_len 参数
exec "$SCRIPT_DIR/run_dual_sim_test.sh" write_imm $MSG_LEN
