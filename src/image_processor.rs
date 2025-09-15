//! 图像处理模块
//! 
//! 负责图像的加载、预处理和格式转换等功能。

use opencv::{
    core::{Mat, Size},
    imgcodecs::{imread, IMREAD_COLOR},
    imgproc::{
        cvt_color, gaussian_blur, resize, COLOR_BGR2GRAY, INTER_LINEAR,
        equalize_hist, adaptive_threshold, ADAPTIVE_THRESH_GAUSSIAN_C, THRESH_BINARY
    },
    prelude::*,
};
use std::path::Path;

use crate::error::{QRDecodeError, Result};
use crate::types::{ImageProcessingParams, ProcessingConfig};

/// 图像处理器
pub struct ImageProcessor {
    /// 处理配置
    config: ProcessingConfig,
    /// 图像处理参数
    params: ImageProcessingParams,
}

impl ImageProcessor {
    /// 创建新的图像处理器
    pub fn new(config: &ProcessingConfig) -> Self {
        Self {
            config: config.clone(),
            params: ImageProcessingParams::default(),
        }
    }
    
    /// 使用自定义参数创建图像处理器
    pub fn with_params(config: &ProcessingConfig, params: ImageProcessingParams) -> Self {
        Self {
            config: config.clone(),
            params,
        }
    }
    
    /// 从文件加载图像
    pub fn load_image<P: AsRef<Path>>(&self, path: P) -> Result<Mat> {
        let path_str = path.as_ref().to_string_lossy();
        
        // 检查文件是否存在
        if !path.as_ref().exists() {
            return Err(QRDecodeError::invalid_input(format!(
                "图像文件不存在: {}",
                path_str
            )));
        }
        
        // 检查文件扩展名
        if let Some(extension) = path.as_ref().extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            let supported_formats = vec!["jpg", "jpeg", "png", "bmp", "tiff", "tif", "webp"];
            
            if !supported_formats.contains(&ext.as_str()) {
                return Err(QRDecodeError::UnsupportedFormat(format!(
                    "不支持的图像格式: {}",
                    ext
                )));
            }
        }
        
        // 加载图像
        let image = imread(&path_str, IMREAD_COLOR)
            .map_err(|e| QRDecodeError::image_processing_error(format!(
                "无法加载图像 {}: {}",
                path_str, e
            )))?;
        
        if image.empty() {
            return Err(QRDecodeError::image_processing_error(format!(
                "加载的图像为空: {}",
                path_str
            )));
        }
        
        if self.config.verbose {
            let size = image.size()?;
            println!("✅ 成功加载图像: {} ({}x{})", path_str, size.width, size.height);
        }
        
