# AdGuard VPN Web Controller

基于 Rust + Axum 构建的 AdGuard VPN CLI 的 Web 管理界面。提供简洁的 Web 界面来管理 AdGuard VPN 连接状态。

> 本项目由 Claude Code 通过 qwen3.7-max 开发。

## 功能特性

- 🔐 密码认证登录
- 📊 查看 VPN 连接状态
- 🌍 查看可用节点列表（含延迟信息）
- 🔌 一键连接/断开指定节点
- ⚡ HTMX 无刷新交互
- 📱 响应式设计，支持移动端
- 🚀 单二进制文件部署，静态链接无依赖

## 技术栈

- **后端**: Rust + Axum
- **前端**: HTML + CSS + HTMX
- **模板**: Askama
- **构建**: Cargo + cargo-zigbuild (跨平台编译)

## 快速开始

### 前置要求

- Rust 1.70+
- AdGuard VPN CLI (`adguardvpn-cli`)
- 支持 Linux/Windows/macOS

### 安装运行

#### 1. 克隆项目

```bash
git clone https://github.com/xaoyao/adguardvpn-web.git
cd adguardvpn-web
```

#### 2. 配置

编辑 `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 3000

[auth]
password = "your-password"  # 修改为你的密码

[vpn]
cli_path = "adguardvpn-cli"  # AdGuard VPN CLI 路径
```

#### 3. 运行

```bash
cargo run --release
```

访问 `http://127.0.0.1:3000` 即可使用。

## 部署

### Linux 部署

```bash
cargo build --release
./target/release/adguardvpn-web
```

### OpenWrt ARM64 部署

项目提供预编译的 ARM64 版本（使用 musl libc，静态链接）：

```bash
# 1. 下载部署包
wget https://github.com/xaoyao/adguardvpn-web/releases/download/v0.1.0/adguardvpn-web-aarch64-openwrt.tar.gz

# 2. 解压
tar -xzf adguardvpn-web-aarch64-openwrt.tar.gz
cd adguardvpn-web-aarch64-openwrt

# 3. 修改密码
sed -i 's/changeme/你的密码/' config.toml

# 4. 启动服务
./vpn.sh start

# 5. 查看状态
./vpn.sh status

# 6. 查看日志
./vpn.sh log

# 7. 停止服务
./vpn.sh stop
```

管理脚本 `vpn.sh` 支持以下命令：

```bash
./vpn.sh start    # 启动
./vpn.sh stop     # 停止
./vpn.sh restart  # 重启
./vpn.sh status   # 查看状态
./vpn.sh log      # 查看日志
```

## 交叉编译

使用 `cargo-zigbuild` 进行跨平台编译：

```bash
# 安装 cargo-zigbuild
cargo install cargo-zigbuild

# 编译 Linux ARM64 (OpenWrt)
cargo zigbuild --release --target aarch64-unknown-linux-musl

# 编译 Linux x86_64
cargo zigbuild --release --target x86_64-unknown-linux-musl
```

## 项目结构

```
adguardvpn-web/
├── src/
│   ├── main.rs          # 入口文件
│   ├── cli.rs           # CLI 封装
│   ├── handlers.rs      # 请求处理器
│   └── auth.rs          # 认证逻辑
├── templates/           # HTML 模板
│   ├── login.html
│   └── index.html
├── static/              # 静态资源
│   └── style.css
├── config.toml          # 配置文件
├── Cargo.toml           # Rust 依赖
└── README.md
```

## API 端点

### 页面路由

- `GET /` - 主页（VPN 控制面板）
- `GET /login` - 登录页
- `POST /login` - 登录处理
- `POST /logout` - 登出

### HTMX API

- `GET /api/status` - 获取 VPN 状态
- `GET /api/locations` - 获取节点列表
- `POST /api/connect` - 连接指定节点
- `POST /api/disconnect` - 断开连接

## 环境变量

可通过环境变量覆盖配置：

- `VPN_PASSWORD` - 登录密码
- `VPN_CLI_PATH` - AdGuard VPN CLI 路径

## 开发

```bash
# 运行开发服务器
cargo run

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

## 常见问题

### Q: 节点名称显示为端口号（如 1080）

A: 这是 CLI 输出格式解析问题，已在最新版本中修复。解析器现在正确识别 "Connected to TOKYO" 格式。

### Q: 无法连接 VPN

A: 请确保：
1. AdGuard VPN CLI 已正确安装
2. `config.toml` 中的 `cli_path` 指向正确的 CLI 路径
3. 当前用户有权限运行 VPN 命令（可能需要 sudo）

### Q: 页面显示空白

A: 检查浏览器控制台是否有 JavaScript 错误，确保 HTMX 库正常加载。

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

- [AdGuard VPN](https://adguard.com/en/adguard-vpn/overview.html) - VPN 服务
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [HTMX](https://htmx.org/) - 前端交互库
