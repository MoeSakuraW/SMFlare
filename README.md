# SMFlare

[![Tauri](https://img.shields.io/badge/Tauri-2.x-blue?logo=tauri)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-green?logo=vue.js)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue?logo=typescript)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> **SMFlare** = **SM**.MS + Cloud**flare**

一个基于 Tauri + Vue 3 + Rust 构建的现代化 SM.MS 图床客户端。

## 功能特性

- **图片上传**：支持点击选择或拖拽上传，支持批量上传
- **相册管理**：无限滚动加载，查看上传历史
- **图片操作**：收藏、备注、删除、批量下载（ZIP 打包）
- **云端同步**：SM.MS 图床存储 + Cloudflare D1 元数据管理
- **安全存储**：AES-256-GCM 加密敏感信息（密码、Token）
- **跨平台**：支持 Windows、macOS、Linux

## 安装

### 下载预编译版本

从 [GitHub Releases](https://github.com/MoeSakuraW/SMFlare/releases) 下载适合你系统的安装包：

- **Windows**:
    - `SMFlare_x64-setup.exe` - 安装向导
    - `SMFlare_x64_en-US.msi` - MSI 安装包
- **macOS**:
    - `SMFlare_aarch64.dmg` - Apple Silicon (M1/M2/M3)
- **Linux**:
    - `SMFlare_amd64.deb` - Debian/Ubuntu
    - `SMFlare_amd64.AppImage` - 通用格式（无需安装）
    - `SMFlare.x86_64.rpm` - Fedora/RHEL/CentOS

### 开发环境要求

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/MoeSakuraW/SMFlare.git
cd SMFlare

# 安装依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

## 使用说明

### 1. 配置 Cloudflare D1 数据库

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. 创建 D1 数据库
3. 获取 `Account ID`、`Database ID` 和 `API Token`
4. 在应用设置中填入配置信息

### 2. 配置 SM.MS 图床

1. 注册 [SM.MS](https://sm.ms/) 账号
2. 在应用中输入用户名和密码
3. 点击获取 Token 完成配置

### 3. 上传图片

- 点击上传区域选择图片
- 或直接拖拽图片到上传区域
- 可添加备注信息

## 配置说明

### 本地配置文件

配置文件存储位置：

- **Windows**: `%APPDATA%\com.bytemoe.smflare\d1_config.json`
- **macOS/Linux**: `~/.config/com.bytemoe.smflare/d1_config.json`

## 开发指南

### 常用命令

```bash
# 启动开发服务器
pnpm dev

# 启动 Tauri 开发模式
pnpm tauri dev

# 类型检查
pnpm vue-tsc --noEmit

# 构建前端
pnpm build

# 构建桌面应用
pnpm tauri build
```

## 贡献指南

欢迎提交 Issue 和 Pull Request！

### 贡献步骤

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 提交规范

请遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat`: 新功能
- `fix`: 修复 Bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建/工具相关

## 许可证

本项目基于 [MIT License](LICENSE) 开源。

## 致谢

- [Tauri](https://tauri.app/) - 构建跨平台桌面应用
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Element Plus](https://element-plus.org/) - Vue 3 UI 组件库
- [SM.MS](https://sm.ms/) - 免费图床服务
- [Cloudflare D1](https://developers.cloudflare.com/d1/) - Serverless SQL 数据库