        Ok(image)
    }
    
    /// 预处理图像
    pub fn preprocess_image(&self, image: &Mat) -> Result<Mat> {
        let mut processed = image.clone();
        
        if self.config.verbose {
            println!("🔄 开始图像预处理...");
        }
        
        // 1. 缩放图像
        if self.params.scale_factor != 1.0 {
            processed = self.resize_image(&processed, self.params.scale_factor)?;
            if self.config.verbose {
                println!("   ✓ 图像缩放: {}x", self.params.scale_factor);
            }
        }
        
        // 2. 转换为灰度图
        if self.params.to_grayscale {
            processed = self.convert_to_grayscale(&processed)?;
            if self.config.verbose {
                println!("   ✓ 转换为灰度图");
            }
        }
        
        // 3. 高斯模糊
        if self.params.gaussian_blur {
            processed = self.apply_gaussian_blur(&processed, self.params.blur_kernel_size)?;
            if self.config.verbose {
                println!("   ✓ 应用高斯模糊 (核大小: {})", self.params.blur_kernel_size);
            }
        }
        
        // 4. 直方图均衡化
        if self.params.histogram_equalization {
            processed = self.apply_histogram_equalization(&processed)?;
            if self.config.verbose {
                println!("   ✓ 直方图均衡化");
            }
        }
        
        // 5. 自适应阈值
        if self.params.adaptive_threshold {
            processed = self.apply_adaptive_threshold(&processed)?;
            if self.config.verbose {
                println!("   ✓ 自适应阈值处理");
            }
        }
        
        if self.config.verbose {
            println!("✅ 图像预处理完成");
        }
        
        Ok(processed)
    }
    
    /// 缩放图像
    pub fn resize_image(&self, image: &Mat, scale_factor: f64) -> Result<Mat> {
        let original_size = image.size()?;
        let new_width = (original_size.width as f64 * scale_factor) as i32;
        let new_height = (original_size.height as f64 * scale_factor) as i32;
        let new_size = Size::new(new_width, new_height);
        
        let mut resized = Mat::default();
        resize(image, &mut resized, new_size, 0.0, 0.0, INTER_LINEAR)
            .map_err(|e| QRDecodeError::image_processing_error(format!(
                "图像缩放失败: {}", e
            )))?;
        
        Ok(resized)
    }
    
    /// 转换为灰度图
    pub fn convert_to_grayscale(&self, image: &Mat) -> Result<Mat> {
        // 检查图像是否已经是灰度图
        if image.channels() == 1 {
            return Ok(image.clone());
        }
        
        let mut gray = Mat::default();
        cvt_color(image, &mut gray, COLOR_BGR2GRAY, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
            .map_err(|e| QRDecodeError::image_processing_error(format!(
                "灰度转换失败: {}", e
            )))?;
        
        Ok(gray)
    }
    
    /// 应用高斯模糊
    pub fn apply_gaussian_blur(&self, image: &Mat, kernel_size: i32) -> Result<Mat> {
        // 确保核大小为奇数
        let kernel_size = if kernel_size % 2 == 0 {
            kernel_size + 1
        } else {
            kernel_size
        };
        
        let mut blurred = Mat::default();
        let kernel_size = Size::new(kernel_size, kernel_size);
        
        gaussian_blur(image, &mut blurred, kernel_size, 0.0, 0.0, opencv::core::BORDER_DEFAULT, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
            .map_err(|e| QRDecodeError::image_processing_error(format!(
                "高斯模糊失败: {}", e
            )))?;
        
        Ok(blurred)
    }
    
    /// 应用直方图均衡化
    pub fn apply_histogram_equalization(&self, image: &Mat) -> Result<Mat> {
        // 确保图像是灰度图
        let gray_image = if image.channels() != 1 {
            self.convert_to_grayscale(image)?
        } else {
            image.clone()
        };
        
        let mut equalized = Mat::default();
        equalize_hist(&gray_image, &mut equalized)
            .map_err(|e| QRDecodeError::image_processing_error(format!(
                "直方图均衡化失败: {}", e
            )))?;
        
        Ok(equalized)
    }
    
    /// 应用自适应阈值
    pub fn apply_adaptive_threshold(&self, image: &Mat) -> Result<Mat> {
        // 确保图像是灰度图
        let gray_image = if image.channels() != 1 {
            self.convert_to_grayscale(image)?
        } else {
            image.clone()
        };
        
        let mut threshold = Mat::default();
        adaptive_threshold(
            &gray_image,
            &mut threshold,
            255.0,
            ADAPTIVE_THRESH_GAUSSIAN_C,
            THRESH_BINARY,
            11,
            2.0,
        )
        .map_err(|e| QRDecodeError::image_processing_error(format!(
            "自适应阈值处理失败: {}", e
        )))?;
        
        Ok(threshold)
    }
    
    /// 保存图像到文件
    pub fn save_image<P: AsRef<Path>>(&self, image: &Mat, path: P) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy();
        
        opencv::imgcodecs::imwrite(&path_str, image, &opencv::core::Vector::new())
            .map_err(|e| QRDecodeError::image_processing_error(format!(
                "保存图像失败 {}: {}",
                path_str, e
            )))?;
        
        if self.config.verbose {
            println!("💾 图像已保存到: {}", path_str);
        }
        
        Ok(())
    }
    
    /// 获取图像信息
    pub fn get_image_info(&self, image: &Mat) -> Result<ImageInfo> {
        let size = image.size()?;
        let channels = image.channels();
        let depth = image.depth();
        let total_pixels = (size.width * size.height) as usize;
        
        Ok(ImageInfo {
            width: size.width,
            height: size.height,
            channels,
            depth,
            total_pixels,
        })
    }
    
    /// 验证图像是否适合二维码检测
    pub fn validate_for_qr_detection(&self, image: &Mat) -> Result<()> {
        let info = self.get_image_info(image)?;
        
        // 检查图像尺寸
        if info.width < 50 || info.height < 50 {
            return Err(QRDecodeError::image_processing_error(
                "图像尺寸太小，无法进行二维码检测 (最小 50x50 像素)".to_string(),
            ));
        }
        
        // 检查图像是否过大
        if info.total_pixels > 50_000_000 {
            return Err(QRDecodeError::image_processing_error(
                "图像尺寸过大，建议缩小后再处理".to_string(),
            ));
        }
        
        Ok(())
    }
}

/// 图像信息结构
#[derive(Debug, Clone)]
pub struct ImageInfo {
    /// 图像宽度
    pub width: i32,
    /// 图像高度
    pub height: i32,
    /// 通道数
    pub channels: i32,
    /// 像素深度
    pub depth: i32,
    /// 总像素数
    pub total_pixels: usize,
}

impl ImageInfo {
    /// 获取图像的宽高比
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
    
    /// 检查是否为灰度图
    pub fn is_grayscale(&self) -> bool {
        self.channels == 1
    }
    
    /// 检查是否为彩色图
    pub fn is_color(&self) -> bool {
        self.channels >= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProcessingConfig;
    use std::path::PathBuf;
    
    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            input_path: PathBuf::from("test.jpg"),
            output_path: None,
            output_format: crate::types::OutputFormat::Text,
            preprocess: true,
            verbose: false,
            show_position: false,
            min_confidence: 0.5,
            save_processed: false,
            processed_output_path: None,
        }
    }
    
    #[test]
    fn test_image_processor_creation() {
        let config = create_test_config();
        let processor = ImageProcessor::new(&config);
        assert_eq!(processor.params.to_grayscale, true);
        assert_eq!(processor.params.gaussian_blur, true);
    }
    
    #[test]
    fn test_image_info() {
        let info = ImageInfo {
            width: 800,
            height: 600,
            channels: 3,
            depth: 8,
            total_pixels: 480000,
        };
        
        assert!((info.aspect_ratio() - 1.333).abs() < 0.01);
        assert!(info.is_color());
        assert!(!info.is_grayscale());
    }
}