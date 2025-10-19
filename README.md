# picmyweb2

一个高性能的 Rust 网页截图命令行工具，支持异步并发截图和多种目标类型。

## 功能特性

- 🚀 **高性能异步截图**：基于 Tokio 的异步架构，支持高并发截图
- 📊 **多种目标类型支持**：URL、域名、IP 地址、IP:端口格式
- 📁 **批量处理**：从文件批量读取目标并进行截图
- 📈 **实时进度显示**：使用 indicatif 库显示实时进度条
- 📝 **详细日志记录**：生成 CSV 格式的截图日志文件
- 🎯 **智能目标识别**：自动识别和分类不同类型的网络目标

## 安装

### 前提条件

- Rust 1.70+ 环境
- Chrome/Chromium 浏览器

### 安装步骤

1. 克隆项目：

```bash
git clone <repository-url>
cd picmyweb2
```

2. 构建项目：

```bash
cargo build --release
```

3. 运行程序：

```bash
cargo run --release -- --help
```

## 使用方法

### 基本使用

1. 创建目标文件（每行一个目标）：

```bash
# targets.txt
https://www.example.com
www.example.org
192.168.1.1
10.0.0.1:8080
```

2. 运行截图：

```bash
cargo run --release -- --file targets.txt --output screenshots
```

### 命令行参数

```bash
USAGE: picmyweb2 [OPTIONS] --file

-f, --file: 包含目标URL/IP的文件路径
-o, --output: 截图保存目录 [default: ./screenshots]
-c, --concurrency: 并发数 [default: 10]
-h, --help: 显示帮助信息
-V, --version: 显示版本信息
```

### 示例

```bash
# 使用默认设置
cargo run --release -- --file urls.txt

# 指定输出目录和并发数
cargo run --release -- --file urls.txt --output my_screenshots --concurrency 20
```

## 依赖项

- `headless_chrome` - 无头 Chrome 浏览器控制
- `tokio` - 异步运行时
- `clap` - 命令行参数解析
- `indicatif` - 进度条显示
- `csv` - CSV 文件处理
- `log` & `env_logger` - 日志系统

## 开发

### 运行测试

```bash
cargo test
```

### 调试模式

```bash
cargo run -- --file test_urls.txt
```

### 发布构建

```bash
cargo build --release
```

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个项目。

## 许可证

本项目采用 MIT 许可证 - 详见[LICENSE](LICENSE)文件。
