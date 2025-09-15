# LovelyAIQRcodeDecoder

🚀 基于 OpenCV-Rust 的高性能二维码解码器

## 项目简介

这是一个使用 Rust 语言和 OpenCV 库开发的专业二维码检测和解码工具。该工具不仅支持常规的二维码识别，还集成了增强图像处理、暴力破解解码和批量处理等高级功能，能够处理各种复杂场景下的二维码识别需求。

## ✨ 核心特性

- 🚀 **高性能解码**: 基于 OpenCV 的高效图像处理引擎
- 🔍 **智能增强**: 集成多种图像变换和预处理算法
- 💪 **暴力破解**: 支持多参数组合的暴力破解解码模式
- 📦 **批量处理**: 支持目录级批量处理和递归扫描
- 📷 **多格式支持**: 支持 PNG、JPG、JPEG、BMP、TIFF、WebP 等格式
- 🎯 **精确检测**: 可配置置信度阈值，过滤低质量结果
- 📊 **多种输出**: 支持 Text、JSON、CSV、Verbose 等输出格式
- 📍 **位置信息**: 输出二维码在图像中的精确坐标位置
- 🎨 **彩色输出**: 支持彩色终端输出和进度显示
- 🛠️ **命令行友好**: 丰富的命令行选项，易于脚本集成

## 📋 系统要求

- **Rust**: 1.70.0 或更高版本
- **OpenCV**: 4.x 系列
- **操作系统**: Linux、macOS、Windows
- **内存**: 建议 4GB 以上（批量处理大图像时）

## 🛠️ 安装指南

### 1. 安装 OpenCV 依赖

#### macOS (推荐使用 Homebrew)
```bash
brew install opencv pkg-config
```

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install libopencv-dev clang libclang-dev pkg-config
```

#### CentOS/RHEL
```bash
sudo yum install opencv-devel clang-devel pkg-config
```

#### Windows
1. 下载并安装 [OpenCV](https://opencv.org/releases/)
2. 设置环境变量 `OPENCV_DIR`
3. 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)

### 2. 编译和安装

```bash
# 克隆项目
git clone https://github.com/yourusername/LovelyAIQRcodeDecoder.git
cd LovelyAIQRcodeDecoder

# 编译发布版本
cargo build --release

# 运行测试
cargo test

# 安装到系统（可选）
cargo install --path .
```

## 🚀 使用指南

### 基本用法

```bash
# 解码单个图像
./target/release/lovely-ai-qrcode-decoder image.jpg

# 或者如果已安装到系统
lovely-ai-qrcode-decoder image.jpg

# 指定输出文件
lovely-ai-qrcode-decoder -o result.txt image.png

# 使用 JSON 格式输出
lovely-ai-qrcode-decoder -f json image.jpg
```

### 高级功能

#### 图像预处理和增强
```bash
# 启用图像预处理
lovely-ai-qrcode-decoder --preprocess image.jpg

# 保存预处理后的图像
lovely-ai-qrcode-decoder --preprocess --save-processed processed.jpg image.jpg

# 启用暴力破解模式（适用于难以识别的图像）
lovely-ai-qrcode-decoder --brute-force --expected-count 2 image.jpg

# 启用反色处理
lovely-ai-qrcode-decoder --invert --preprocess image.jpg
```

#### 批量处理
```bash
# 批量处理目录中的所有图像
lovely-ai-qrcode-decoder --batch /path/to/images/

# 递归处理子目录
lovely-ai-qrcode-decoder --batch --recursive /path/to/images/

# 生成批量处理报告
lovely-ai-qrcode-decoder --batch --report-output report.json /path/to/images/

# 显示处理进度
lovely-ai-qrcode-decoder --batch --show-progress /path/to/images/
```

#### 输出控制
```bash
# 设置置信度阈值
lovely-ai-qrcode-decoder --min-confidence 0.8 image.jpg

# 显示位置信息
lovely-ai-qrcode-decoder --show-position image.jpg

# 详细输出模式
lovely-ai-qrcode-decoder -f verbose --preprocess image.jpg

