---
name: nacos-manager
description: "Nacos \u914D\u7F6E\u4E2D\u5FC3\u548C\u6CE8\u518C\u4E2D\u5FC3\u7BA1\u7406\u5DE5\u5177\
  \u3002\u5F53\u7528\u6237\u9700\u8981\u64CD\u4F5C Nacos\uFF08\u5305\u62EC\u7BA1\u7406\
  \u547D\u540D\u7A7A\u95F4\u3001\u8BFB\u53D6/\u53D1\u5E03/\u5220\u9664/\u5BFC\u51FA\
  /\u5BFC\u5165\u914D\u7F6E\u3001\u67E5\u770B\u914D\u7F6E\u5386\u53F2\u5E76\u56DE\u6EDA\
  \u3001\u67E5\u8BE2\u670D\u52A1\u5217\u8868/\u8BE6\u60C5/\u8BA2\u9605\u8005\u3001\
  \u5065\u5EB7\u68C0\u67E5\uFF09\u65F6\u4F7F\u7528\u6B64 Skill\u3002\u8986\u76D6 Nacos\
  \ \u63A7\u5236\u53F0\u6240\u6709\u5E38\u7528\u64CD\u4F5C\uFF0C\u652F\u6301\u591A\
  \u6E90\u914D\u7F6E\u8BFB\u53D6\uFF08\u663E\u5F0F\u53C2\u6570 > \u73AF\u5883\u53D8\
  \u91CF > Java \u5C5E\u6027 > application.yaml > bootstrap.yaml > \u9ED8\u8BA4\u503C\
  \uFF09\u3002"
---
# Nacos 管理 Skill

通过 Nacos v2.x Open API，对配置中心和注册中心执行全量控制面操作。

## 前置条件

- Python 3.8+，`requests` 库已安装
- Nacos 服务端点可访问（默认 `http://localhost:8848`）

## 配置解析

连接信息按以下优先级（从高到低）自动合并：

| 优先级 | 来源 | 说明 |
|--------|------|------|
| 1 | 命令行参数 | `--server-addr`, `--username`, `--password`, `--namespace` |
| 2 | 环境变量 | `NACOS_SERVER_ADDR`, `NACOS_USERNAME`, `NACOS_PASSWORD`, `NACOS_NAMESPACE` |
| 3 | Java 系统属性 | `-Dnacos.server_addr=...` 等 |
| 4 | application.yaml / application.yml | 前缀 `nacos:` |
| 5 | bootstrap.yaml / bootstrap.yml | 前缀 `nacos:` |
| 6 | 默认值 | `http://localhost:8848` |

YAML 配置示例：
```yaml
nacos:
  server_addr: http://nacos.internal:8848/nacos   # 若配置了 contextPath，直接写入地址
  username: nacos
  password: nacos123
  namespace: dev
```

> **关于上下文路径**：如果 Nacos 使用了自定义 `contextPath`（如 `/nacos`），直接在 `server_addr` 中包含路径即可，例如 `http://host:8848/nacos`。脚本将所有请求拼接到该基础地址之后。

当复用了上述配置源时，调用脚本时可省略对应参数；仅在需要覆盖时显式传入。

## 一切操作通过捆绑脚本执行

**所有操作的统一入口是 `scripts/nacos_ops.py`。** 每次调用时，首先读取此脚本确认所需子命令和参数，然后执行。

脚本路径：`scripts/nacos_ops.py`（相对于本 SKILL.md 所在目录）

### 认证

脚本会自动处理认证：
- 若提供了 `username` 和 `password`，调用 `POST /v1/auth/login` 获取 accessToken，并缓存在进程中
- 后续所有请求自动附带 `accessToken` 查询参数
- 若 Nacos 未启用认证（登录接口返回 404），自动跳过令牌附加

### 通用模式

```bash
python scripts/nacos_ops.py <domain> <action> [--key value ...] [--server-addr ...] [--username ...] [--password ...] [--namespace ...]
```

`--server-addr`、`--username`、`--password`、`--namespace` 四个全局参数对所有子命令都有效。

## 操作速查

### 命名空间（namespace）

```bash
# 列表
python scripts/nacos_ops.py namespace list

# 创建
python scripts/nacos_ops.py namespace create --id <id> --name <名称> [--desc <描述>]

# 修改
python scripts/nacos_ops.py namespace update --id <id> --name <新名称> [--desc <新描述>]

# 删除
python scripts/nacos_ops.py namespace delete --id <id>
```

