//! QR Code Decoder Library
//! 
//! 这个库提供了基于 OpenCV 的二维码检测和解码功能。

pub mod cli;
pub mod error;
pub mod image_processor;
pub mod output;
pub mod qr_decoder;
pub mod types;
pub mod wechat_qr_decoder;
pub mod batch_processor;
pub mod enhanced_processor;
pub mod brute_force_decoder;


// 重新导出主要的公共接口
pub use cli::Args;
pub use error::QRDecodeError;
pub use image_processor::ImageProcessor;
pub use output::OutputFormatter;
pub use qr_decoder::QRDecoder;
pub use types::*;
pub use batch_processor::{BatchProcessor, BatchConfig, BatchResult};
pub use enhanced_processor::EnhancedImageProcessor;
pub use brute_force_decoder::BruteForceDecoder;


/// 库的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库的名称
pub const NAME: &str = env!("CARGO_PKG_NAME");