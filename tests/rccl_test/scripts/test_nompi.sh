
trap "kill 0" SIGINT

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

echo $SCRIPT_DIR

cd $SCRIPT_DIR/..

echo $(pwd)

make

# LD_PRELOAD=/home/peng/projects/rdma_all/hack_libc/target/debug/libhack_libc.so make nompi_rank0 &
# LD_PRELOAD=/home/peng/projects/rdma_all/hack_libc/target/debug/libhack_libc.so make nompi_rank1 &


make nompi_hack_rank0 &
make nompi_hack_rank1 &
wait