### 配置（config）

```bash
# 获取配置内容
python scripts/nacos_ops.py config get --data-id <dataId> [--group <group>] [--tenant <namespaceId>]

# 发布（创建/更新）— 直接传入内容
python scripts/nacos_ops.py config publish --data-id <dataId> --content "<内容>" [--type yaml|json|xml|text|properties|html]

# 发布 — 从文件读取内容
python scripts/nacos_ops.py config publish --data-id <dataId> --content-file ./app.yaml [--type yaml]

# 列表查询（分页+模糊过滤）
python scripts/nacos_ops.py config list [--data-id <关键字>] [--group <group>] [--tenant <tenant>] \
    [--page-no 1] [--page-size 10] [--search blur|accurate]

# 删除
python scripts/nacos_ops.py config delete --data-id <dataId> [--group <group>] [--tenant <tenant>]

# 导出配置到 Zip 文件
python scripts/nacos_ops.py config export [--tenant <tenant>] [--data-id <过滤>] [--group <过滤>] \
    [--output-file nacos_backup.zip]

# 从 Zip 文件导入配置
python scripts/nacos_ops.py config import --zip-file <path/to/file.zip> [--tenant <tenant>]
```

### 配置历史（config history）

```bash
# 查询历史版本列表
python scripts/nacos_ops.py config history --data-id <dataId> [--group <group>] [--tenant <tenant>] \
    [--page-no 1] [--page-size 10]

# 回滚到指定历史版本（自动获取历史内容并重新发布）
python scripts/nacos_ops.py config rollback --data-id <dataId> --nid <历史ID> [--group <group>] [--tenant <tenant>]
```

### 服务（service）

```bash
# 服务列表
python scripts/nacos_ops.py service list [--namespace-id <id>] [--group-name <group>] \
    [--page-no 1] [--page-size 10]

# 服务详情（含实例列表）
python scripts/nacos_ops.py service detail --name <服务名> [--group-name <group>] [--namespace-id <id>]

# 订阅者列表
python scripts/nacos_ops.py service subscribers --name <服务名> [--group-name <group>] [--namespace-id <id>]
```

### 系统（system）

```bash
# 健康检查
python scripts/nacos_ops.py system health

# 系统开关 / 服务器状态
python scripts/nacos_ops.py system switches
```

## 典型使用场景

### 场景 1：查看开发环境所有配置

```bash
python scripts/nacos_ops.py config list --tenant dev --page-size 50 \
    --server-addr http://nacos-dev:8848 --username admin --password admin
```

### 场景 2：从 YAML 文件发布配置到指定命名空间

```bash
python scripts/nacos_ops.py config publish \
    --data-id application-dev.yaml \
    --group DEFAULT_GROUP \
    --tenant dev \
    --content-file ./config/application-dev.yaml \
    --type yaml
```

### 场景 3：批量导出 + 备份后导入到另一个命名空间

```bash
# 导出 dev 命名空间的所有配置
python scripts/nacos_ops.py config export --tenant dev --output-file dev_backup.zip

# 导入到 staging 命名空间
python scripts/nacos_ops.py config import --zip-file dev_backup.zip --tenant staging
```

### 场景 4：配置回滚

```bash
# 先查历史
python scripts/nacos_ops.py config history --data-id app.yaml --tenant prod

# 找到目标版本的 nid（如 12345），回滚
python scripts/nacos_ops.py config rollback --data-id app.yaml --nid 12345 --tenant prod
```

### 场景 5：检查服务健康

```bash
# 系统级健康检查
python scripts/nacos_ops.py system health

# 查看某个服务的实例列表
python scripts/nacos_ops.py service detail --name user-service --namespace-id prod
```

## 输出格式

- 所有操作结果以 JSON 格式输出（除非是 `config get`，直接输出配置原文）
- 成功/失败通过 JSON 中的 `success` 字段或 HTTP 状态码判断
- 列表类操作包含分页信息（`total_count`, `page_number`, `pages_available`）

## 错误处理

脚本在遇到网络错误或 HTTP 错误时输出错误信息到 stderr 并以退出码 1 退出。调用时根据退出码和 stderr 内容判断是否需要重试或提示用户检查连接参数。
