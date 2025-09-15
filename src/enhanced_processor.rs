//! 增强图像预处理模块
//! 
//! 基于 LoveLy-QRCode-Scanner 项目的思路，实现多种图像变换功能
//! 通过对图像进行不同形式的变换（亮度、对比度、模糊度等），
//! 提高二维码解码的成功率。

use opencv::{
    core::{Mat, Scalar, Size, CV_8UC1, CV_8UC3},
    imgproc::{
        cvt_color, gaussian_blur, COLOR_BGR2GRAY, COLOR_GRAY2BGR,
        bilateral_filter, median_blur, morphology_ex, MORPH_CLOSE, MORPH_OPEN,
        get_structuring_element, MORPH_RECT
    },
    prelude::*,
};
use std::collections::HashMap;

use crate::error::{QRDecodeError, Result};
use crate::qr_decoder::QRDecoder;
use crate::types::{QRCodeResult, ProcessingConfig};

/// 图像变换类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformType {
    /// 原始图像
    Original,
    /// 亮度调整
    Brightness(i32),
    /// 对比度调整
    Contrast(f64),
    /// 伽马校正
    Gamma(f64),
    /// 高斯模糊
    GaussianBlur(i32),
    /// 双边滤波
    BilateralFilter,
    /// 中值滤波
    MedianBlur(i32),
    /// 形态学操作 - 开运算
    MorphOpen,
    /// 形态学操作 - 闭运算
    MorphClose,
    /// 锐化
    Sharpen,
    /// 组合变换：亮度+对比度
    BrightnessContrast(i32, f64),
}

impl TransformType {
    /// 获取变换的描述
    pub fn description(&self) -> String {
        match self {
            TransformType::Original => "原始图像".to_string(),
            TransformType::Brightness(value) => format!("亮度调整: {}", value),
            TransformType::Contrast(value) => format!("对比度调整: {:.2}", value),
            TransformType::Gamma(value) => format!("伽马校正: {:.2}", value),
            TransformType::GaussianBlur(kernel) => format!("高斯模糊: {}x{}", kernel, kernel),
            TransformType::BilateralFilter => "双边滤波".to_string(),
            TransformType::MedianBlur(kernel) => format!("中值滤波: {}x{}", kernel, kernel),
            TransformType::MorphOpen => "形态学开运算".to_string(),
            TransformType::MorphClose => "形态学闭运算".to_string(),
            TransformType::Sharpen => "锐化".to_string(),
            TransformType::BrightnessContrast(b, c) => format!("亮度+对比度: {} / {:.2}", b, c),
        }
    }
}

/// 增强图像处理器
pub struct EnhancedImageProcessor {
    /// 处理配置
    config: ProcessingConfig,
    /// 解码器配置
    decoder_config: ProcessingConfig,
    /// 变换尝试统计
    transform_stats: HashMap<String, usize>,
}

impl EnhancedImageProcessor {
    /// 创建新的增强图像处理器
    pub fn new(config: ProcessingConfig) -> Result<Self> {
        let decoder_config = config.clone();
        
        Ok(Self {
            config,
            decoder_config,
            transform_stats: HashMap::new(),
        })
    }
    
