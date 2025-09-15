//! 暴力破解解码器模块
//! 基于 Cli_AutoVer.py 的逻辑实现，支持多种图像变换组合进行暴力破解解码

use opencv::{
    core::{Mat, Point2f, Scalar, Size, Vector},
    imgproc::{self, THRESH_BINARY, THRESH_OTSU, INTER_LINEAR},
    prelude::*,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;

use crate::{
    error::QRDecodeError,
    types::{QRCodeResult, QRPosition},
    wechat_qr_decoder::WeChatQRDecoder,
};

/// 暴力破解配置
#[derive(Debug, Clone)]
pub struct BruteForceConfig {
    /// 对比度选项 [2, 1, 3]
    pub contrast_options: Vec<f64>,
    /// 亮度选项 [-75, 75, -50, -25, -10, 0, 25, 50]
    pub brightness_options: Vec<i32>,
    /// 模糊选项 [-7, -3, 7, 3, -1, 5, 9, 11, 13, 15, 17, 19, 21, 23, 25]
    pub blur_options: Vec<i32>,
    /// 缩放选项 [0.2, 0.5, 0.7, 0.9, 1.3, 2.0]
    pub scale_options: Vec<f64>,
    /// 重复检测距离阈值
    pub duplicate_threshold: f64,
    /// 是否随机化参数组合
    pub randomize: bool,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            contrast_options: vec![2.0, 1.0, 3.0],
            brightness_options: vec![-75, 75, -50, -25, -10, 0, 25, 50],
            blur_options: vec![-7, -3, 7, 3, -1, 5, 9, 11, 13, 15, 17, 19, 21, 23, 25],
            scale_options: vec![0.2, 0.5, 0.7, 0.9, 1.3, 2.0],
            duplicate_threshold: 10.0,
            randomize: false,
        }
    }
}

/// 变换参数
#[derive(Debug, Clone)]
pub struct TransformParams {
    pub contrast: f64,
    pub brightness: i32,
    pub blur: i32,
    pub scale: f64,
    pub grayscale: bool,
    pub binary: bool,
}

/// 暴力破解解码器
pub struct BruteForceDecoder {
    config: BruteForceConfig,
    decoder: WeChatQRDecoder,
}

impl BruteForceDecoder {
    /// 创建新的暴力破解解码器
    pub fn new() -> Result<Self, QRDecodeError> {
        // 创建默认的处理配置
        let processing_config = crate::types::ProcessingConfig::default();
        let decoder = WeChatQRDecoder::new(&processing_config)
            .map_err(|e| QRDecodeError::decode_error(format!("创建解码器失败: {:?}", e)))?;
        Ok(Self {
            config: BruteForceConfig::default(),
            decoder,
        })
    }

    /// 从文件路径解码二维码（批量处理接口）
    pub fn decode_with_brute_force(
        &mut self,
        file_path: &std::path::Path,
        expected_count: usize,
        randomize: bool,
    ) -> Result<Vec<crate::types::QrResult>, QRDecodeError> {
        // 设置随机化选项
        self.config.randomize = randomize;
        
        // 加载图像
        let image = opencv::imgcodecs::imread(
            &file_path.to_string_lossy(),
            opencv::imgcodecs::IMREAD_COLOR,
        ).map_err(|e| QRDecodeError::decode_error(format!("加载图像失败: {}", e)))?;
        
        if image.empty() {
            return Err(QRDecodeError::invalid_input("图像为空".to_string()));
        }
        
        // 执行暴力破解解码
        let qr_results = self.detect_and_decode(&image)
            .map_err(|e| QRDecodeError::decode_error(format!("解码失败: {:?}", e)))?;
        
        // 转换结果格式
        let mut results = Vec::new();
        for qr_result in qr_results {
            let result = crate::types::QrResult {
                content: qr_result.content,
                points: Some(vec![
                    (qr_result.position.x as f32, qr_result.position.y as f32),
                    (qr_result.position.x as f32 + qr_result.position.width as f32, qr_result.position.y as f32),
                    (qr_result.position.x as f32 + qr_result.position.width as f32, qr_result.position.y as f32 + qr_result.position.height as f32),
                    (qr_result.position.x as f32, qr_result.position.y as f32 + qr_result.position.height as f32),
                ]),
            };
            results.push(result);
        }
        
        Ok(results)
    }
    
    // 重复检测机制 - 基于坐标距离阈值
    fn is_duplicate(&self, new_result: &QRCodeResult, existing_results: &[QRCodeResult]) -> bool {
        const DISTANCE_THRESHOLD: f64 = 50.0; // 距离阈值，匹配Python版本
        
        for existing in existing_results {
            // 计算中心点距离
            let dx = (new_result.position.x - existing.position.x) as f64;
            let dy = (new_result.position.y - existing.position.y) as f64;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < DISTANCE_THRESHOLD {
                return true;
            }
        }
        false
    }

