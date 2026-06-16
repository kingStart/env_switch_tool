# EnvTools

跨平台环境变量分组切换 + Hosts 域名映射 + 场景管理工具。支持 CLI / GUI 双模式，分组管理，已打开终端实时生效。

## 特性

- **分组管理**: 将环境变量按场景分组（Java/Node/Go/Production...），一键切换
- **Hosts 域名映射**: 管理本地域名解析，按组启用/禁用，自动写入系统 hosts 文件
- **Profile 场景**: 将多个组绑定为场景（如"全栈开发"），一键激活整个环境
- **实时生效**: 修改后所有已打开终端自动获取新环境变量，无需重启
- **跨平台**: Windows / Linux / macOS，支持 bash / zsh / fish / PowerShell
- **PATH 智能合并**: 支持 prepend / append / override 三种模式
- **优先级冲突解决**: 多分组同时启用时，高优先级值胜出
- **导入/导出**: JSON 格式，团队共享配置
- **GUI + 系统托盘**: 图形化管理，托盘快速切换
- **零配置**: `envtools init` 自动检测 shell 并注入 hook
- **安全提权**: Hosts 文件操作按需弹出 UAC/sudo 提权

## 快速开始

```bash
# 编译安装
cargo install --path crates/cli

# 初始化（自动注入 shell hook）
envtools init

# 创建环境变量组并添加变量
envtools group create java -d "Java 17" -p 10
envtools set java JAVA_HOME=/usr/lib/jvm/java-17 +PATH=/usr/lib/jvm/java-17/bin

# 启用（所有终端立即生效）
envtools enable java

# 创建 Hosts 域名映射组
envtools group create local-dns -d "本地开发域名" --kind hosts
envtools hosts add local-dns 127.0.0.1 api.local
envtools hosts add local-dns 127.0.0.1 db.local
envtools enable local-dns
envtools hosts sync  # 需要管理员权限

# 创建场景（Profile）
envtools profile create fullstack-dev -g java,local-dns
envtools profile activate fullstack-dev  # 一键启用所有关联组
```

## 架构

DDD 分层设计 + TDD 驱动开发，详见 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)。

## 文档

- [架构设计](docs/ARCHITECTURE.md)
- [用户指南](docs/USER_GUIDE.md)
- [Hosts & Profile 功能说明](docs/HOSTS_PROFILE.md)

## 开发

```bash
# 运行测试
cargo test --workspace

# 构建 CLI
cargo build --release -p envtools-cli

# 构建 GUI (需要 Node.js)
npm install
npm run build
cd src-tauri && cargo build --release
```

## License

MIT
