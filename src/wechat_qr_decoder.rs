//! WeChat QR Code 解码器模块
//!
//! 使用 WeChat 提供的 CNN 模型实现高精度二维码检测和解码功能。
//! 包含检测模型和超分辨率模型，能够处理小尺寸和复杂环境下的二维码。

use opencv::{
    core::{Mat, Point2f, Vector},
    wechat_qrcode::WeChatQRCode,
    prelude::*,
};
use std::path::Path;

use crate::error::{QRDecodeError, Result};
use crate::types::{ProcessingConfig, QRCodeResult, QRPosition};

/// WeChat QR Code 解码器
pub struct WeChatQRDecoder {
    /// 处理配置
    config: ProcessingConfig,
    /// WeChat QR Code 检测器
    detector: WeChatQRCode,
    /// 模型是否已加载
    model_loaded: bool,
}

impl WeChatQRDecoder {
    /// 创建新的 WeChat QR Code 解码器
    pub fn new(config: &ProcessingConfig) -> Result<Self> {
        // 模型文件路径
        let detect_prototxt = "models/detect.prototxt";
        let detect_caffemodel = "models/detect.caffemodel";
        let sr_prototxt = "models/sr.prototxt";
        let sr_caffemodel = "models/sr.caffemodel";
        
        // 检查模型文件是否存在
        if !Path::new(detect_prototxt).exists() {
            return Err(QRDecodeError::decode_error(format!(
                "检测模型文件不存在: {}", detect_prototxt
            )));
        }
        
        if !Path::new(detect_caffemodel).exists() {
            return Err(QRDecodeError::decode_error(format!(
                "检测模型权重文件不存在: {}", detect_caffemodel
            )));
        }
        
        if !Path::new(sr_prototxt).exists() {
            return Err(QRDecodeError::decode_error(format!(
                "超分辨率模型文件不存在: {}", sr_prototxt
            )));
        }
        
        if !Path::new(sr_caffemodel).exists() {
            return Err(QRDecodeError::decode_error(format!(
                "超分辨率模型权重文件不存在: {}", sr_caffemodel
            )));
        }
        
        // 创建 WeChat QR Code 检测器
        let detector = WeChatQRCode::new(
            detect_prototxt,
            detect_caffemodel,
            sr_prototxt,
            sr_caffemodel,
        ).map_err(|e| QRDecodeError::decode_error(format!(
            "无法创建 WeChat QR Code 检测器: {}", e
        )))?;
        
        if config.verbose {
            println!("✅ WeChat QR Code 模型加载成功");
            println!("   - 检测模型: {}", detect_prototxt);
            println!("   - 检测权重: {}", detect_caffemodel);
            println!("   - 超分辨率模型: {}", sr_prototxt);
            println!("   - 超分辨率权重: {}", sr_caffemodel);
        }
        
        Ok(Self {
            config: config.clone(),
            detector,
            model_loaded: true,
        })
    }
    
