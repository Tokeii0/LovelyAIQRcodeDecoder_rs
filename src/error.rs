//! 错误处理模块
//! 
//! 定义了二维码解码过程中可能出现的各种错误类型。

use thiserror::Error;

/// 二维码解码过程中的错误类型
#[derive(Debug, Error)]
pub enum QRDecodeError {
    /// OpenCV 相关错误
    #[error("OpenCV 错误: {0}")]
    OpenCVError(#[from] opencv::Error),
    
    /// 文件 I/O 错误
    #[error("文件 I/O 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    /// 图像中未找到二维码
    #[error("图像中未找到二维码")]
    NoQRCodeFound,
    
    /// 二维码解码失败
    #[error("二维码解码失败: {0}")]
    DecodeError(String),
    
    /// 不支持的图像格式
    #[error("不支持的图像格式: {0}")]
    UnsupportedFormat(String),
    
    /// 无效的输入参数
    #[error("无效的输入参数: {0}")]
    InvalidInput(String),
    
    /// 图像处理错误
    #[error("图像处理错误: {0}")]
    ImageProcessingError(String),
    
    /// 输出格式化错误
    #[error("输出格式化错误: {0}")]
    OutputError(String),
    
    /// JSON 序列化错误
    #[error("JSON 序列化错误: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// 通用错误
    #[error("通用错误: {0}")]
    GenericError(#[from] anyhow::Error),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, QRDecodeError>;

impl QRDecodeError {
    /// 创建一个解码错误
    pub fn decode_error<S: Into<String>>(msg: S) -> Self {
        QRDecodeError::DecodeError(msg.into())
    }
    
    /// 创建一个图像处理错误
    pub fn image_processing_error<S: Into<String>>(msg: S) -> Self {
        QRDecodeError::ImageProcessingError(msg.into())
    }
    
    /// 创建一个输出错误
    pub fn output_error<S: Into<String>>(msg: S) -> Self {
        QRDecodeError::OutputError(msg.into())
    }
    
    /// 创建一个无效输入错误
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        QRDecodeError::InvalidInput(msg.into())
    }
}