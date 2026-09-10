#!/usr/bin/env python3
"""
Nacos v2.x 操作工具 — 覆盖配置中心与注册中心所有控制面操作。

配置读取优先级（从高到低）：
  1. 命令行显式传入参数
  2. 环境变量 (NACOS_SERVER_ADDR, NACOS_USERNAME, NACOS_PASSWORD, NACOS_NAMESPACE)
  3. Java 系统属性 (-Dnacos.server_addr 等)
  4. 当前目录 application.yaml / application.yml (前缀 nacos)
  5. 当前目录 bootstrap.yaml / bootstrap.yml (前缀 nacos)
  6. 默认值 http://localhost:8848

所有写操作需要认证；读操作在未启用认证的 Nacos 上可直接调用。
"""

import argparse
import json
import os
import re
import sys
import io
import zipfile
from pathlib import Path
from typing import Optional, Dict, Any, Tuple

import requests

# ---------------------------------------------------------------------------
# 常量
# ---------------------------------------------------------------------------
DEFAULT_SERVER = "http://localhost:8848"
DEFAULT_GROUP = "DEFAULT_GROUP"
CONFIG_KEYS = ["server_addr", "username", "password", "namespace"]

# ---------------------------------------------------------------------------
# 配置解析
# ---------------------------------------------------------------------------


def _try_parse_yaml(content: str) -> Dict[str, Any]:
    """尝试用 PyYAML 解析；不可用时回退到简易正则解析。"""
    try:
        import yaml  # type: ignore
        return yaml.safe_load(content) or {}
    except ImportError:
        pass

    # 简易解析：只提取 nacos 前缀下的标量值
    result: Dict[str, Any] = {}
    current_path: list = []
    for line in content.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        indent = len(line) - len(line.lstrip())
        level = indent // 2
        current_path = current_path[:level]
        if ":" in stripped:
            key, _, val = stripped.partition(":")
            key = key.strip()
            val = val.strip().strip('"').strip("'")
            current_path.append(key)
            if val:
                _set_nested(result, list(current_path), val)
    return result


def _set_nested(d: dict, path: list, value: str) -> None:
    for k in path[:-1]:
        d = d.setdefault(k, {})
    d[path[-1]] = value


def _read_yaml_file(filepath: Path) -> Optional[Dict[str, Any]]:
    """读取单个 YAML 文件，不存在返回 None。"""
    if not filepath.exists():
        return None
    content = filepath.read_text(encoding="utf-8")
    return _try_parse_yaml(content)


def _extract_nacos_config(yaml_data: Dict[str, Any]) -> Dict[str, str]:
    """从解析后的 YAML 数据中提取 nacos 前缀配置。"""
    nacos_block = yaml_data.get("nacos", {})
    if not isinstance(nacos_block, dict):
        return {}
    result = {}
    for key in CONFIG_KEYS:
        val = nacos_block.get(key)
        if val is not None:
            result[key] = str(val)
    return result


def _parse_java_props() -> Dict[str, str]:
    """从命令行参数中提取 -Dnacos.xxx=yyy 形式的 Java 系统属性。"""
    result = {}
    for arg in sys.argv:
        m = re.match(r"-Dnacos\.(\w+)=(.+)", arg)
        if m:
            key = m.group(1)
            val = m.group(2)
            if key in CONFIG_KEYS and key not in result:  # 第一个优先
                result[key] = val
    return result