    /// 使用多种变换尝试解码二维码
    pub fn decode_with_transforms(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>> {
        if self.config.verbose {
            println!("🔄 开始增强图像预处理解码...");
        }
        
        // 定义要尝试的变换序列
        let transforms = self.get_transform_sequence();
        
        for (i, transform) in transforms.iter().enumerate() {
            if self.config.verbose {
                println!("   [{}/{}] 尝试变换: {}", i + 1, transforms.len(), transform.description());
            }
            
            // 应用变换
            match self.apply_transform(image, *transform) {
                Ok(transformed_image) => {
                    // 创建新的解码器实例并尝试解码变换后的图像
                    let mut decoder = QRDecoder::new(&self.decoder_config);
                    match decoder.decode_qr_codes(&transformed_image) {
                        Ok(results) if !results.is_empty() => {
                            // 记录成功的变换
                            *self.transform_stats.entry(transform.description()).or_insert(0) += 1;
                            
                            if self.config.verbose {
                                println!("   ✅ 解码成功! 找到 {} 个二维码", results.len());
                                for (j, result) in results.iter().enumerate() {
                                    println!("      [{}] 内容: {} (置信度: {:.2})", 
                                           j + 1, result.content, result.confidence);
                                }
                            }
                            
                            return Ok(results);
                        }
                        Ok(_) => {
                            if self.config.verbose {
                                println!("   ❌ 未找到二维码");
                            }
                        }
                        Err(e) => {
                            if self.config.verbose {
                                println!("   ❌ 解码错误: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    if self.config.verbose {
                        println!("   ❌ 变换失败: {}", e);
                    }
                }
            }
        }
        
        if self.config.verbose {
            println!("❌ 所有变换尝试均失败");
        }
        
        Ok(vec![])
    }
    
    /// 获取变换序列
    /// 基于 QReader 和 LoveLy-QRCode-Scanner 的优化策略
    fn get_transform_sequence(&self) -> Vec<TransformType> {
        vec![
            // 1. 首先尝试原始图像
            TransformType::Original,
            
            // 2. 轻微调整系列（最常见的成功案例）
            TransformType::Brightness(20),
            TransformType::Brightness(-20),
            TransformType::Contrast(1.2),
            TransformType::Contrast(0.8),
            TransformType::Gamma(0.8),
            TransformType::Gamma(1.2),
            
            // 3. 组合轻微调整（高成功率）
            TransformType::BrightnessContrast(15, 1.3),
            TransformType::BrightnessContrast(-15, 1.3),
            TransformType::BrightnessContrast(25, 0.7),
            TransformType::BrightnessContrast(-25, 0.7),
            
            // 4. 中等强度调整
            TransformType::Brightness(40),
            TransformType::Brightness(-40),
            TransformType::Contrast(1.5),
            TransformType::Contrast(0.6),
            TransformType::Gamma(0.5),
            TransformType::Gamma(1.5),
            
            // 5. 滤波和降噪（对模糊图像有效）
            TransformType::BilateralFilter,
            TransformType::MedianBlur(3),
            TransformType::GaussianBlur(3),
            TransformType::MedianBlur(5),
            
            // 6. 锐化（对模糊二维码特别有效）
            TransformType::Sharpen,
            
            // 7. 形态学操作（对噪声图像有效）
            TransformType::MorphOpen,
            TransformType::MorphClose,
            
            // 8. 强烈调整（最后尝试）
            TransformType::Brightness(60),
            TransformType::Brightness(-60),
            TransformType::Contrast(2.0),
            TransformType::Contrast(0.4),
            TransformType::Gamma(0.3),
            TransformType::Gamma(2.2),
            
            // 9. 极端组合变换
            TransformType::BrightnessContrast(50, 1.8),
            TransformType::BrightnessContrast(-50, 1.8),
            TransformType::BrightnessContrast(40, 0.5),
            TransformType::BrightnessContrast(-40, 0.5),
            
            // 10. 模糊处理的最后尝试
            TransformType::GaussianBlur(5),
            TransformType::GaussianBlur(7),
        ]
    }
    
    /// 应用指定的变换
    fn apply_transform(&self, image: &Mat, transform: TransformType) -> Result<Mat> {
        match transform {
            TransformType::Original => Ok(image.clone()),
            TransformType::Brightness(value) => self.adjust_brightness(image, value),
            TransformType::Contrast(value) => self.adjust_contrast(image, value),
            TransformType::Gamma(value) => self.apply_gamma_correction(image, value),
            TransformType::GaussianBlur(kernel_size) => self.apply_gaussian_blur(image, kernel_size),
            TransformType::BilateralFilter => self.apply_bilateral_filter(image),
            TransformType::MedianBlur(kernel_size) => self.apply_median_blur(image, kernel_size),
            TransformType::MorphOpen => self.apply_morphology_open(image),
            TransformType::MorphClose => self.apply_morphology_close(image),
            TransformType::Sharpen => self.apply_sharpen(image),
            TransformType::BrightnessContrast(brightness, contrast) => {
                let temp = self.adjust_brightness(image, brightness)?;
                self.adjust_contrast(&temp, contrast)
            }
        }
    }
    
    /// 调整亮度
    fn adjust_brightness(&self, image: &Mat, value: i32) -> Result<Mat> {
        let mut result = Mat::default();
        let scalar = Scalar::all(value as f64);
        
        opencv::core::add(image, &scalar, &mut result, &opencv::core::no_array(), -1)
            .map_err(|e| QRDecodeError::image_processing_error(format!("亮度调整失败: {}", e)))?;
        
        Ok(result)
    }
    
    /// 调整对比度
    fn adjust_contrast(&self, image: &Mat, alpha: f64) -> Result<Mat> {
        let mut result = Mat::default();
        
        opencv::core::multiply(image, &Scalar::all(alpha), &mut result, 1.0, -1)
            .map_err(|e| QRDecodeError::image_processing_error(format!("对比度调整失败: {}", e)))?;
        
        Ok(result)
    }
    
    /// 应用伽马校正
    fn apply_gamma_correction(&self, image: &Mat, gamma: f64) -> Result<Mat> {
        let mut result = Mat::default();
        
        // 归一化到 0-1 范围
        let mut normalized = Mat::default();
        image.convert_to(&mut normalized, opencv::core::CV_32F, 1.0 / 255.0, 0.0)
            .map_err(|e| QRDecodeError::image_processing_error(format!("归一化失败: {}", e)))?;
        
        // 应用伽马校正
        opencv::core::pow(&normalized, gamma, &mut result)
            .map_err(|e| QRDecodeError::image_processing_error(format!("伽马校正失败: {}", e)))?;
        
        // 转换回 0-255 范围
        let mut final_result = Mat::default();
        result.convert_to(&mut final_result, opencv::core::CV_8U, 255.0, 0.0)
            .map_err(|e| QRDecodeError::image_processing_error(format!("反归一化失败: {}", e)))?;
        
        Ok(final_result)
    }
    
    /// 应用高斯模糊
    fn apply_gaussian_blur(&self, image: &Mat, kernel_size: i32) -> Result<Mat> {
        let mut result = Mat::default();
        let kernel_size = if kernel_size % 2 == 0 { kernel_size + 1 } else { kernel_size };
        let size = Size::new(kernel_size, kernel_size);
        
        gaussian_blur(image, &mut result, size, 0.0, 0.0, opencv::core::BORDER_DEFAULT, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
            .map_err(|e| QRDecodeError::image_processing_error(format!("高斯模糊失败: {}", e)))?;
        
        Ok(result)
    }
    
    /// 应用双边滤波
    fn apply_bilateral_filter(&self, image: &Mat) -> Result<Mat> {
        let mut result = Mat::default();
        
        bilateral_filter(image, &mut result, 9, 75.0, 75.0, opencv::core::BORDER_DEFAULT)
            .map_err(|e| QRDecodeError::image_processing_error(format!("双边滤波失败: {}", e)))?;
        
        Ok(result)
    }
    
    /// 应用中值滤波
    fn apply_median_blur(&self, image: &Mat, kernel_size: i32) -> Result<Mat> {
        let mut result = Mat::default();
        let kernel_size = if kernel_size % 2 == 0 { kernel_size + 1 } else { kernel_size };
        
        median_blur(image, &mut result, kernel_size)
            .map_err(|e| QRDecodeError::image_processing_error(format!("中值滤波失败: {}", e)))?;
        
        Ok(result)
    }
    
    /// 应用形态学开运算
    fn apply_morphology_open(&self, image: &Mat) -> Result<Mat> {
        // 转换为灰度图
        let gray = self.to_grayscale_if_needed(image)?;
        
        let mut result = Mat::default();
        let kernel = get_structuring_element(MORPH_RECT, Size::new(3, 3), opencv::core::Point::new(-1, -1))
            .map_err(|e| QRDecodeError::image_processing_error(format!("创建形态学核失败: {}", e)))?;
        
        morphology_ex(&gray, &mut result, MORPH_OPEN, &kernel, opencv::core::Point::new(-1, -1), 1, opencv::core::BORDER_CONSTANT, opencv::imgproc::morphology_default_border_value()?)
            .map_err(|e| QRDecodeError::image_processing_error(format!("形态学开运算失败: {}", e)))?;
        
        // 如果原图是彩色的，转换回彩色
        if image.channels() == 3 {
            let mut color_result = Mat::default();
            cvt_color(&result, &mut color_result, COLOR_GRAY2BGR, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                .map_err(|e| QRDecodeError::image_processing_error(format!("灰度转彩色失败: {}", e)))?;
            Ok(color_result)
        } else {
            Ok(result)
        }
    }
    
    /// 应用形态学闭运算
    fn apply_morphology_close(&self, image: &Mat) -> Result<Mat> {
        // 转换为灰度图
        let gray = self.to_grayscale_if_needed(image)?;
        
        let mut result = Mat::default();
        let kernel = get_structuring_element(MORPH_RECT, Size::new(3, 3), opencv::core::Point::new(-1, -1))
            .map_err(|e| QRDecodeError::image_processing_error(format!("创建形态学核失败: {}", e)))?;
        
        morphology_ex(&gray, &mut result, MORPH_CLOSE, &kernel, opencv::core::Point::new(-1, -1), 1, opencv::core::BORDER_CONSTANT, opencv::imgproc::morphology_default_border_value()?)
            .map_err(|e| QRDecodeError::image_processing_error(format!("形态学闭运算失败: {}", e)))?;
        
        // 如果原图是彩色的，转换回彩色
        if image.channels() == 3 {
            let mut color_result = Mat::default();
            cvt_color(&result, &mut color_result, COLOR_GRAY2BGR, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                .map_err(|e| QRDecodeError::image_processing_error(format!("灰度转彩色失败: {}", e)))?;
            Ok(color_result)
        } else {
            Ok(result)
        }
    }
    
    /// 应用锐化
    fn apply_sharpen(&self, image: &Mat) -> Result<Mat> {
        // 锐化核
        let kernel_data: [f32; 9] = [
            0.0, -1.0, 0.0,
            -1.0, 5.0, -1.0,
            0.0, -1.0, 0.0
        ];
        
        let kernel = Mat::new_rows_cols_with_data(3, 3, &kernel_data)
            .map_err(|e| QRDecodeError::image_processing_error(format!("创建锐化核失败: {}", e)))?;
        
        let mut result = Mat::default();
        opencv::imgproc::filter_2d(image, &mut result, -1, &kernel, opencv::core::Point::new(-1, -1), 0.0, opencv::core::BORDER_DEFAULT)
            .map_err(|e| QRDecodeError::image_processing_error(format!("锐化失败: {}", e)))?;
        
        Ok(result)
    }
    
    /// 如果需要，转换为灰度图
    fn to_grayscale_if_needed(&self, image: &Mat) -> Result<Mat> {
        if image.channels() == 1 {
            Ok(image.clone())
        } else {
            let mut gray = Mat::default();
            cvt_color(image, &mut gray, COLOR_BGR2GRAY, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                .map_err(|e| QRDecodeError::image_processing_error(format!("灰度转换失败: {}", e)))?;
            Ok(gray)
        }
    }
    
    /// 获取变换统计信息
    pub fn get_transform_stats(&self) -> &HashMap<String, usize> {
        &self.transform_stats
    }
    
    /// 打印变换统计信息
    pub fn print_transform_stats(&self) {
        if self.transform_stats.is_empty() {
            println!("📊 暂无变换统计信息");
            return;
        }
        
        println!("📊 变换成功统计:");
        let mut stats: Vec<_> = self.transform_stats.iter().collect();
        stats.sort_by(|a, b| b.1.cmp(a.1));
        
        for (transform, count) in stats {
            println!("   {} : {} 次", transform, count);
        }
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
    fn test_transform_type_description() {
        assert_eq!(TransformType::Original.description(), "原始图像");
        assert_eq!(TransformType::Brightness(30).description(), "亮度调整: 30");
        assert_eq!(TransformType::Contrast(1.5).description(), "对比度调整: 1.50");
    }
    
    #[test]
    fn test_transform_sequence() {
        let config = create_test_config();
        let processor = EnhancedImageProcessor::new(config).unwrap();
        let transforms = processor.get_transform_sequence();
        
        assert!(!transforms.is_empty());
        assert_eq!(transforms[0], TransformType::Original);
    }
}