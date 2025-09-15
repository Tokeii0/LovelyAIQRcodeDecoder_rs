//! 二维码检测和解码核心模块
//! 
//! 使用 OpenCV 的 QRCodeDetector 实现二维码的检测和解码功能。

use opencv::{
    core::{Mat, Point2f, Vector},
    objdetect::QRCodeDetector,
    prelude::*,
};
use std::collections::HashMap;

use crate::error::{QRDecodeError, Result};
use crate::types::{ProcessingConfig, QRCodeResult, QRPosition};
use crate::wechat_qr_decoder::WeChatQRDecoder;

/// 二维码解码器
pub struct QRDecoder {
    /// 处理配置
    config: ProcessingConfig,
    /// OpenCV QR 码检测器
    detector: QRCodeDetector,
    /// WeChat QR 码解码器（可选）
    wechat_decoder: Option<WeChatQRDecoder>,
    /// 解码统计信息
    stats: DecodingStats,
}

impl QRDecoder {
    /// 创建新的二维码解码器
    pub fn new(config: &ProcessingConfig) -> Self {
        let detector = QRCodeDetector::default().expect("无法创建 QRCodeDetector");
        
        // 尝试创建 WeChat QR 解码器
        let wechat_decoder = match WeChatQRDecoder::new(config) {
            Ok(decoder) => {
                if config.verbose {
                    println!("✅ WeChat QR Code 模型已启用");
                }
                Some(decoder)
            }
            Err(e) => {
                if config.verbose {
                    println!("⚠️  WeChat QR Code 模型加载失败，使用标准解码器: {}", e);
                }
                None
            }
        };
        
        Self {
            config: config.clone(),
            detector,
            wechat_decoder,
            stats: DecodingStats::new(),
        }
    }
    