def resolve_config(explicit: Optional[Dict[str, str]] = None) -> Dict[str, str]:
    """按优先级合并所有配置源，返回 dict(server_addr, username, password, namespace)。

    优先级：explicit > 环境变量 > Java 系统属性 > application.yaml > bootstrap.yaml > 默认值。
    """
    config: Dict[str, str] = {}

    # 6. 默认值
    config["server_addr"] = DEFAULT_SERVER

    # 5. bootstrap.yaml / bootstrap.yml
    cwd = Path.cwd()
    for name in ("bootstrap.yaml", "bootstrap.yml"):
        data = _read_yaml_file(cwd / name)
        if data:
            nc = _extract_nacos_config(data)
            for k, v in nc.items():
                config[k] = v

    # 4. application.yaml / application.yml
    for name in ("application.yaml", "application.yml"):
        data = _read_yaml_file(cwd / name)
        if data:
            nc = _extract_nacos_config(data)
            for k, v in nc.items():
                config[k] = v

    # 3. Java 系统属性
    java_props = _parse_java_props()
    for k, v in java_props.items():
        config[k] = v

    # 2. 环境变量
    env_mapping = {
        "server_addr": "NACOS_SERVER_ADDR",
        "username": "NACOS_USERNAME",
        "password": "NACOS_PASSWORD",
        "namespace": "NACOS_NAMESPACE",
    }
    for key, env_var in env_mapping.items():
        val = os.environ.get(env_var)
        if val:
            config[key] = val

    # 1. 显式传入参数（最高优先级）
    if explicit:
        for k, v in explicit.items():
            if v:
                config[k] = v

    return config


# ---------------------------------------------------------------------------
# Auth
# ---------------------------------------------------------------------------

_token_cache: Dict[str, str] = {}


def login(server_addr: str, username: str, password: str) -> Optional[str]:
    """登录 Nacos 并返回 accessToken；认证未启用时返回 None。"""
    cache_key = f"{server_addr}:{username}"
    if cache_key in _token_cache:
        return _token_cache[cache_key]

    if not username or not password:
        return None

    url = f"{server_addr}/v1/auth/login"
    try:
        resp = requests.post(
            url,
            data={"username": username, "password": password},
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            timeout=10,
        )
    except requests.RequestException:
        return None

    if resp.status_code == 200:
        data = resp.json()
        token = data.get("accessToken")
        if token:
            _token_cache[cache_key] = token
            return token

    # 可能是未启用认证的 Nacos
    if resp.status_code == 404:
        return None

    return None


def _add_token(url: str, token: Optional[str]) -> str:
    """在 URL 上附加 accessToken 查询参数。"""
    if not token:
        return url
    sep = "&" if "?" in url else "?"
    return f"{url}{sep}accessToken={token}"


# ---------------------------------------------------------------------------
# 命名空间
# ---------------------------------------------------------------------------


