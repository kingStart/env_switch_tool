# EnvTools 架构文档

## 概述

EnvTools 是一个跨平台环境变量分组切换工具，支持 CLI 和 GUI 双模式，通过 Shell Hook 机制实现已打开终端实时生效。

**技术栈**: Rust + Tauri v2 + React + TailwindCSS

**架构模式**: DDD (Domain-Driven Design) 分层 + TDD (Test-Driven Development)

---

## DDD 分层架构

```
Interface Layer ──→ Application Layer ──→ Domain Layer ←── Infrastructure Layer
(CLI, Tauri GUI)    (Use Cases)           (纯业务逻辑)     (TOML, Registry, File IO)
```

### 依赖规则

- **Domain** 层零外部依赖，仅定义 trait (Port)
- **Infrastructure** 实现 Domain 的 trait（依赖倒置）
- **Application** 编排 Domain 对象，协调 Infrastructure
- **Interface** 调用 Application Use Cases

### 层级职责

| 层 | 目录 | 职责 |
|----|------|------|
| Domain | `crates/domain/` | 聚合根、值对象、领域服务、Repository Trait、领域事件 |
| Application | `crates/application/` | Use Case 编排、DTO、Port 定义 |
| Infrastructure | `crates/infrastructure/` | TOML 持久化、平台 Adapter、State Writer、Hosts 文件操作、跨平台提权 |
| Interface/CLI | `crates/cli/` | CLI 参数解析、命令路由 |
| Interface/GUI | `src-tauri/` + `src/` | Tauri IPC Commands、React 前端 |

---

## 领域模型

### 核心概念

- **ManagedGroup (Aggregate Root)**: 可管理的组（环境变量组或 Hosts 域名映射组），可启用/禁用
- **GroupKind (Value Object)**: 组类型枚举 — `Env`（环境变量）或 `Hosts`（域名映射）
- **EnvVariable (Value Object)**: 单个 key=value 环境变量，含 PathMode
- **HostsEntry (Value Object)**: 单条 IP-hostname 映射，带格式校验
- **Profile (Aggregate Root)**: 场景——绑定多个组名，一键激活/停用
- **Priority (Value Object)**: 优先级，冲突解决时高优先级胜出
- **GroupPolicy (Domain Service)**: 跨 Group 合并策略（PATH prepend/append、冲突解决）
- **DomainEvent**: GroupEnabled / GroupDisabled / VariableAdded 等

### PathMode 语义

| Mode | 行为 |
|------|------|
| Override | 直接覆盖变量值 |
| Prepend | 将值前置到已有值，用路径分隔符连接 |
| Append | 将值追加到已有值 |

---

## 数据流

### 启用分组流程

```
用户执行 `envtools enable java-dev`
    │
    ├─→ CLI 解析参数
    ├─→ EnableGroupUseCase.execute("java-dev")
    │       ├─→ GroupRepository.find_by_name("java-dev")
    │       ├─→ EnvGroup.enable() → emit GroupEnabled event
    │       ├─→ GroupRepository.save(group)
    │       └─→ SyncEnvironmentUseCase.execute()
    │               ├─→ GroupRepository.find_active()
    │               ├─→ GroupPolicy::resolve(active_groups)
    │               ├─→ StateFileWriter.write_bash(resolved)
    │               ├─→ StateFileWriter.write_powershell(resolved)
    │               └─→ StateFileWriter.write_fish(resolved)
    │
    └─→ 生成 ~/.envtools/active.env, active.ps1, active.fish
```

### Shell Hook 实时生效

```
已打开终端: 用户按回车
    │
    └─→ Shell prompt 触发 hook
            ├─→ 检查 active.env mtime
            ├─→ 如变更: unset 旧变量
            └─→ source 新的 active.env → 环境变量生效
```

---

## 跨平台支持

| 平台 | Shell Hook | 系统级持久化 | 广播 |
|------|-----------|-------------|------|
| Windows | PowerShell prompt hook | Registry `HKCU\Environment` | WM_SETTINGCHANGE |
| Linux | bash PROMPT_COMMAND / zsh precmd | ~/.profile.d/envtools.sh | 无（依赖 hook） |
| macOS | zsh precmd / fish prompt | launchctl setenv + profile | 无（依赖 hook） |

---

## 项目结构

```
env_tools/
├── Cargo.toml                    # Workspace 根
├── crates/
│   ├── domain/                   # Domain Layer
│   │   └── src/
│   │       ├── model/            # EnvGroup, EnvVariable, Priority
│   │       ├── service/          # GroupPolicy
│   │       ├── repository.rs     # Trait 定义
│   │       ├── event.rs          # 领域事件
│   │       └── error.rs          # 错误类型
│   ├── application/              # Application Layer
│   │   └── src/
│   │       ├── use_case/         # EnableGroup, DisableGroup, ManageGroup, ExportImport, Sync
│   │       ├── dto.rs            # 数据传输对象
│   │       └── port.rs           # StateFileWriter trait
│   ├── infrastructure/           # Infrastructure Layer
│   │   └── src/
│   │       ├── persistence/      # TomlGroupRepository
│   │       ├── platform/         # Windows/Linux/macOS adapters
│   │       └── shell/            # StateWriter, HookGenerator
│   └── cli/                      # Interface Layer - CLI
│       └── src/
│           ├── main.rs           # clap 入口
│           └── commands/         # 命令实现
├── src-tauri/                    # Interface Layer - Tauri GUI 后端
│   └── src/
│       ├── lib.rs                # App setup + tray
│       ├── commands.rs           # IPC handlers
│       └── tray.rs               # 系统托盘
├── src/                          # React 前端
│   ├── App.tsx
│   └── components/
├── .github/workflows/ci.yml      # CI/CD
└── docs/                          # 文档
```

---

## 测试策略 (TDD)

| 层级 | 数量 | 类型 | 工具 |
|------|------|------|------|
| Domain | 27 | 单元测试 | proptest |
| Application | 10 | Mock 驱动 | RwLock mock |
| Infrastructure | 11 | 集成测试 | tempfile |
| CLI E2E | 11 | 端到端 | assert_cmd + predicates |
| **合计** | **59** | | |

---

## 关键依赖

| Crate | 用途 |
|-------|------|
| clap | CLI 参数解析 |
| serde + toml | 配置序列化 |
| serde_json | Export/Import 格式 |
| winreg + windows | Windows Registry |
| tauri v2 | GUI 框架 |
| thiserror | 错误类型 |
| dirs | 跨平台路径 |
| tempfile | 测试隔离 |
| assert_cmd | CLI E2E 测试 |