# 静默模式（仅输出结果）
lovely-ai-qrcode-decoder --quiet image.jpg
```

### 📝 命令行选项

#### 基本选项
| 选项 | 简写 | 描述 |
|------|------|------|
| `--output <文件>` | `-o` | 指定输出文件路径 |
| `--format <格式>` | `-f` | 输出格式：text, json, csv, verbose |
| `--verbose` | `-v` | 显示详细处理信息 |
| `--quiet` | `-q` | 静默模式，只输出结果 |
| `--help` | `-h` | 显示帮助信息 |
| `--version` | `-V` | 显示版本信息 |

#### 图像处理选项
| 选项 | 简写 | 描述 |
|------|------|------|
| `--preprocess` | `-p` | 启用图像预处理 |
| `--brute-force` | | 启用暴力破解解码模式 |
| `--invert` | | 启用反色处理 |
| `--save-processed <文件>` | | 保存预处理后的图像 |
| `--min-confidence <值>` | | 最小置信度阈值 (0.0-1.0) |
| `--expected-count <数量>` | | 预期二维码数量 |
| `--randomize` | | 随机化暴力破解参数 |

#### 批量处理选项
| 选项 | 简写 | 描述 |
|------|------|------|
| `--batch <目录>` | | 启用批量处理模式 |
| `--recursive` | | 递归处理子目录 |
| `--report-output <文件>` | | 批量处理报告输出路径 |
| `--show-progress` | | 显示处理进度条 |

#### 显示选项
| 选项 | 简写 | 描述 |
|------|------|------|
| `--show-position` | | 显示二维码位置信息 |
| `--colored-output` | | 启用彩色终端输出 |

## 📊 输出格式

### 文本格式 (默认)
```
🎯 解码完成，找到 1 个二维码（置信度 >= 0.50）

二维码内容: https://example.com
位置: (100, 150) - (200, 250)
置信度: 0.95

✅ 处理完成
```

### JSON 格式
```json
{
  "results": [
    {
      "content": "https://example.com",
      "points": [
        [100.0, 150.0],
        [200.0, 150.0],
        [200.0, 250.0],
        [100.0, 250.0]
      ]
    }
  ],
  "total_found": 1,
  "processing_time_ms": 45,
  "success": true
}
```

### CSV 格式
```csv
content,point1_x,point1_y,point2_x,point2_y,point3_x,point3_y,point4_x,point4_y
"https://example.com",100.0,150.0,200.0,150.0,200.0,250.0,100.0,250.0
```

### Verbose 格式
```
🚀 开始处理图像...
📷 图像加载完成
🔧 开始图像预处理...
✨ 图像预处理完成
🔍 开始增强二维码检测和解码...
🎯 解码完成，找到 1 个二维码（置信度 >= 0.50）

=== 解码结果 ===
内容: https://example.com
位置: [(100.0, 150.0), (200.0, 150.0), (200.0, 250.0), (100.0, 250.0)]

