# EnvTools 用户指南

## 快速开始

### 安装

```bash
# 从源码编译
cargo install --path crates/cli

# 或从 release 下载预编译二进制
```

### 初始化（仅需一次）

```bash
envtools init
```

此命令会：
1. 创建配置目录 `~/.envtools/`
2. 自动检测你的 shell（bash/zsh/fish/PowerShell）
3. 自动注入 hook 到对应的 profile 文件
4. **重启终端后即可使用，无需任何其他配置**

---

## 核心用法

### 创建环境变量分组

```bash
# 创建一个 Java 开发环境分组
envtools group create java-dev -d "Java 17 开发环境" -p 10

# 创建一个 Node.js 分组
envtools group create node-dev -d "Node.js 开发" -p 5
```

### 添加环境变量

```bash
# 普通变量（覆盖模式）
envtools set java-dev JAVA_HOME=/usr/lib/jvm/java-17
envtools set java-dev MAVEN_HOME=/opt/maven

# PATH 前置（推荐）: 使用 + 前缀
envtools set java-dev +PATH=/usr/lib/jvm/java-17/bin

# PATH 追加: 使用 += 语法
envtools set java-dev PATH+=/opt/maven/bin

# 一次设置多个
envtools set node-dev NODE_ENV=development NPM_REGISTRY=https://registry.npmmirror.com
```

### 启用/禁用分组

```bash
# 启用（对所有终端立即生效）
envtools enable java-dev

# 同时启用多个
envtools enable java-dev node-dev

# 禁用
envtools disable java-dev
```

**关键特性**: 启用/禁用后，所有已打开的终端在执行下一条命令时自动获取新的环境变量，无需重启终端。

### 查看状态

```bash
# 查看所有分组
envtools group list

# 查看分组详情
envtools group show java-dev

# 查看当前生效的环境变量快照
envtools status
```

---

## 进阶功能

### 优先级冲突解决

当多个启用的分组定义了同一个变量时，**高优先级的值胜出**：

```bash
envtools group create prod-db -d "Production DB" -p 20
envtools group create dev-db -d "Dev DB" -p 5

envtools set prod-db DATABASE_URL=postgres://prod-host/db
envtools set dev-db DATABASE_URL=postgres://localhost/db

# 同时启用时，prod-db 的值生效（priority 20 > 5）
envtools enable prod-db dev-db
envtools status
# → DATABASE_URL = postgres://prod-host/db
```

### 导出/导入配置

```bash
# 导出所有分组到文件
envtools export -o my-env-config.json

# 只导出指定分组
envtools export -g java-dev -g node-dev -o partial.json

# 导入配置（跳过已存在的分组）
envtools import my-env-config.json

# 强制覆盖已存在的分组
envtools import my-env-config.json --overwrite
```

适用场景：团队共享环境配置、新机器快速恢复、多环境切换。

### 删除变量/分组

```bash
# 删除分组中的变量
envtools unset java-dev MAVEN_HOME

# 删除整个分组
envtools group delete java-dev
```

---

## GUI 模式

安装 Tauri 应用后：
- 主界面左侧为分组列表，点击 Toggle 开关快速启用/禁用
- 右侧面板编辑分组中的变量
- **系统托盘**常驻后台，右键可快速切换分组

---

## 工作原理

```
┌─────────────────────────────────────────────┐
│  envtools enable java-dev                    │
│                                              │
│  1. 更新 config.toml（标记 active=true）     │
│  2. 重新计算合并后的环境变量                   │
│  3. 写入 ~/.envtools/active.env              │
│  4. (Windows) 更新注册表 + 广播变更           │
└─────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────┐
│  已打开的终端                                 │
│                                              │
│  Shell Hook (每次 prompt 触发):               │
│  1. 检查 active.env 文件修改时间              │
│  2. 如有变更 → unset 旧变量                  │
│  3. source active.env → 新变量生效           │
└─────────────────────────────────────────────┘
```

**性能**: Hook 只在文件 mtime 变化时执行 source，正常情况下开销为 1 次 stat 系统调用（< 0.1ms）。

---

## 支持的 Shell

| Shell | Hook 机制 | Profile 文件 |
|-------|----------|--------------|
| bash | PROMPT_COMMAND | ~/.bashrc |
| zsh | precmd hook | ~/.zshrc |
| fish | fish_prompt event | ~/.config/fish/config.fish |
| PowerShell | prompt function 注入 | $PROFILE |

---

## Hosts 域名映射

### 创建 Hosts 组

```bash
envtools group create local-dns --kind hosts -d "本地开发域名"
```

### 管理映射条目

```bash
envtools hosts add local-dns 127.0.0.1 api.local
envtools hosts add local-dns 127.0.0.1 db.local
envtools hosts remove local-dns db.local
```

### 同步到系统

```bash
envtools enable local-dns
envtools hosts sync  # 需要管理员权限
```

---

## Profile 场景管理

### 创建场景

```bash
envtools profile create fullstack -d "全栈开发" -g java-dev,local-dns
```

### 激活/停用场景

```bash
envtools profile activate fullstack    # 启用所有关联组
envtools profile deactivate fullstack  # 禁用所有关联组
```

### 查看场景

```bash
envtools profile list
envtools profile show fullstack
```

---

## 配置文件

位置: `~/.envtools/config.toml`

```toml
[[groups]]
name = "java-dev"
kind = "env"
description = "Java 17 开发环境"
active = true
priority = 10

[[groups.variables]]
key = "JAVA_HOME"
value = "/usr/lib/jvm/java-17"
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

[[profiles]]
name = "fullstack"
description = "全栈开发"
groups = ["java-dev", "local-dns"]
```