    /// 检测并解码图像中的所有二维码
    pub fn decode_qr_codes(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>> {
        if self.config.verbose {
            println!("🔍 开始二维码检测和解码...");
        }
        
        let mut results = Vec::new();
        
        // 优先使用 WeChat 解码器
        if let Some(ref mut wechat_decoder) = self.wechat_decoder {
            if self.config.verbose {
                println!("🚀 使用 WeChat QR Code 模型进行检测...");
            }
            
            match wechat_decoder.decode_qr_codes(image) {
                Ok(wechat_results) => {
                    if !wechat_results.is_empty() {
                        results.extend(wechat_results);
                        if self.config.verbose {
                            println!("✅ WeChat 模型检测成功");
                        }
                    } else {
                        if self.config.verbose {
                            println!("⚠️  WeChat 模型未检测到二维码，尝试标准解码器...");
                        }
                        // WeChat 解码器未检测到，使用标准解码器
                        results.extend(self.fallback_decode(image)?);
                    }
                }
                Err(e) => {
                    if self.config.verbose {
                        println!("⚠️  WeChat 解码失败: {}，使用标准解码器...", e);
                    }
                    // WeChat 解码器失败，使用标准解码器
                    results.extend(self.fallback_decode(image)?);
                }
            }
        } else {
            // 没有 WeChat 解码器，使用标准解码器
            if self.config.verbose {
                println!("📷 使用标准 OpenCV 解码器...");
            }
            results.extend(self.fallback_decode(image)?);
        }
        
        // 过滤低置信度结果
        let filtered_results: Vec<QRCodeResult> = results
            .into_iter()
            .filter(|result| result.confidence >= self.config.min_confidence)
            .collect();
        
        // 更新统计信息
        self.stats.total_attempts += 1;
        if !filtered_results.is_empty() {
            self.stats.successful_decodes += 1;
            self.stats.total_qr_codes_found += filtered_results.len();
        }
        
        if self.config.verbose {
            if filtered_results.is_empty() {
                println!("❌ 未检测到二维码");
            } else {
                println!("✅ 检测到 {} 个二维码", filtered_results.len());
                for (i, result) in filtered_results.iter().enumerate() {
                    println!("   QR {} - 置信度: {:.2}, 内容长度: {} 字符, 类型: {}", 
                        i + 1, result.confidence, result.content.len(), result.qr_type);
                }
            }
        }
        
        Ok(filtered_results)
    }
    
    /// 回退解码方法（使用标准 OpenCV 解码器）
    fn fallback_decode(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>> {
        let mut results = Vec::new();
        
        // 尝试检测多个二维码
        match self.detect_and_decode_multi(image) {
            Ok(multi_results) => {
                if !multi_results.is_empty() {
                    results.extend(multi_results);
                } else {
                    // 如果多重检测失败，尝试单个检测
                    if let Ok(single_result) = self.detect_and_decode_single(image) {
                        results.push(single_result);
                    }
                }
            }
            Err(_) => {
                // 多重检测失败，尝试单个检测
                if let Ok(single_result) = self.detect_and_decode_single(image) {
                    results.push(single_result);
                }
            }
        }
        
        Ok(results)
    }
    
    /// 检测并解码单个二维码
    pub fn detect_and_decode_single(&mut self, image: &Mat) -> Result<QRCodeResult> {
        let mut points = Vector::<Point2f>::new();
        let mut straight_qrcode = Mat::default();
        
        // 检测并解码二维码
        let decoded_info = self.detector
            .detect_and_decode(image, &mut points, &mut straight_qrcode)
            .map_err(|e| QRDecodeError::decode_error(format!("二维码检测失败: {}", e)))?;
        
        if decoded_info.is_empty() {
            return Err(QRDecodeError::NoQRCodeFound);
        }
        
        let decoded_string = String::from_utf8(decoded_info)
            .map_err(|e| QRDecodeError::decode_error(format!("解码字符串转换失败: {}", e)))?;
        
        // 计算位置信息
        let position = self.calculate_position_from_points(&points)?;
        
        // 计算置信度（基于检测到的角点数量和图像质量）
        let confidence = self.calculate_confidence(&points, &straight_qrcode)?;
        
        let result = QRCodeResult::new(
            decoded_string,
            position,
            confidence,
            "QR_CODE".to_string(),
        );
        
        Ok(result)
    }
    
    /// 检测并解码多个二维码
    fn detect_and_decode_multi(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>> {
        let mut decoded_infos = Vector::<String>::new();
        let mut points = Vector::<Mat>::new();
        let mut straight_qrcodes = Vector::<Mat>::new();
        
        // 检测多个二维码
        let _success = self.detector
            .detect_and_decode_multi(image, &mut decoded_infos, &mut points, &mut straight_qrcodes)
            .map_err(|e| QRDecodeError::decode_error(format!("多重二维码检测失败: {}", e)))?;
        
        if decoded_infos.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut results = Vec::new();
        
        // 处理每个检测到的二维码
        for i in 0..decoded_infos.len() {
            let decoded_info = decoded_infos.get(i)
                .map_err(|e| QRDecodeError::decode_error(format!("获取解码信息失败: {}", e)))?;
            
            if decoded_info.is_empty() {
                continue;
            }
            
            // 获取对应的角点
            let qr_points = points.get(i)
                .map_err(|e| QRDecodeError::decode_error(format!("获取角点信息失败: {}", e)))?;
            
            // 转换角点格式
            let corner_points = self.extract_corner_points(&qr_points)?;
            
            // 计算位置信息
            let position = self.calculate_position_from_corners(&corner_points)?;
            
            // 获取对应的直线化二维码图像
            let straight_qrcode = straight_qrcodes.get(i)
                .map_err(|e| QRDecodeError::decode_error(format!("获取直线化图像失败: {}", e)))?;
            
            // 计算置信度
            let confidence = self.calculate_confidence_from_corners(&corner_points, &straight_qrcode)?;
            
            let result = QRCodeResult::new(
                decoded_info,
                position,
                confidence,
                "QR_CODE".to_string(),
            );
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 从角点计算位置信息
    fn calculate_position_from_points(&self, points: &Vector<Point2f>) -> Result<QRPosition> {
        if points.len() < 4 {
            return Err(QRDecodeError::decode_error("角点数量不足".to_string()));
        }
        
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        
        let mut corners = Vec::new();
        
        for i in 0..points.len() {
            let point = points.get(i)
                .map_err(|e| QRDecodeError::decode_error(format!("获取角点失败: {}", e)))?;
            
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
            
            corners.push((point.x, point.y));
        }
        
        let position = QRPosition::new(
            min_x as i32,
            min_y as i32,
            (max_x - min_x) as i32,
            (max_y - min_y) as i32,
        ).with_corners(corners);
        
        Ok(position)
    }
    
    /// 从角点数组计算位置信息
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
    
    /// 提取角点坐标
    fn extract_corner_points(&self, points_mat: &Mat) -> Result<Vec<(f32, f32)>> {
        let mut corners = Vec::new();
        
        // 假设角点以 Point2f 格式存储
        let rows = points_mat.rows();
        
        for i in 0..rows {
            let point: Point2f = *points_mat.at_2d(i, 0)
                .map_err(|e| QRDecodeError::decode_error(format!("提取角点失败: {}", e)))?;
            corners.push((point.x, point.y));
        }
        
        Ok(corners)
    }
    
    /// 计算置信度
    fn calculate_confidence(&self, points: &Vector<Point2f>, straight_qrcode: &Mat) -> Result<f32> {
        let mut confidence: f32 = 0.5; // 基础置信度
        
        // 基于角点数量调整置信度
        if points.len() >= 4 {
            confidence += 0.2;
        }
        
        // 基于直线化图像质量调整置信度
        if !straight_qrcode.empty() {
            let size = straight_qrcode.size()?;
            if size.width > 20 && size.height > 20 {
                confidence += 0.2;
            }
        }
        
        // 基于角点的几何特性调整置信度
        if points.len() >= 4 {
            let area = self.calculate_qr_area(points)?;
            if area > 100.0 {
                confidence += 0.1;
            }
        }
        
        Ok(confidence.min(1.0))
    }
    
    /// 从角点计算置信度
    fn calculate_confidence_from_corners(&self, corners: &[(f32, f32)], straight_qrcode: &Mat) -> Result<f32> {
        let mut confidence: f32 = 0.5; // 基础置信度
        
        // 基于角点数量调整置信度
        if corners.len() >= 4 {
            confidence += 0.2;
        }
        
        // 基于直线化图像质量调整置信度
        if !straight_qrcode.empty() {
            let size = straight_qrcode.size()?;
            if size.width > 20 && size.height > 20 {
                confidence += 0.2;
            }
        }
        
        // 基于角点的几何特性调整置信度
        if corners.len() >= 4 {
            let area = self.calculate_area_from_corners(corners);
            if area > 100.0 {
                confidence += 0.1;
            }
        }
        
        Ok(confidence.min(1.0))
    }
    
    /// 计算二维码区域面积
    fn calculate_qr_area(&self, points: &Vector<Point2f>) -> Result<f32> {
        if points.len() < 4 {
            return Ok(0.0);
        }
        
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        
        for i in 0..points.len() {
            let point = points.get(i)
                .map_err(|e| QRDecodeError::decode_error(format!("获取角点失败: {}", e)))?;
            
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        
        Ok((max_x - min_x) * (max_y - min_y))
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
    
    /// 获取解码统计信息
    pub fn get_stats(&self) -> &DecodingStats {
        &self.stats
    }
    
    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = DecodingStats::new();
    }
}

/// 解码统计信息
#[derive(Debug, Clone, Default)]
pub struct DecodingStats {
    /// 总尝试次数
    pub total_attempts: usize,
    /// 成功解码次数
    pub successful_decodes: usize,
    /// 总共找到的二维码数量
    pub total_qr_codes_found: usize,
    /// 按内容长度分组的统计
    pub content_length_stats: HashMap<String, usize>,
}

impl DecodingStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self {
            total_attempts: 0,
            successful_decodes: 0,
            total_qr_codes_found: 0,
            content_length_stats: HashMap::new(),
        }
    }
    
    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_decodes as f64 / self.total_attempts as f64
        }
    }
    
    /// 计算平均每次检测到的二维码数量
    pub fn average_qr_codes_per_attempt(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.total_qr_codes_found as f64 / self.total_attempts as f64
        }
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
    fn test_qr_decoder_creation() {
        let config = create_test_config();
        let decoder = QRDecoder::new(&config);
        assert_eq!(decoder.stats.total_attempts, 0);
    }
    
    #[test]
    fn test_decoding_stats() {
        let mut stats = DecodingStats::new();
        stats.total_attempts = 10;
        stats.successful_decodes = 8;
        stats.total_qr_codes_found = 12;
        
        assert_eq!(stats.success_rate(), 0.8);
        assert_eq!(stats.average_qr_codes_per_attempt(), 1.2);
    }
    
    #[test]
    fn test_area_calculation() {
        let corners = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        let config = create_test_config();
        let decoder = QRDecoder::new(&config);
        
        let area = decoder.calculate_area_from_corners(&corners);
        assert_eq!(area, 10000.0);
    }
}