=== 处理统计 ===
总计找到: 1 个二维码
处理时间: 45ms
✅ 处理完成
```

## 🔧 核心功能详解

### 图像预处理技术

启用 `--preprocess` 选项时，工具会自动应用以下图像增强技术：

- **🎨 灰度转换**: 将彩色图像转换为灰度图像，提高处理效率
- **🌫️ 高斯模糊**: 减少图像噪声，改善边缘检测
- **📈 直方图均衡化**: 增强图像对比度，突出二维码特征
- **🎯 自适应阈值**: 改善二值化效果，适应不同光照条件
- **🔄 形态学操作**: 优化图像结构，去除小噪点
- **🔍 多尺度检测**: 支持不同尺寸的二维码检测

### 暴力破解解码

当常规方法无法识别时，启用 `--brute-force` 模式：

- **📊 参数组合**: 自动尝试多种对比度、亮度、模糊、缩放参数组合
- **🎲 随机化**: 支持 `--randomize` 选项随机化参数顺序
- **🔄 反色处理**: 支持 `--invert` 选项处理反色二维码
- **📍 重复检测**: 智能去除重复检测结果
- **⚡ 并行处理**: 利用多核CPU加速处理

### 批量处理功能

- **📁 目录扫描**: 自动扫描指定目录中的所有图像文件
- **🔄 递归处理**: 支持递归处理子目录
- **📊 进度显示**: 实时显示处理进度和统计信息
- **📋 报告生成**: 生成详细的批量处理报告
- **🎨 彩色输出**: 支持彩色终端输出，提升用户体验

## 💡 性能优化建议

1. **📏 图像尺寸**: 对于超大图像（>4K），建议先缩放到合适尺寸
2. **🔧 预处理**: 对于低质量、模糊或光照不均的图像，启用预处理可显著提高识别率
3. **🎯 置信度**: 适当调整置信度阈值（0.3-0.8）以平衡准确性和召回率
4. **💪 暴力破解**: 对于难以识别的图像，使用暴力破解模式
5. **📦 批量处理**: 对于大量文件，使用批量处理模式提高效率
6. **🚀 并行处理**: 利用多核CPU，同时处理多个文件

## ⚠️ 错误处理

工具提供详细的错误信息和退出码：

| 错误类型 | 退出码 | 描述 |
|---------|--------|------|
| **文件不存在** | 2 | 检查输入文件路径是否正确 |
| **OpenCV错误** | 3 | 检查OpenCV安装和图像完整性 |
| **参数错误** | 4 | 检查命令行参数格式 |
| **格式不支持** | 5 | 确认文件格式在支持列表中 |
| **图像处理错误** | 6 | 图像损坏或格式异常 |
| **输出错误** | 7 | 检查输出目录写权限 |

## 📁 项目结构

```
LovelyAIQRcodeDecoder/
├── src/
│   ├── main.rs                 # 🚀 主程序入口和命令行处理
│   ├── cli.rs                  # 📋 命令行参数定义和解析
│   ├── qr_decoder.rs           # 🔍 二维码解码核心逻辑
│   ├── image_processor.rs      # 🖼️ 图像预处理和增强
│   ├── brute_force_decoder.rs  # 💪 暴力破解解码器
│   ├── batch_processor.rs      # 📦 批量处理功能
│   ├── progress_display.rs     # 📊 进度显示和统计
│   ├── output_formatter.rs     # 📄 输出格式化（JSON/CSV/文本）
│   └── utils.rs                # 🛠️ 工具函数和辅助方法
├── models/                     # 🤖 OpenCV DNN模型文件
│   ├── detect.caffemodel       # Caffe检测模型
│   ├── detect.prototxt         # 模型配置文件
│   ├── sr.caffemodel           # 超分辨率模型
│   └── sr.prototxt             # 超分辨率配置
├── test/                       # 🧪 测试图像文件
│   ├── qr_*.png                # 各种测试二维码图像
│   └── ...                     # 更多测试文件
├── Cargo.toml                  # 📦 项目配置和依赖
├── Cargo.lock                  # 🔒 依赖版本锁定
├── .gitignore                  # 🚫 Git忽略文件配置
└── README.md                   # 📖 项目说明文档
```

## 🛠️ 开发说明

### 构建和测试

```bash
# 🧪 运行单元测试
cargo test

# 🚀 构建发布版本（优化编译）
cargo build --release

# 🔍 运行代码检查
cargo clippy

# 📝 格式化代码
cargo fmt

# 📊 运行基准测试
cargo bench

# 🧹 清理构建文件
cargo clean
```

### 开发环境设置

```bash
# 安装开发工具
cargo install cargo-watch cargo-edit

# 监视文件变化并自动重新编译
cargo watch -x run

