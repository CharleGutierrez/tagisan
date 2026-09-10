# Nacos v2.x Open API 参考

本文档详列本 Skill 所使用的所有 Nacos Open API 端点及参数。

---

## 认证

### POST /v1/auth/login

用户登录，获取 accessToken。若 Nacos 未启用认证，此接口返回 404。

**请求（x-www-form-urlencoded）:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| username | String | 是 | 用户名 |
| password | String | 是 | 密码 |

**响应：**
```json
{
  "accessToken": "eyJhbGciOiJIUzI1NiJ9...",
  "tokenTtl": 18000,
  "globalAdmin": true,
  "username": "nacos"
}
```

accessToken 通过查询参数 `?accessToken=xxx` 附加到后续请求 URL 上。

---

## 命名空间

### GET /v1/console/namespaces — 查询命名空间列表

**响应：**
```json
{
  "code": 200,
  "data": [
    {
      "namespace": "",
      "namespaceShowName": "public",
      "namespaceDesc": null,
      "quota": 200,
      "configCount": 3,
      "type": 0
    }
  ]
}
```

### POST /v1/console/namespaces — 创建命名空间

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| customNamespaceId | String | 是 | 命名空间 ID |
| namespaceName | String | 是 | 命名空间名称 |
| namespaceDesc | String | 否 | 命名空间描述 |

### PUT /v1/console/namespaces — 修改命名空间

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| namespaceId | String | 是 | 命名空间 ID |
| namespaceName | String | 是 | 命名空间名称 |
| namespaceDesc | String | 否 | 命名空间描述 |

### DELETE /v1/console/namespaces — 删除命名空间

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| namespaceId | String | 是 | 命名空间 ID |

---

## 配置管理

### GET /v1/cs/configs — 获取配置

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| dataId | String | 是 | 配置 Data ID |
| group | String | 是 | 配置分组 |
| tenant | String | 否 | 命名空间 ID（public 留空） |

**响应：** 纯文本配置内容。

---

### GET /v1/cs/configs — 配置列表（分页+搜索）

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| search | String | 是 | `blur`（模糊）或 `accurate`（精确） |
| dataId | String | 否 | Data ID 过滤 |
| group | String | 否 | Group 过滤 |
| tenant | String | 否 | 命名空间 ID |
| pageNo | int | 是 | 页码（从 1 开始） |
| pageSize | int | 是 | 每页条数 |

**响应：**
```json
{
  "totalCount": 100,
  "pageNumber": 1,
  "pagesAvailable": 10,
  "pageItems": [
    {
      "id": "xxx",
      "dataId": "application.yaml",
      "group": "DEFAULT_GROUP",
      "content": "...",
      "md5": "...",
      "tenant": "",
      "appName": "",
      "type": "yaml"
    }
  ]
}
```

### POST /v1/cs/configs — 发布配置

**请求（x-www-form-urlencoded）：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| dataId | String | 是 | Data ID |
| group | String | 是 | 分组 |
| content | String | 是 | 配置内容 |
| type | String | 否 | 配置类型（text/json/xml/yaml/properties/html） |
| tenant | String | 否 | 命名空间 ID |

**响应：** `true` / `false`

### DELETE /v1/cs/configs — 删除配置

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| dataId | String | 是 | Data ID |
| group | String | 是 | 分组 |
| tenant | String | 否 | 命名空间 ID |

### GET /v1/cs/configs?export=true — 导出配置

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| export | String | 是 | 固定值 `true` |
| tenant | String | 否 | 过滤命名空间 |
| dataId | String | 否 | 过滤 Data ID |
| group | String | 否 | 过滤 Group |

**响应：** ZIP 二进制流。

### POST /v1/cs/configs?import=true — 导入配置

**请求（multipart/form-data）：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| import | String | 是 | 固定值 `true`（查询参数） |
| tenant | String | 否 | 目标命名空间（查询参数） |
| file | File | 是 | ZIP 文件 |

---

## 配置历史

### GET /v1/cs/history — 查询历史版本列表

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| dataId | String | 是 | Data ID |
| group | String | 是 | 分组 |
| tenant | String | 否 | 命名空间 ID |
| pageNo | int | 是 | 页码 |
| pageSize | int | 是 | 每页条数 |

**响应：**
```json
{
  "totalCount": 5,
  "pageNumber": 1,
  "pagesAvailable": 1,
  "pageItems": [
    {
      "id": "12345",
      "lastId": -1,
      "dataId": "app.yaml",
      "group": "DEFAULT_GROUP",
      "tenant": "",
      "appName": "",
      "md5": "...",
      "content": "...",
      "srcIp": "192.168.1.1",
      "srcUser": "nacos",
      "opType": "U",
      "createdTime": "2024-01-01T00:00:00.000+0800",
      "lastModifiedTime": "2024-01-01T00:00:00.000+0800"
    }
  ]
}
```

### GET /v1/cs/history (with nid) — 查询历史版本详情

在以上参数基础上增加 `nid`：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| nid | String | 是 | 历史版本 ID（从列表中的 `id` 字段获取） |

---

## 服务管理

### GET /v1/ns/catalog/services — 查询服务列表

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| pageNo | int | 是 | 页码 |
| pageSize | int | 是 | 每页条数 |
| namespaceId | String | 否 | 命名空间 ID |
| groupName | String | 否 | 分组名 |

**响应：**
```json
{
  "count": 10,
  "serviceList": ["service-1", "service-2"]
}
```

### GET /v1/ns/catalog/instances — 查询服务实例列表

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| serviceName | String | 是 | 服务名 |
| groupName | String | 是 | 分组名 |
| namespaceId | String | 否 | 命名空间 ID |

**响应：**
```json
{
  "name": "user-service",
  "groupName": "DEFAULT_GROUP",
  "protectThreshold": 0.0,
  "healthChecker": { "type": "TCP" },
  "list": [
    {
      "ip": "192.168.1.100",
      "port": 8080,
      "weight": 1.0,
      "healthy": true,
      "enabled": true,
      "ephemeral": true,
      "clusterName": "DEFAULT",
      "serviceName": "user-service",
      "metadata": {}
    }
  ]
}
```

### GET /v1/ns/service/subscribers — 查询订阅者列表

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| serviceName | String | 是 | 服务名 |
| groupName | String | 是 | 分组名 |
| namespaceId | String | 否 | 命名空间 ID |

**响应：**
```json
{
  "subscribers": [
    {
      "addrStr": "192.168.1.200:58894",
      "agent": "Nacos-Java-Client:v2.2.3",
      "app": "unknown",
      "ip": "192.168.1.200",
      "port": 0,
      "namespaceId": "",
      "serviceName": "user-service",
      "groupName": "DEFAULT_GROUP"
    }
  ]
}
```

---

## 系统

### GET /v1/console/health/readiness — 健康检查

**响应：**
```json
{"status": "UP"}
```

### GET /v1/console/server/state — 服务器状态

**响应：**
```json
{
  "version": "2.2.3",
  "nacosVersion": "2.2.3",
  "standalone_mode": "standalone",
  "auth_enabled": false,
  "server_state": "UP",
  "function_mode": ""
}
```