    /// 生成所有参数组合
    fn generate_param_combinations(&self) -> Vec<TransformParams> {
        let mut combinations = Vec::new();
        
        // 完全匹配Python版本的参数范围
        let contrast_options = vec![2.0, 1.0, 3.0];
        let brightness_options = vec![-75, 75, -50, -25, -10, 0, 25, 50];
        let blur_options = vec![-7, -3, 7, 3, -1, 5, 9, 11, 13, 15, 17, 19, 21, 23, 25];
        let scale_options = vec![0.2, 0.5, 0.7, 0.9, 1.3, 2.0];
        
        for &scale in &scale_options {
            for &grayscale in &[true] { // Python版本固定使用灰度
                for &contrast in &contrast_options {
                    for &brightness in &brightness_options {
                        for &blur in &blur_options {
                            for &binary in &[true, false] {
                                combinations.push(TransformParams {
                                    contrast,
                                    brightness,
                                    blur,
                                    scale,
                                    grayscale,
                                    binary,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        combinations
    }

    /// 应用图像变换
    fn apply_transform(
        &self,
        image: &Mat,
        params: &TransformParams,
        invert: bool,
    ) -> Result<Mat, QRDecodeError> {
        let mut result = image.clone();
        
        // 缩放处理
        if params.scale != 1.0 {
            let new_size = opencv::core::Size::new(
                (result.cols() as f64 * params.scale) as i32,
                (result.rows() as f64 * params.scale) as i32,
            );
            let mut temp = opencv::core::Mat::default();
            opencv::imgproc::resize(&result, &mut temp, new_size, 0.0, 0.0, opencv::imgproc::INTER_LINEAR)
                .map_err(|e| QRDecodeError::image_processing_error(format!("缩放处理失败: {}", e)))?;
            result = temp;
        }
        
        // 亮度和对比度调整
        let mut temp = Mat::default();
        result.convert_to(&mut temp, -1, params.contrast, params.brightness as f64)
             .map_err(|e| QRDecodeError::image_processing_error(format!("亮度对比度调整失败: {}", e)))?;
        result = temp;
        
        // 模糊处理
        if params.blur != 0 {
            let kernel_size = params.blur.abs();
            if kernel_size > 1 {
                let ksize = Size::new(kernel_size, kernel_size);
                let mut temp = Mat::default();
                imgproc::gaussian_blur(&result, &mut temp, ksize, 0.0, 0.0, opencv::core::BORDER_DEFAULT, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                     .map_err(|e| QRDecodeError::image_processing_error(format!("模糊处理失败: {}", e)))?;
                result = temp;
            }
        }
        
        // 灰度转换
        if params.grayscale {
            let mut temp = Mat::default();
            imgproc::cvt_color(&result, &mut temp, imgproc::COLOR_BGR2GRAY, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                 .map_err(|e| QRDecodeError::image_processing_error(format!("灰度转换失败: {}", e)))?;
            result = temp;
        }
        
        // 二值化处理 (使用THRESH_BINARY | THRESH_OTSU匹配Python版本)
        if params.binary {
            let mut temp = opencv::core::Mat::default();
            opencv::imgproc::threshold(&result, &mut temp, 0.0, 255.0, 
                opencv::imgproc::THRESH_BINARY | opencv::imgproc::THRESH_OTSU)
                .map_err(|e| QRDecodeError::image_processing_error(format!("二值化处理失败: {}", e)))?;
            result = temp;
        }
        
        // 反色处理
        if invert {
            let mut temp = Mat::default();
            opencv::core::bitwise_not(&result, &mut temp, &opencv::core::no_array())
                 .map_err(|e| QRDecodeError::image_processing_error(format!("反色处理失败: {}", e)))?;
            result = temp;
        }
        
        Ok(result)
    }



    /// 检测和解码二维码
    pub fn detect_and_decode(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>, QRDecodeError> {
        let mut all_results = Vec::new();
        let mut combinations = self.generate_param_combinations();
        
        // 随机化处理（如果启用）
        if self.config.randomize {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            combinations.shuffle(&mut rng);
        }
        
        println!("开始暴力破解，共{}种参数组合", combinations.len());
        
        for (i, params) in combinations.iter().enumerate() {
            if i % 100 == 0 {
                println!("进度: {}/{}", i, combinations.len());
            }
            
            match self.apply_transform(image, params, false) {
                Ok(processed_image) => {
                    match self.decoder.decode_qr_codes(&processed_image) {
                        Ok(results) => {
                            if !results.is_empty() {
                                println!("✅ 参数组合 {} 检测到 {} 个二维码 (scale:{}, contrast:{}, brightness:{}, blur:{}, binary:{})", 
                                    i, results.len(), params.scale, params.contrast, params.brightness, params.blur, params.binary);
                                
                                // 添加去重逻辑
                                for result in results {
                                    if !self.is_duplicate(&result, &all_results) {
                                        all_results.push(result);
                                    }
                                }
                                
                                // 找到二维码后立即返回结果，不再继续尝试其他参数组合
                                if !all_results.is_empty() {
                                    println!("🎯 成功找到 {} 个二维码，停止暴力破解", all_results.len());
                                    return Ok(all_results);
                                }
                            }
                        }
                        Err(_) => {} // 忽略解码错误
                    }
                }
                Err(_) => {} // 忽略变换错误
            }
        }
        
        // 如果所有参数组合都尝试完了还没找到二维码
        if all_results.is_empty() {
            println!("❌ 暴力破解完成，未找到任何二维码");
        }
        
        Ok(all_results)
    }
}