//! 数据结构定义模块
//! 
//! 定义了二维码解码过程中使用的各种数据结构。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli::Args;
use crate::error::{QRDecodeError, Result};

/// 简化的二维码解码结果（用于批量处理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrResult {
    /// 解码后的文本内容
    pub content: String,
    /// 二维码角点坐标 (可选)
    pub points: Option<Vec<(f32, f32)>>,
}

/// 二维码在图像中的位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRPosition {
    /// 左上角 x 坐标
    pub x: i32,
    /// 左上角 y 坐标
    pub y: i32,
    /// 宽度
    pub width: i32,
    /// 高度
    pub height: i32,
    /// 四个角点坐标 (可选)
    pub corners: Option<Vec<(f32, f32)>>,
}

impl QRPosition {
    /// 创建新的位置信息
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corners: None,
        }
    }
    
    /// 设置角点坐标
    pub fn with_corners(mut self, corners: Vec<(f32, f32)>) -> Self {
        self.corners = Some(corners);
        self
    }
    
    /// 获取中心点坐标
    pub fn center(&self) -> (f32, f32) {
        (
            self.x as f32 + self.width as f32 / 2.0,
            self.y as f32 + self.height as f32 / 2.0,
        )
    }
    
    /// 获取面积
    pub fn area(&self) -> i32 {
        self.width * self.height
    }
}

/// 二维码解码结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeResult {
    /// 解码后的文本内容
    pub content: String,
    /// 二维码在图像中的位置
    pub position: QRPosition,
    /// 解码的置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 二维码类型 (如 QR_CODE, DATA_MATRIX 等)
    pub qr_type: String,
    /// 解码时间戳
    pub timestamp: DateTime<Utc>,
    /// 原始字节数据 (可选)
    pub raw_bytes: Option<Vec<u8>>,
}

impl QRCodeResult {
    /// 创建新的解码结果
    pub fn new<S: Into<String>>(
        content: S,
        position: QRPosition,
        confidence: f32,
        qr_type: S,
    ) -> Self {
        Self {
            content: content.into(),
            position,
            confidence,
            qr_type: qr_type.into(),
            timestamp: Utc::now(),
            raw_bytes: None,
        }
    }
    
    /// 设置原始字节数据
    pub fn with_raw_bytes(mut self, raw_bytes: Vec<u8>) -> Self {
        self.raw_bytes = Some(raw_bytes);
        self
    }
    
    /// 检查解码结果是否有效
    pub fn is_valid(&self) -> bool {
        !self.content.is_empty() && self.confidence > 0.0
    }
}

/// 输出格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// 纯文本格式
    Text,
    /// JSON 格式
    Json,
    /// CSV 格式
    Csv,
    /// 详细格式
    Verbose,
}

impl std::str::FromStr for OutputFormat {
    type Err = QRDecodeError;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "verbose" | "v" => Ok(OutputFormat::Verbose),
            _ => Err(QRDecodeError::invalid_input(format!(
                "不支持的输出格式: {}",
                s
            ))),
        }
    }
}

/// 处理配置
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    /// 输入图像路径
    pub input_path: PathBuf,
    /// 输出文件路径 (可选)
    pub output_path: Option<PathBuf>,
    /// 输出格式
    pub output_format: OutputFormat,
    /// 是否启用图像预处理
    pub preprocess: bool,
    /// 是否显示详细信息
    pub verbose: bool,
    /// 是否显示位置信息
    pub show_position: bool,
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 是否保存处理后的图像
    pub save_processed: bool,
    /// 处理后图像保存路径
    pub processed_output_path: Option<PathBuf>,
    /// 是否启用暴力破解模式
    pub brute_force: bool,
    /// 预期的二维码数量
    pub expected_count: usize,
    /// 是否随机化参数
    pub randomize: bool,
    /// 是否反色处理
    pub invert: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            input_path: PathBuf::new(),
            output_path: None,
            output_format: OutputFormat::Text,
            preprocess: true,
            verbose: false,
            show_position: false,
            min_confidence: 0.0,
            save_processed: false,
            processed_output_path: None,
            brute_force: false,
            expected_count: 1,
            randomize: false,
            invert: false,
        }
    }
}

impl ProcessingConfig {
    /// 从命令行参数创建配置
    pub fn from_args(args: &Args) -> Result<Self> {
        Ok(Self {
            input_path: args.input_path.clone(),
            output_path: args.output_path.clone(),
            output_format: args.output_format,
            preprocess: args.preprocess,
            verbose: args.verbose,
            show_position: args.show_position,
            min_confidence: args.min_confidence,
            save_processed: args.save_processed,
            processed_output_path: args.processed_output_path.clone(),
            brute_force: args.brute_force,
            expected_count: args.expected_count,
            randomize: args.randomize,
            invert: args.invert,
        })
    }
    
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        // 检查输入文件是否存在
        if !self.input_path.exists() {
            return Err(QRDecodeError::invalid_input(format!(
                "输入文件不存在: {}",
                self.input_path.display()
            )));
        }
        
        // 检查置信度阈值
        if self.min_confidence < 0.0 || self.min_confidence > 1.0 {
            return Err(QRDecodeError::invalid_input(
                "置信度阈值必须在 0.0 到 1.0 之间".to_string(),
            ));
        }
        
        Ok(())
    }
}

/// 图像处理参数
#[derive(Debug, Clone)]
pub struct ImageProcessingParams {
    /// 是否转换为灰度图
    pub to_grayscale: bool,
    /// 是否应用高斯模糊
    pub gaussian_blur: bool,
    /// 高斯模糊核大小
    pub blur_kernel_size: i32,
    /// 是否应用直方图均衡化
    pub histogram_equalization: bool,
    /// 是否应用自适应阈值
    pub adaptive_threshold: bool,
    /// 缩放因子
    pub scale_factor: f64,
}

impl Default for ImageProcessingParams {
    fn default() -> Self {
        Self {
            to_grayscale: true,
            gaussian_blur: true,
            blur_kernel_size: 3,
            histogram_equalization: true,
            adaptive_threshold: false,
            scale_factor: 1.0,
        }
    }
}