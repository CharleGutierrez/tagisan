#!/bin/bash
# sec-kernel PoC 统一构建脚本
#
# 用法: ./build.sh [cve_id]
# 示例: ./build.sh cve_2026_31431
#        ./build.sh  (编译所有 CVE)
#
# 流程:
#   1. 静态编译 C 源码（gcc -static）
#   2. Strip 所有符号（strip --strip-all）
#   3. Mask ELF magic（\x7fELF → CVE\x00）
#   4. 输出 masked 二进制到 poc-bin/
#
# 支持编译:
#   - poc.c -> <cve_id>.bin (本地 LPE 验证)

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
POC_SRC_DIR="$SCRIPT_DIR"
POC_BIN_DIR="$PROJECT_DIR/poc-bin"
COMMON_DIR="$POC_SRC_DIR/common"

# 确保输出目录存在
mkdir -p "$POC_BIN_DIR"

# 编译单个源文件的函数
compile_poc_source() {
    local src_file="$1"
    local output_name="$2"
    local cve_id="$3"
    
    # 检查源文件
    if [ ! -f "$src_file" ]; then
        echo -e "${YELLOW}  Source not found: $src_file (skipping)${NC}"
        return 0
    fi

    echo "  Compiling: $(basename "$src_file") -> ${output_name}.bin"
    
    # Step 1: 静态编译
    # Check if source uses pthread (include <pthread.h>)
    EXTRA_LIBS=""
    if grep -q 'pthread.h' "$src_file" 2>/dev/null; then
        EXTRA_LIBS="-lpthread"
    fi
    
    gcc -static -O2 -Wall -Wextra \
        -o "$POC_BIN_DIR/${output_name}" \
        "$src_file" \
        -I "$COMMON_DIR" \
        -DPOC_NO_SYSTEM_MODIFY=1 \
        $EXTRA_LIBS \
        2>&1
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}  Compilation failed for ${output_name}!${NC}"
        return 1
    fi
    
    # 显示编译后大小
    ORIG_SIZE=$(stat -c%s "$POC_BIN_DIR/${output_name}" 2>/dev/null || stat -f%z "$POC_BIN_DIR/${output_name}")
    
    # Step 2: Strip 所有符号
    strip --strip-all "$POC_BIN_DIR/${output_name}"
    STRIP_SIZE=$(stat -c%s "$POC_BIN_DIR/${output_name}" 2>/dev/null || stat -f%z "$POC_BIN_DIR/${output_name}")
    
    # Step 3: 复制到 .bin 文件
    cp "$POC_BIN_DIR/${output_name}" "$POC_BIN_DIR/${output_name}.bin"
    
    # Step 4: Mask ELF magic (\x7fELF -> CVE\x00)
    printf 'CVE\x00' | dd of="$POC_BIN_DIR/${output_name}.bin" bs=1 count=4 conv=notrunc 2>/dev/null
    
    # 删除未 mask 的原始 ELF
    rm -f "$POC_BIN_DIR/${output_name}"
    
    # 验证
    MAGIC=$(xxd -l 4 -p "$POC_BIN_DIR/${output_name}.bin")
    if [ "$MAGIC" != "43564500" ]; then
        echo -e "${RED}  Magic verification failed for ${output_name}!${NC}"
        return 1
    fi
    
    # 移除执行权限
    chmod -x "$POC_BIN_DIR/${output_name}.bin"
    
    FINAL_SIZE=$(stat -c%s "$POC_BIN_DIR/${output_name}.bin" 2>/dev/null || stat -f%z "$POC_BIN_DIR/${output_name}.bin")
    echo -e "${GREEN}    Success: ${FINAL_SIZE} bytes${NC}"
    
    return 0
}

echo "======================================"
echo " sec-kernel PoC Builder v0.2.0"
echo "======================================"
echo ""

# 如果提供了 CVE ID，只编译该 CVE
if [ -n "$1" ]; then
    CVE_ID="$1"
    SRC_DIR="$POC_SRC_DIR/$CVE_ID"
    
    if [ ! -d "$SRC_DIR" ]; then
        echo -e "${RED}Error: CVE directory not found: $SRC_DIR${NC}"
        exit 1
    fi
    
    echo "CVE ID:  $CVE_ID"
    echo "Source:  $SRC_DIR"
    echo "Output:  $POC_BIN_DIR/"
    echo ""
    
    # 编译主 PoC (poc.c)
    compile_poc_source "$SRC_DIR/poc.c" "$CVE_ID" "$CVE_ID"

    echo ""
    echo "======================================"
    echo -e "${GREEN}BUILD SUCCESS for $CVE_ID${NC}"
    echo "======================================"
else
    # 编译所有 CVE
    echo "Compiling all CVE PoCs..."
    echo ""
    
    BUILD_COUNT=0
    ERROR_COUNT=0
    
    for dir in "$POC_SRC_DIR"/cve_*/; do
        if [ ! -d "$dir" ]; then
            continue
        fi
        
        CVE_ID=$(basename "$dir")
        echo -e "${YELLOW}[$CVE_ID]${NC}"
        
        # 编译 poc.c (必须存在)
        if compile_poc_source "$dir/poc.c" "$CVE_ID" "$CVE_ID"; then
            BUILD_COUNT=$((BUILD_COUNT + 1))
        else
            ERROR_COUNT=$((ERROR_COUNT + 1))
            echo -e "${RED}  Failed to compile poc.c for $CVE_ID${NC}"
        fi
        
        echo ""
    done
    
    echo "======================================"
    echo -e "${GREEN}BUILD COMPLETE${NC}"
    echo "  Successful: $BUILD_COUNT CVEs"
    if [ $ERROR_COUNT -gt 0 ]; then
        echo -e "  ${RED}Errors: $ERROR_COUNT CVEs${NC}"
    fi
    echo "  Output: $POC_BIN_DIR/"
    echo "======================================"
fi