def list_namespaces(server_addr: str, token: Optional[str] = None) -> list:
    """查询所有命名空间。"""
    url = _add_token(f"{server_addr}/v1/console/namespaces", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    data = resp.json()
    return data.get("data", [])


def create_namespace(
    server_addr: str,
    namespace_id: str,
    namespace_name: str,
    namespace_desc: str = "",
    token: Optional[str] = None,
) -> bool:
    """创建命名空间。"""
    url = _add_token(f"{server_addr}/v1/console/namespaces", token)
    resp = requests.post(
        url,
        data={
            "customNamespaceId": namespace_id,
            "namespaceName": namespace_name,
            "namespaceDesc": namespace_desc,
        },
        timeout=10,
    )
    resp.raise_for_status()
    return resp.json().get("code", -1) == 200


def update_namespace(
    server_addr: str,
    namespace_id: str,
    namespace_name: str,
    namespace_desc: str = "",
    token: Optional[str] = None,
) -> bool:
    """修改命名空间（名称和描述）。"""
    url = _add_token(f"{server_addr}/v1/console/namespaces", token)
    resp = requests.put(
        url,
        data={
            "namespaceId": namespace_id,
            "namespaceName": namespace_name,
            "namespaceDesc": namespace_desc,
        },
        timeout=10,
    )
    resp.raise_for_status()
    return resp.json().get("code", -1) == 200


def delete_namespace(
    server_addr: str, namespace_id: str, token: Optional[str] = None
) -> bool:
    """删除命名空间。"""
    url = _add_token(
        f"{server_addr}/v1/console/namespaces?namespaceId={namespace_id}", token
    )
    resp = requests.delete(url, timeout=10)
    resp.raise_for_status()
    return resp.json().get("code", -1) == 200


# ---------------------------------------------------------------------------
# 配置
# ---------------------------------------------------------------------------


def get_config(
    server_addr: str,
    data_id: str,
    group: str = DEFAULT_GROUP,
    tenant: str = "",
    token: Optional[str] = None,
) -> str:
    """获取指定配置的内容。"""
    params = f"dataId={data_id}&group={group}"
    if tenant:
        params += f"&tenant={tenant}"
    url = _add_token(f"{server_addr}/v1/cs/configs?{params}", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    return resp.text


def publish_config(
    server_addr: str,
    data_id: str,
    group: str,
    content: str,
    config_type: str = "text",
    tenant: str = "",
    token: Optional[str] = None,
) -> bool:
    """发布（创建或更新）配置。"""
    url = _add_token(f"{server_addr}/v1/cs/configs", token)
    resp = requests.post(
        url,
        data={
            "dataId": data_id,
            "group": group,
            "content": content,
            "type": config_type,
            "tenant": tenant,
        },
        timeout=10,
    )
    resp.raise_for_status()
    return resp.json().get("code", -1) == 200


def delete_config(
    server_addr: str,
    data_id: str,
    group: str = DEFAULT_GROUP,
    tenant: str = "",
    token: Optional[str] = None,
) -> bool:
    """删除配置。"""
    params = f"dataId={data_id}&group={group}"
    if tenant:
        params += f"&tenant={tenant}"
    url = _add_token(f"{server_addr}/v1/cs/configs?{params}", token)
    resp = requests.delete(url, timeout=10)
    resp.raise_for_status()
    return resp.json().get("code", -1) == 200


def list_configs(
    server_addr: str,
    tenant: str = "",
    data_id: str = "",
    group: str = "",
    page_no: int = 1,
    page_size: int = 10,
    search: str = "blur",
    token: Optional[str] = None,
) -> dict:
    """配置列表查询（分页+模糊过滤）。

    search 参数: "blur"=模糊搜索 / "accurate"=精确搜索。
    """
    # Nacos 2.4.x 要求 dataId 和 group 参数必须存在（即使是空字符串）
    params = (
        f"pageNo={page_no}&pageSize={page_size}&search={search}"
        f"&dataId={data_id}&group={group}"
    )
    if tenant:
        params += f"&tenant={tenant}"
    url = _add_token(f"{server_addr}/v1/cs/configs?{params}", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    result = resp.json()
    return {
        "total_count": result.get("totalCount", 0),
        "page_number": result.get("pageNumber", page_no),
        "pages_available": result.get("pagesAvailable", 0),
        "items": result.get("pageItems", []),
    }


def export_configs(
    server_addr: str,
    tenant: str = "",
    data_id: str = "",
    group: str = "",
    output_file: str = "nacos_configs.zip",
    token: Optional[str] = None,
) -> str:
    """将指定配置导出为 Zip 文件。返回文件路径。"""
    # Nacos 2.4.x 要求 dataId 和 group 参数必须存在（即使是空字符串）
    params = f"export=true&dataId={data_id}&group={group}"
    if tenant:
        params += f"&tenant={tenant}"
    url = _add_token(f"{server_addr}/v1/cs/configs?{params}", token)
    resp = requests.get(url, timeout=30)
    resp.raise_for_status()
    with open(output_file, "wb") as f:
        f.write(resp.content)
    return output_file


def import_configs(
    server_addr: str,
    zip_file_path: str,
    tenant: str = "",
    token: Optional[str] = None,
) -> dict:
    """从 Zip 文件导入配置到指定命名空间。"""
    params = "import=true"
    if tenant:
        params += f"&tenant={tenant}"
    url = _add_token(f"{server_addr}/v1/cs/configs?{params}", token)
    with open(zip_file_path, "rb") as f:
        resp = requests.post(
            url,
            files={"file": (os.path.basename(zip_file_path), f)},
            timeout=30,
        )
    resp.raise_for_status()
    try:
        return resp.json()
    except ValueError:
        return {"raw": resp.text}


# ---------------------------------------------------------------------------
# 配置历史
# ---------------------------------------------------------------------------


def list_history(
    server_addr: str,
    data_id: str,
    group: str = DEFAULT_GROUP,
    tenant: str = "",
    page_no: int = 1,
    page_size: int = 10,
    token: Optional[str] = None,
) -> dict:
    """查询配置的历史版本列表。"""
    # Nacos 2.4.x 需要 search=accurate 以区分列表查询和详情查询（详情查询需要 nid 参数）
    params = (
        f"search=accurate&dataId={data_id}&group={group}"
        f"&pageNo={page_no}&pageSize={page_size}"
    )
    if tenant:
        params += f"&tenant={tenant}"
    url = _add_token(f"{server_addr}/v1/cs/history?{params}", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    result = resp.json()
    return {
        "total_count": result.get("totalCount", 0),
        "page_number": result.get("pageNumber", page_no),
        "pages_available": result.get("pagesAvailable", 0),
        "items": result.get("pageItems", []),
    }


def get_history_detail(
    server_addr: str,
    data_id: str,
    group: str,
    tenant: str,
    nid: str,
    token: Optional[str] = None,
) -> dict:
    """查询指定历史版本的完整信息（含配置内容）。"""
    params = f"dataId={data_id}&group={group}&nid={nid}"
    if tenant:
        params += f"&tenant={tenant}"
    url = _add_token(f"{server_addr}/v1/cs/history?{params}", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    return resp.json()


def rollback_config(
    server_addr: str,
    data_id: str,
    group: str,
    tenant: str,
    nid: str,
    token: Optional[str] = None,
) -> bool:
    """回滚配置到指定历史版本：先获取历史版本内容，再重新发布。"""
    detail = get_history_detail(server_addr, data_id, group, tenant, nid, token)
    content = detail.get("content", "")
    # 历史接口返回的 type 可能是 configType 也可能是 type
    config_type = detail.get("configType") or detail.get("type", "text")
    if not content:
        print("错误：历史版本内容为空，无法回滚。", file=sys.stderr)
        return False
    return publish_config(server_addr, data_id, group, content, config_type, tenant, token)


# ---------------------------------------------------------------------------
# 服务管理
# ---------------------------------------------------------------------------


def list_services(
    server_addr: str,
    namespace_id: str = "",
    group_name: str = "",
    page_no: int = 1,
    page_size: int = 10,
    token: Optional[str] = None,
) -> dict:
    """查询服务列表（通过 catalog 接口）。"""
    params = f"pageNo={page_no}&pageSize={page_size}"
    if namespace_id:
        params += f"&namespaceId={namespace_id}"
    if group_name:
        params += f"&groupName={group_name}"
    url = _add_token(f"{server_addr}/v1/ns/catalog/services?{params}", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    result = resp.json()
    return {
        "count": result.get("count", 0),
        "services": result.get("serviceList", []),
    }


def get_service_detail(
    server_addr: str,
    service_name: str,
    group_name: str = DEFAULT_GROUP,
    namespace_id: str = "",
    token: Optional[str] = None,
) -> dict:
    """查询服务详情（实例列表）。

    使用 /v1/ns/instance/list 端点（兼容 Nacos 2.x），而非 /v1/ns/catalog/instances。
    """
    params = f"serviceName={service_name}&groupName={group_name}"
    if namespace_id:
        params += f"&namespaceId={namespace_id}"
    url = _add_token(f"{server_addr}/v1/ns/instance/list?{params}", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    result = resp.json()
    return {
        "service_name": result.get("name", service_name),
        "group_name": result.get("groupName", group_name),
        "protect_threshold": result.get("reachProtectionThreshold", False),
        "valid": result.get("valid", True),
        "instances": result.get("hosts", []),
    }


def get_subscribers(
    server_addr: str,
    service_name: str,
    group_name: str = DEFAULT_GROUP,
    namespace_id: str = "",
    token: Optional[str] = None,
) -> list:
    """查询服务的订阅者列表。"""
    params = f"serviceName={service_name}&groupName={group_name}"
    if namespace_id:
        params += f"&namespaceId={namespace_id}"
    url = _add_token(f"{server_addr}/v1/ns/service/subscribers?{params}", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    data = resp.json()
    return data.get("subscribers", [])


# ---------------------------------------------------------------------------
# 系统
# ---------------------------------------------------------------------------


def health_check(server_addr: str) -> dict:
    """Nacos 健康检查。"""
    url = f"{server_addr}/v1/console/health/readiness"
    resp = requests.get(url, timeout=5)
    resp.raise_for_status()
    return resp.json()


def get_server_state(server_addr: str, token: Optional[str] = None) -> dict:
    """获取服务器状态 / 系统开关信息。"""
    url = _add_token(f"{server_addr}/v1/console/server/state", token)
    resp = requests.get(url, timeout=10)
    resp.raise_for_status()
    return resp.json()


# ===========================================================================
# CLI
# ===========================================================================


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Nacos v2.x 管理工具",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
使用示例：
  # 查询命名空间列表
  python nacos_ops.py namespace list --server-addr http://nacos:8848

  # 获取配置内容
  python nacos_ops.py config get --data-id application-dev.yaml --group DEFAULT_GROUP

  # 发布配置
  python nacos_ops.py config publish --data-id app.yaml --content "..."
        """,
    )
    # 全局连接参数
    for opt, env_var, help_text in [
        ("--server-addr", "NACOS_SERVER_ADDR", "Nacos 服务地址"),
        ("--username", "NACOS_USERNAME", "用户名"),
        ("--password", "NACOS_PASSWORD", "密码"),
        ("--namespace", "NACOS_NAMESPACE", "默认命名空间 ID"),
    ]:
        parser.add_argument(opt, default=None, help=f"{help_text}（默认从 ${env_var} 读取）")

    sub = parser.add_subparsers(dest="command", required=True)

    # ---- namespace ----
    ns = sub.add_parser("namespace", help="命名空间管理")
    ns_sub = ns.add_subparsers(dest="ns_action", required=True)
    ns_sub.add_parser("list", help="查询命名空间列表")
    ns_create = ns_sub.add_parser("create", help="创建命名空间")
    ns_create.add_argument("--id", required=True, help="命名空间 ID")
    ns_create.add_argument("--name", required=True, help="命名空间名称")
    ns_create.add_argument("--desc", default="", help="命名空间描述")
    ns_update = ns_sub.add_parser("update", help="修改命名空间")
    ns_update.add_argument("--id", required=True, help="命名空间 ID")
    ns_update.add_argument("--name", required=True, help="命名空间新名称")
    ns_update.add_argument("--desc", default="", help="命名空间新描述")
    ns_delete = ns_sub.add_parser("delete", help="删除命名空间")
    ns_delete.add_argument("--id", required=True, help="命名空间 ID")

    # ---- config ----
    cfg = sub.add_parser("config", help="配置管理")
    cfg_sub = cfg.add_subparsers(dest="cfg_action", required=True)

    cfg_get = cfg_sub.add_parser("get", help="获取配置")
    cfg_get.add_argument("--data-id", required=True)
    cfg_get.add_argument("--group", default=DEFAULT_GROUP)
    cfg_get.add_argument("--tenant", default="")

    cfg_list = cfg_sub.add_parser("list", help="配置列表")
    cfg_list.add_argument("--data-id", default="")
    cfg_list.add_argument("--group", default="")
    cfg_list.add_argument("--tenant", default="")
    cfg_list.add_argument("--page-no", type=int, default=1)
    cfg_list.add_argument("--page-size", type=int, default=10)
    cfg_list.add_argument("--search", default="blur", choices=["blur", "accurate"])

    cfg_pub = cfg_sub.add_parser("publish", help="发布配置")
    cfg_pub.add_argument("--data-id", required=True)
    cfg_pub.add_argument("--group", default=DEFAULT_GROUP)
    cfg_pub.add_argument("--tenant", default="")
    cfg_pub.add_argument("--content", default="", help="配置内容；或用 --content-file 从文件读取")
    cfg_pub.add_argument("--content-file", default="", help="从文件读取配置内容，与 --content 二选一")
    cfg_pub.add_argument("--type", default="text", help="配置类型（text/json/xml/yaml/properties/html）")

    cfg_del = cfg_sub.add_parser("delete", help="删除配置")
    cfg_del.add_argument("--data-id", required=True)
    cfg_del.add_argument("--group", default=DEFAULT_GROUP)
    cfg_del.add_argument("--tenant", default="")

    cfg_exp = cfg_sub.add_parser("export", help="导出配置（Zip）")
    cfg_exp.add_argument("--tenant", default="")
    cfg_exp.add_argument("--data-id", default="")
    cfg_exp.add_argument("--group", default="")
    cfg_exp.add_argument("--output-file", default="nacos_configs.zip")

    cfg_imp = cfg_sub.add_parser("import", help="导入配置（Zip）")
    cfg_imp.add_argument("--zip-file", required=True)
    cfg_imp.add_argument("--tenant", default="")

    cfg_hist = cfg_sub.add_parser("history", help="查询配置历史版本")
    cfg_hist.add_argument("--data-id", required=True)
    cfg_hist.add_argument("--group", default=DEFAULT_GROUP)
    cfg_hist.add_argument("--tenant", default="")
    cfg_hist.add_argument("--page-no", type=int, default=1)
    cfg_hist.add_argument("--page-size", type=int, default=10)

    cfg_rollback = cfg_sub.add_parser("rollback", help="回滚配置到指定历史版本")
    cfg_rollback.add_argument("--data-id", required=True)
    cfg_rollback.add_argument("--group", default=DEFAULT_GROUP)
    cfg_rollback.add_argument("--tenant", default="")
    cfg_rollback.add_argument("--nid", required=True, help="历史版本 ID")

    # ---- service ----
    svc = sub.add_parser("service", help="服务管理")
    svc_sub = svc.add_subparsers(dest="svc_action", required=True)
    svc_list = svc_sub.add_parser("list", help="服务列表")
    svc_list.add_argument("--namespace-id", default="")
    svc_list.add_argument("--group-name", default="")
    svc_list.add_argument("--page-no", type=int, default=1)
    svc_list.add_argument("--page-size", type=int, default=10)
    svc_detail = svc_sub.add_parser("detail", help="服务详情")
    svc_detail.add_argument("--name", required=True, help="服务名")
    svc_detail.add_argument("--group-name", default=DEFAULT_GROUP)
    svc_detail.add_argument("--namespace-id", default="")
    svc_subers = svc_sub.add_parser("subscribers", help="查询订阅者")
    svc_subers.add_argument("--name", required=True, help="服务名")
    svc_subers.add_argument("--group-name", default=DEFAULT_GROUP)
    svc_subers.add_argument("--namespace-id", default="")

    # ---- system ----
    sys_ = sub.add_parser("system", help="系统操作")
    sys_sub = sys_.add_subparsers(dest="sys_action", required=True)
    sys_sub.add_parser("health", help="健康检查")
    sys_sub.add_parser("switches", help="获取系统开关 / 服务器状态")

    return parser


def main():
    parser = _build_parser()
    args = parser.parse_args()

    # 解析配置
    explicit = {
        "server_addr": args.server_addr,
        "username": args.username,
        "password": args.password,
        "namespace": args.namespace,
    }
    cfg = resolve_config(explicit)
    server_addr = cfg["server_addr"].rstrip("/")
    username = cfg.get("username", "")
    password = cfg.get("password", "")
    namespace = cfg.get("namespace", "")
    token = login(server_addr, username, password) if username else None

    def _tenant(override: str = "") -> str:
        return override or namespace

    try:
        # ========== namespace ==========
        if args.command == "namespace":
            if args.ns_action == "list":
                print(json.dumps(list_namespaces(server_addr, token), indent=2, ensure_ascii=False))
            elif args.ns_action == "create":
                ok = create_namespace(server_addr, args.id, args.name, args.desc, token)
                print(json.dumps({"success": ok, "action": "create", "id": args.id}))
            elif args.ns_action == "update":
                ok = update_namespace(server_addr, args.id, args.name, args.desc, token)
                print(json.dumps({"success": ok, "action": "update", "id": args.id}))
            elif args.ns_action == "delete":
                ok = delete_namespace(server_addr, args.id, token)
                print(json.dumps({"success": ok, "action": "delete", "id": args.id}))

        # ========== config ==========
        elif args.command == "config":
            if args.cfg_action == "get":
                print(get_config(server_addr, args.data_id, args.group, _tenant(args.tenant), token))
            elif args.cfg_action == "list":
                result = list_configs(
                    server_addr,
                    _tenant(args.tenant),
                    args.data_id,
                    args.group,
                    args.page_no,
                    args.page_size,
                    args.search,
                    token,
                )
                print(json.dumps(result, indent=2, ensure_ascii=False))
            elif args.cfg_action == "publish":
                content = args.content
                if args.content_file:
                    content = Path(args.content_file).read_text(encoding="utf-8")
                ok = publish_config(
                    server_addr, args.data_id, args.group, content, args.type, _tenant(args.tenant), token
                )
                print(json.dumps({"success": ok, "action": "publish", "data_id": args.data_id}))
            elif args.cfg_action == "delete":
                ok = delete_config(server_addr, args.data_id, args.group, _tenant(args.tenant), token)
                print(json.dumps({"success": ok, "action": "delete", "data_id": args.data_id}))
            elif args.cfg_action == "export":
                path = export_configs(server_addr, _tenant(args.tenant), args.data_id, args.group, args.output_file, token)
                print(json.dumps({"success": True, "output_file": os.path.abspath(path)}))
            elif args.cfg_action == "import":
                result = import_configs(server_addr, args.zip_file, _tenant(args.tenant), token)
                print(json.dumps(result, indent=2, ensure_ascii=False))
            elif args.cfg_action == "history":
                result = list_history(
                    server_addr, args.data_id, args.group, _tenant(args.tenant), args.page_no, args.page_size, token
                )
                print(json.dumps(result, indent=2, ensure_ascii=False))
            elif args.cfg_action == "rollback":
                ok = rollback_config(server_addr, args.data_id, args.group, _tenant(args.tenant), args.nid, token)
                print(json.dumps({"success": ok, "action": "rollback", "data_id": args.data_id, "nid": args.nid}))

        # ========== service ==========
        elif args.command == "service":
            if args.svc_action == "list":
                result = list_services(
                    server_addr, args.namespace_id or namespace, args.group_name, args.page_no, args.page_size, token
                )
                print(json.dumps(result, indent=2, ensure_ascii=False))
            elif args.svc_action == "detail":
                result = get_service_detail(
                    server_addr, args.name, args.group_name, args.namespace_id or namespace, token
                )
                print(json.dumps(result, indent=2, ensure_ascii=False))
            elif args.svc_action == "subscribers":
                result = get_subscribers(
                    server_addr, args.name, args.group_name, args.namespace_id or namespace, token
                )
                print(json.dumps(result, indent=2, ensure_ascii=False))

        # ========== system ==========
        elif args.command == "system":
            if args.sys_action == "health":
                print(json.dumps(health_check(server_addr), indent=2, ensure_ascii=False))
            elif args.sys_action == "switches":
                print(json.dumps(get_server_state(server_addr, token), indent=2, ensure_ascii=False))

    except requests.RequestException as e:
        print(f"请求失败: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
