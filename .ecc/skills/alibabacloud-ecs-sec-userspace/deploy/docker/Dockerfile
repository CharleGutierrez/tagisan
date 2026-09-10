FROM python:3.12-slim

LABEL maintainer="sec-userspace"
LABEL description="Linux Security Inspection Tool"

# 零外部依赖，无需 pip install
WORKDIR /data/sec-userspace

# 复制项目文件（thirdparties 已在 scripts/ 内）
COPY scripts/ ./scripts/
COPY assets/ ./assets/
COPY configs/ ./configs/

# 安装可选库（失败不阻塞）
RUN pip install --no-cache-dir PyNaCl psutil 2>/dev/null || true

# 创建输出目录
RUN mkdir -p /data/sec-userspace/workspace

# root 用户运行（安全软件）
USER root

# 默认入口
ENTRYPOINT ["python3", "-m", "scripts.main"]
CMD ["--help"]