    /// 检测并解码图像中的所有二维码
    pub fn decode_qr_codes(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>> {
        if !self.model_loaded {
            return Err(QRDecodeError::decode_error("模型未加载".to_string()));
        }
        
        if self.config.verbose {
            println!("🔍 使用 WeChat QR Code 模型进行检测和解码...");
        }
        
        let mut points = Vector::<Mat>::new();
        
        // 使用 WeChat QR Code 检测器进行检测和解码
        let decoded_infos = self.detector
            .detect_and_decode(image, &mut points)
            .map_err(|e| QRDecodeError::decode_error(format!(
                "WeChat QR Code 检测失败: {}", e
            )))?;
        
        if decoded_infos.is_empty() {
            if self.config.verbose {
                println!("❌ 未检测到二维码");
            }
            return Ok(Vec::new());
        }
        
        let mut results = Vec::new();
        
        // 处理每个检测到的二维码
        for i in 0..decoded_infos.len() {
            let decoded_info = decoded_infos.get(i)
                .map_err(|e| QRDecodeError::decode_error(format!(
                    "获取解码信息失败: {}", e
                )))?;
            
            if decoded_info.is_empty() {
                continue;
            }
            
            // 获取对应的角点
            let qr_points = points.get(i)
                .map_err(|e| QRDecodeError::decode_error(format!(
                    "获取角点信息失败: {}", e
                )))?;
            
            // 转换角点格式
            let corner_points = self.extract_corner_points(&qr_points)?;
            
            // 计算位置信息
            let position = self.calculate_position_from_corners(&corner_points)?;
            
            // 计算置信度（WeChat 模型通常有更高的准确性）
            let confidence = self.calculate_confidence(&corner_points, image)?;
            
            let result = QRCodeResult::new(
                decoded_info,
                position,
                confidence,
                "WECHAT_QR_CODE".to_string(),
            );
            
            results.push(result);
        }
        
        // 过滤低置信度结果
        let filtered_results: Vec<QRCodeResult> = results
            .into_iter()
            .filter(|result| result.confidence >= self.config.min_confidence)
            .collect();
        
        if self.config.verbose {
            if filtered_results.is_empty() {
                println!("❌ 未检测到符合置信度要求的二维码");
            } else {
                println!("✅ 检测到 {} 个二维码", filtered_results.len());
                for (i, result) in filtered_results.iter().enumerate() {
                    println!("   QR {} - 置信度: {:.2}, 内容长度: {} 字符", 
                        i + 1, result.confidence, result.content.len());
                }
            }
        }
        
        Ok(filtered_results)
    }
    
    /// 提取角点坐标
    fn extract_corner_points(&self, points_mat: &Mat) -> Result<Vec<(f32, f32)>> {
        let mut corners = Vec::new();
        
        // WeChat QR Code 返回的角点格式可能不同，需要适配
        let rows = points_mat.rows();
        let cols = points_mat.cols();
        
        if cols == 2 {
            // 如果是 Nx2 的矩阵，每行是一个点的 (x, y) 坐标
            for i in 0..rows {
                let x: f32 = *points_mat.at_2d(i, 0)
                    .map_err(|e| QRDecodeError::decode_error(format!(
                        "提取角点 x 坐标失败: {}", e
                    )))?;
                let y: f32 = *points_mat.at_2d(i, 1)
                    .map_err(|e| QRDecodeError::decode_error(format!(
                        "提取角点 y 坐标失败: {}", e
                    )))?;
                corners.push((x, y));
            }
        } else {
            // 尝试其他格式
            for i in 0..rows {
                let point: Point2f = *points_mat.at_2d(i, 0)
                    .map_err(|e| QRDecodeError::decode_error(format!(
                        "提取角点失败: {}", e
                    )))?;
                corners.push((point.x, point.y));
            }
        }
        
        Ok(corners)
    }
    
    /// 从角点计算位置信息
    fn calculate_position_from_corners(&self, corners: &[(f32, f32)]) -> Result<QRPosition> {
        if corners.len() < 4 {
            return Err(QRDecodeError::decode_error("角点数量不足".to_string()));
        }
        
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        
        for &(x, y) in corners {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        
        let position = QRPosition::new(
            min_x as i32,
            min_y as i32,
            (max_x - min_x) as i32,
            (max_y - min_y) as i32,
        ).with_corners(corners.to_vec());
        
        Ok(position)
    }
    
    /// 计算置信度
    fn calculate_confidence(&self, corners: &[(f32, f32)], image: &Mat) -> Result<f32> {
        let mut confidence: f32 = 0.8; // WeChat 模型基础置信度更高
        
        // 基于角点数量调整置信度
        if corners.len() >= 4 {
            confidence += 0.1;
        }
        
        // 基于二维码区域大小调整置信度
        let area = self.calculate_area_from_corners(corners);
        if area > 100.0 {
            confidence += 0.05;
        }
        if area > 1000.0 {
            confidence += 0.05;
        }
        
        // 基于图像质量调整置信度
        let image_size = image.size()?;
        if image_size.width > 200 && image_size.height > 200 {
            confidence += 0.05;
        }
        
        Ok(confidence.min(1.0))
    }
    
    /// 从角点计算面积
    fn calculate_area_from_corners(&self, corners: &[(f32, f32)]) -> f32 {
        if corners.len() < 4 {
            return 0.0;
        }
        
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        
        for &(x, y) in corners {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        
        (max_x - min_x) * (max_y - min_y)
    }
    
    /// 检查模型是否已加载
    pub fn is_model_loaded(&self) -> bool {
        self.model_loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OutputFormat, ProcessingConfig};
    use std::path::PathBuf;
    
    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            input_path: PathBuf::from("test.jpg"),
            output_path: None,
            output_format: OutputFormat::Text,
            preprocess: false,
            verbose: false,
            show_position: false,
            min_confidence: 0.5,
            save_processed: false,
            processed_output_path: None,
        }
    }
    
    #[test]
    fn test_area_calculation() {
        let corners = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        let config = create_test_config();
        
        // 注意：这个测试需要模型文件存在才能运行
        if let Ok(decoder) = WeChatQRDecoder::new(&config) {
            let area = decoder.calculate_area_from_corners(&corners);
            assert_eq!(area, 10000.0);
        }
    }
}