# Hosts 域名映射 & Profile 场景功能

## 概述

EnvTools v0.2 新增两大功能：
1. **Hosts 域名映射组** — 管理系统 hosts 文件条目，按组启用/禁用
2. **Profile 场景** — 将多个组绑定为场景，一键激活整套环境

## Hosts 域名映射

### 组类型

创建组时通过 `--kind` 参数指定类型：
- `env`（默认）— 环境变量组
- `hosts` — 域名映射组

```bash
envtools group create local-services --kind hosts -d "本地微服务域名"
```

### 管理 Hosts 条目

```bash
# 添加映射
envtools hosts add local-services 127.0.0.1 api.local
envtools hosts add local-services 127.0.0.1 auth.local
envtools hosts add local-services 10.0.0.5 db.local

# 删除映射
envtools hosts remove local-services auth.local
```

### 同步到系统

启用 hosts 组后，执行 sync 写入系统 hosts 文件：

```bash
envtools enable local-services
envtools hosts sync
```

**注意**：sync 操作需要管理员权限，系统会弹出 UAC/sudo 提示。

### 标记块管理

EnvTools 使用标记块方式管理 hosts 文件，不会影响用户手写条目：

```
# 用户原有条目不受影响
127.0.0.1 localhost

# >>> envtools managed >>>
127.0.0.1 api.local
127.0.0.1 auth.local
10.0.0.5 db.local
# <<< envtools managed <<<
```

### Hosts 文件位置

| 平台 | 路径 |
|------|------|
| Windows | `C:\Windows\System32\drivers\etc\hosts` |
| Linux/macOS | `/etc/hosts` |

---

## Profile 场景

### 概念

Profile 是一组关联组的集合。激活 Profile 会启用其中所有组（叠加模式，不影响其他已激活组）。

### CLI 操作

```bash
# 创建场景
envtools profile create fullstack-dev -d "全栈开发环境" -g nodejs-env,local-dns,python-ml

# 查看场景
envtools profile list
envtools profile show fullstack-dev

# 激活（启用所有关联组）
envtools profile activate fullstack-dev

# 停用（禁用所有关联组）
envtools profile deactivate fullstack-dev

# 删除
envtools profile delete fullstack-dev
```

### GUI 操作

左侧栏顶部的 **Profiles** 区域：
- 展开/收起 Profile 列表
- 点击 **Activate** 按钮激活场景
- 点击 **Deactivate** 按钮停用场景
- 点击 **+ Create Profile** 创建新场景

---

## 提权机制

修改 hosts 文件需要管理员权限。EnvTools 通过以下方式提权：

| 平台 | 方式 |
|------|------|
| Windows | `Start-Process -Verb RunAs`（弹出 UAC） |
| Linux | `pkexec`（弹出 polkit 认证） |
| macOS | `osascript` with administrator privileges |

如果权限不足，CLI 会返回 `ElevationRequired` 错误并提示用户以管理员身份重试。

---

## 配置文件格式

所有配置存储在 `~/.envtools/config.toml`：

```toml
[[groups]]
name = "nodejs-env"
kind = "env"
description = "Node.js 开发环境"
active = true
priority = 5

[[groups.variables]]
key = "NODE_ENV"
value = "development"
path_mode = "override"

[[groups]]
name = "local-dns"
kind = "hosts"
description = "本地开发域名"
active = true
priority = 0

[[groups.hosts_entries]]
ip = "127.0.0.1"
hostname = "api.local"

[[groups.hosts_entries]]
ip = "127.0.0.1"
hostname = "db.local"

[[profiles]]
name = "fullstack-dev"
description = "全栈开发环境"
groups = ["nodejs-env", "local-dns"]
```