# 添加新依赖
cargo add <dependency_name>
```

### 代码贡献

我们欢迎各种形式的贡献！请遵循以下步骤：

1. **🍴 Fork 项目**: 点击右上角的 Fork 按钮
2. **🌿 创建分支**: `git checkout -b feature/amazing-feature`
3. **💻 编写代码**: 遵循 Rust 编码规范
4. **🧪 添加测试**: 为新功能添加相应测试
5. **📝 提交更改**: `git commit -m 'feat: add amazing feature'`
6. **🚀 推送分支**: `git push origin feature/amazing-feature`
7. **📋 创建 PR**: 提交 Pull Request 并描述更改内容

### 提交信息规范

请使用以下格式的提交信息：

- `feat:` 新功能
- `fix:` 错误修复
- `docs:` 文档更新
- `style:` 代码格式调整
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建过程或辅助工具的变动

## 📄 许可证

本项目采用 **MIT 许可证** - 详见 [LICENSE](LICENSE) 文件。

这意味着您可以自由地：
- ✅ 商业使用
- ✅ 修改代码
- ✅ 分发代码
- ✅ 私人使用

## 🙏 致谢

感谢以下开源项目和社区：

- **[OpenCV](https://opencv.org/)** - 强大的计算机视觉库
- **[Rust](https://www.rust-lang.org/)** - 安全高效的系统编程语言
- **[opencv-rust](https://github.com/twistedfall/opencv-rust)** - OpenCV 的 Rust 绑定
- **[clap](https://clap.rs/)** - 优雅的命令行参数解析库
- **[serde](https://serde.rs/)** - 序列化和反序列化框架
- **[tokio](https://tokio.rs/)** - 异步运行时（如果使用）

## 📞 支持与反馈

如果您遇到问题或有建议，请：

1. **🐛 报告 Bug**: 在 [Issues](../../issues) 页面创建新的问题报告
2. **💡 功能建议**: 在 [Issues](../../issues) 页面提出功能请求
3. **❓ 使用问题**: 查看 [Discussions](../../discussions) 或创建新的讨论

---

**⭐ 如果这个项目对您有帮助，请给我们一个 Star！**

## 📈 更新日志

### v2.0.0 (2024-12-XX) - 重大更新
- 🚀 **新增暴力破解解码模式** - 大幅提升难识别图像的解码成功率
- 📦 **批量处理功能** - 支持目录递归处理和进度显示
- 🎨 **彩色终端输出** - 提升用户体验
- 🤖 **集成OpenCV DNN模型** - 支持深度学习增强检测
- 📊 **多种输出格式** - JSON、CSV、详细文本格式
- ⚡ **性能优化** - 并行处理和内存优化
- 🔧 **增强图像预处理** - 更多预处理选项和参数调节

### v1.2.0 (2024-01-25)
- ✨ 添加 JSON 和 CSV 输出格式
- 🛠️ 改进错误处理和退出码
- 📖 添加详细的帮助信息
- 🐛 修复图像加载和处理的已知问题

### v1.1.0 (2024-01-20)
- 🔧 添加图像预处理功能
- 📈 改进解码准确率
- 📦 添加基础批量处理支持
- ⚡ 性能优化和内存使用改进

### v1.0.0 (2024-01-15)
- 🎉 初始版本发布
- 🔍 支持基本二维码解码功能
- 🖼️ 支持多种图像格式（PNG、JPG、BMP等）
- 💻 完整的命令行界面

## ❓ 常见问题

### Q: 为什么某些二维码无法识别？
**A:** 可能的原因和解决方案：
- 📷 **图像质量问题**: 尝试使用 `--preprocess` 选项
- 🔄 **角度或变形**: 使用 `--brute-force` 模式
- 🌓 **光照不均**: 启用预处理和暴力破解
- 📏 **尺寸过小**: 确保二维码在图像中足够大

### Q: 如何提高处理速度？
**A:** 优化建议：
- 🚀 使用发布版本：`cargo build --release`
- 📦 批量处理：一次处理多个文件
- 🎯 调整参数：根据图像质量选择合适的选项
- 💾 充足内存：确保系统有足够可用内存

### Q: 支持哪些图像格式？
**A:** 支持的格式包括：
- 📸 **常见格式**: PNG, JPG/JPEG, BMP, TIFF
- 🔧 **其他格式**: GIF, WebP（取决于OpenCV编译选项）

### Q: 如何处理大量图像文件？
**A:** 使用批量处理功能：
```bash
# 处理整个目录
./lovely-ai-qrcode-decoder /path/to/images/ --recursive --progress

# 生成处理报告
./lovely-ai-qrcode-decoder /path/to/images/ --generate-report --output report.json
```

### Q: 遇到 OpenCV 相关错误怎么办？
**A:** 检查以下几点：
- 📦 确认 OpenCV 正确安装
- 🔧 检查系统环境变量
- 📋 查看错误日志获取详细信息
- 🆕 尝试更新到最新版本的 OpenCV

---

💡 **提示**: 如果遇到其他问题，请在 [Issues](../../issues) 页面搜索或创建新的问题报告。

---

如有问题或建议，请提交 Issue 或联系维护者。