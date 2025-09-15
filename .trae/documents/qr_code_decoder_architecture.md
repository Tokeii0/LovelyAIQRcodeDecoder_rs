# 基于 OpenCV-Rust 的二维码解码器技术架构文档

## 1. Architecture design

```mermaid
graph TD
    A[用户输入] --> B[命令行解析器]
    B --> C[图像加载模块]
    C --> D[OpenCV 图像处理]
    D --> E[二维码检测引擎]
    E --> F[解码处理器]
    F --> G[结果输出模块]
    
    subgraph "应用层"
        B
        G
    end
    
    subgraph "核心处理层"
        C
        D
        E
        F
    end
    
    subgraph "外部依赖"
        H[OpenCV 库]
        I[文件系统]
    end
    
    D --> H
    C --> I
    G --> I
```

## 2. Technology Description

- Frontend: 无（命令行应用）
- Backend: Rust + opencv-rust@0.95.1 + clap@4.0 + serde@1.0 + serde_json@1.0

## 3. Route definitions

本项目为命令行应用程序，不涉及 Web 路由，主要的程序入口点：

| Entry Point | Purpose |
|-------------|----------|
| main() | 程序主入口，处理命令行参数和整体流程控制 |
| decode_qr() | 二维码解码核心函数 |
| process_image() | 图像处理和预处理函数 |
| output_result() | 结果输出和格式化函数 |

## 4. API definitions

### 4.1 Core API

**图像处理相关**

```rust
// 图像加载
pub fn load_image(path: &str) -> Result<Mat, opencv::Error>

// 图像预处理
pub fn preprocess_image(image: &Mat) -> Result<Mat, opencv::Error>

// 二维码检测
pub fn detect_qr_codes(image: &Mat) -> Result<Vec<QRCode>, opencv::Error>
```

**解码相关**

```rust
// 二维码解码
pub fn decode_qr_code(qr_region: &Mat) -> Result<String, DecodeError>

// 结果结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct QRCodeResult {
    pub content: String,
    pub position: QRPosition,
    pub confidence: f32,
    pub data_type: QRDataType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QRPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub corners: Vec<Point2f>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QRDataType {
    Text,
    Url,
    Email,
    Phone,
    WiFi,
    VCard,
    Unknown,
}
```

**命令行接口**

```rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 输入图像文件路径
    #[arg(short, long)]
    pub input: String,
    
    /// 输出格式 (json, text, csv)
    #[arg(short, long, default_value = "text")]
    pub output_format: String,
    
    /// 输出文件路径（可选）
    #[arg(short = 'o', long)]
    pub output_file: Option<String>,
    
    /// 详细输出模式
    #[arg(short, long)]
    pub verbose: bool,
    
    /// 图像预处理选项
    #[arg(long)]
    pub preprocess: bool,
}
```

## 5. Server architecture diagram

```mermaid
graph TD
    A[命令行参数] --> B[参数验证层]
    B --> C[图像处理服务层]
    C --> D[OpenCV 接口层]
    D --> E[二维码检测层]
    E --> F[解码服务层]
    F --> G[结果处理层]
    G --> H[输出格式化层]
    
    subgraph "应用程序"
        B
        C
        E
        F
        G
        H
    end
    
    subgraph "外部库接口"
        D
    end
```

## 6. Data model

### 6.1 Data model definition

```mermaid
erDiagram
    QRCodeResult {
        string content
        QRPosition position
        float confidence
        QRDataType data_type
        datetime detected_at
    }
    
    QRPosition {
        int x
        int y
        int width
        int height
        Point2f corners
    }
    
    ProcessingConfig {
        bool preprocess
        string output_format
        bool verbose
        string input_path
        string output_path
    }
    
    ImageMetadata {
        int width
        int height
        string format
        int channels
        string color_space
    }
    
    QRCodeResult ||--|| QRPosition : contains
    QRCodeResult ||--o| ImageMetadata : from_image
```

### 6.2 Data Definition Language

本项目不使用传统数据库，主要使用内存中的数据结构和文件系统存储。

**核心数据结构定义：**

```rust
// Cargo.toml 依赖配置
[dependencies]
opencv = "0.95.1"
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
log = "0.4"
env_logger = "0.10"

// 错误处理类型
#[derive(Debug, thiserror::Error)]
pub enum QRDecodeError {
    #[error("OpenCV error: {0}")]
    OpenCVError(#[from] opencv::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("No QR code found in image")]
    NoQRCodeFound,
    
    #[error("Failed to decode QR code: {0}")]
    DecodeError(String),
    
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
}

// 配置结构体
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub output_format: OutputFormat,
    pub preprocess: bool,
    pub verbose: bool,
    pub detection_params: DetectionParams,
}

#[derive(Debug, Clone)]
pub struct DetectionParams {
    pub min_size: i32,
    pub max_size: i32,
    pub threshold: f64,
    pub use_adaptive_threshold: bool,
}
```