//! 结果输出和格式化模块
//! 
//! 负责将二维码解码结果格式化并输出到不同的目标。

use chrono::{DateTime, Utc};
use serde_json;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use crate::error::{QRDecodeError, Result};
use crate::types::{OutputFormat, ProcessingConfig, QRCodeResult};

/// 输出格式化器
pub struct OutputFormatter {
    /// 处理配置
    config: ProcessingConfig,
}

impl OutputFormatter {
    /// 创建新的输出格式化器
    pub fn new(config: &ProcessingConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
    
    /// 输出解码结果
    pub fn output_results(&self, results: &[QRCodeResult]) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }
        
        let formatted_output = match self.config.output_format {
            OutputFormat::Text => self.format_as_text(results)?,
            OutputFormat::Json => self.format_as_json(results)?,
            OutputFormat::Csv => self.format_as_csv(results)?,
            OutputFormat::Verbose => self.format_as_verbose(results)?,
        };
        
        // 输出到文件或标准输出
        if let Some(output_path) = &self.config.output_path {
            self.write_to_file(&formatted_output, output_path)?;
        } else {
            self.write_to_stdout(&formatted_output)?;
        }
        
        Ok(())
    }
    
    /// 格式化为纯文本
    fn format_as_text(&self, results: &[QRCodeResult]) -> Result<String> {
        let mut output = String::new();
        
        for (i, result) in results.iter().enumerate() {
            if results.len() > 1 {
                output.push_str(&format!("=== 二维码 {} ===\n", i + 1));
            }
            
            output.push_str(&result.content);
            
            if self.config.show_position {
                output.push_str(&format!(
                    " [位置: ({}, {}), 大小: {}x{}]",
                    result.position.x,
                    result.position.y,
                    result.position.width,
                    result.position.height
                ));
            }
            
            if results.len() > 1 {
                output.push('\n');
            }
        }
        
        Ok(output)
    }
    
    /// 格式化为 JSON
    fn format_as_json(&self, results: &[QRCodeResult]) -> Result<String> {
        let output_data = if results.len() == 1 {
            // 单个结果直接输出对象
            serde_json::to_string_pretty(&results[0])?
        } else {
            // 多个结果输出数组
            serde_json::to_string_pretty(results)?
        };
        
        Ok(output_data)
    }
    
    /// 格式化为 CSV
    fn format_as_csv(&self, results: &[QRCodeResult]) -> Result<String> {
        let mut output = String::new();
        
        // CSV 头部
        if self.config.show_position {
            output.push_str("content,confidence,type,timestamp,x,y,width,height\n");
        } else {
            output.push_str("content,confidence,type,timestamp\n");
        }
        
        // CSV 数据行
        for result in results {
            let escaped_content = self.escape_csv_field(&result.content);
            let timestamp = result.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
            
            if self.config.show_position {
                output.push_str(&format!(
                    "{},{:.3},{},\"{}\",{},{},{},{}\n",
                    escaped_content,
                    result.confidence,
                    result.qr_type,
                    timestamp,
                    result.position.x,
                    result.position.y,
                    result.position.width,
                    result.position.height
                ));
            } else {
                output.push_str(&format!(
                    "{},{:.3},{},\"{}\"\n",
                    escaped_content,
                    result.confidence,
                    result.qr_type,
                    timestamp
                ));
            }
        }
        
        Ok(output)
    }
    
    /// 格式化为详细格式
    fn format_as_verbose(&self, results: &[QRCodeResult]) -> Result<String> {
        let mut output = String::new();
        
        output.push_str(&format!("二维码解码结果报告\n"));
        output.push_str(&format!("生成时间: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("检测到的二维码数量: {}\n\n", results.len()));
        
        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!("┌─ 二维码 #{} ─────────────────────────────────────┐\n", i + 1));
            output.push_str(&format!("│ 类型: {}\n", result.qr_type));
            output.push_str(&format!("│ 置信度: {:.3}\n", result.confidence));
            output.push_str(&format!("│ 解码时间: {}\n", result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
            
            // 位置信息
            output.push_str(&format!("│ 位置: ({}, {})\n", result.position.x, result.position.y));
            output.push_str(&format!("│ 大小: {} x {} 像素\n", result.position.width, result.position.height));
            output.push_str(&format!("│ 面积: {} 平方像素\n", result.position.area()));
            
            let (center_x, center_y) = result.position.center();
            output.push_str(&format!("│ 中心点: ({:.1}, {:.1})\n", center_x, center_y));
            
            // 角点信息
            if let Some(corners) = &result.position.corners {
                output.push_str(&format!("│ 角点数量: {}\n", corners.len()));
                for (j, (x, y)) in corners.iter().enumerate() {
                    output.push_str(&format!("│   角点 {}: ({:.1}, {:.1})\n", j + 1, x, y));
                }
            }
            
            // 内容信息
            output.push_str(&format!("│ 内容长度: {} 字符\n", result.content.len()));
            
            if let Some(raw_bytes) = &result.raw_bytes {
                output.push_str(&format!("│ 原始字节长度: {} 字节\n", raw_bytes.len()));
            }
            
            // 内容预览
            let content_preview = if result.content.len() > 100 {
                format!("{}...", &result.content[..97])
            } else {
                result.content.clone()
            };
            
            output.push_str(&format!("│ 内容预览:\n"));
            for line in content_preview.lines() {
                output.push_str(&format!("│   {}\n", line));
            }
            
            output.push_str(&format!("└─────────────────────────────────────────────────┘\n"));
            
            if i < results.len() - 1 {
                output.push('\n');
            }
        }
        
        // 统计信息
        output.push_str(&format!("\n📊 统计信息:\n"));
        output.push_str(&format!("   • 总二维码数量: {}\n", results.len()));
        
        let avg_confidence: f32 = results.iter().map(|r| r.confidence).sum::<f32>() / results.len() as f32;
        output.push_str(&format!("   • 平均置信度: {:.3}\n", avg_confidence));
        
        let total_content_length: usize = results.iter().map(|r| r.content.len()).sum();
        output.push_str(&format!("   • 总内容长度: {} 字符\n", total_content_length));
        
        let total_area: i32 = results.iter().map(|r| r.position.area()).sum();
        output.push_str(&format!("   • 总覆盖面积: {} 平方像素\n", total_area));
        
        Ok(output)
    }
    
    /// 转义 CSV 字段
    fn escape_csv_field(&self, field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
    
    /// 写入文件
    fn write_to_file(&self, content: &str, path: &Path) -> Result<()> {
        let mut file = File::create(path)
            .map_err(|e| QRDecodeError::output_error(format!(
                "无法创建输出文件 {}: {}",
                path.display(), e
            )))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| QRDecodeError::output_error(format!(
                "写入文件失败 {}: {}",
                path.display(), e
            )))?;
        
        if self.config.verbose {
            println!("💾 结果已保存到: {}", path.display());
        }
        
        Ok(())
    }
    
    /// 写入标准输出
    fn write_to_stdout(&self, content: &str) -> Result<()> {
        print!("{}", content);
        io::stdout().flush()
            .map_err(|e| QRDecodeError::output_error(format!("标准输出刷新失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 输出摘要信息
    pub fn output_summary(&self, results: &[QRCodeResult]) -> Result<()> {
        
        if results.is_empty() {
            eprintln!("❌ 未检测到二维码");
        } else {
            eprintln!("✅ 成功检测到 {} 个二维码", results.len());
            
            if self.config.verbose {
                let avg_confidence: f32 = results.iter().map(|r| r.confidence).sum::<f32>() / results.len() as f32;
                eprintln!("   平均置信度: {:.3}", avg_confidence);
                
                let total_chars: usize = results.iter().map(|r| r.content.len()).sum();
                eprintln!("   总内容长度: {} 字符", total_chars);
            }
        }
        
        Ok(())
    }
    
    /// 输出错误信息
    pub fn output_error(&self, error: &QRDecodeError) {
        eprintln!("❌ 错误: {}", error);
    }
    
    /// 输出处理进度
    pub fn output_progress(&self, message: &str) {
        if self.config.verbose {
            eprintln!("🔄 {}", message);
        }
    }
}

/// 输出统计信息
#[derive(Debug, Clone)]
pub struct OutputStats {
    /// 输出的二维码数量
    pub qr_codes_output: usize,
    /// 输出的总字符数
    pub total_characters: usize,
    /// 输出文件大小（字节）
    pub output_size_bytes: usize,
    /// 输出格式
    pub format_used: OutputFormat,
    /// 输出时间
    pub output_time: DateTime<Utc>,
}

impl OutputStats {
    /// 创建新的输出统计
    pub fn new(results: &[QRCodeResult], format: OutputFormat, output_size: usize) -> Self {
        let total_characters = results.iter().map(|r| r.content.len()).sum();
        
        Self {
            qr_codes_output: results.len(),
            total_characters,
            output_size_bytes: output_size,
            format_used: format,
            output_time: Utc::now(),
        }
    }
    
    /// 计算压缩比（输出大小 vs 原始内容大小）
    pub fn compression_ratio(&self) -> f64 {
        if self.total_characters == 0 {
            0.0
        } else {
            self.output_size_bytes as f64 / self.total_characters as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ProcessingConfig, QRPosition};
    use chrono::Utc;
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
    
    fn create_test_result() -> QRCodeResult {
        let position = QRPosition::new(10, 20, 100, 100);
        QRCodeResult::new("Test QR Code", position, 0.95, "QR_CODE")
    }
    
    #[test]
    fn test_text_formatting() {
        let config = create_test_config();
        let formatter = OutputFormatter::new(&config);
        let results = vec![create_test_result()];
        
        let output = formatter.format_as_text(&results).unwrap();
        assert!(output.contains("Test QR Code"));
    }
    
    #[test]
    fn test_json_formatting() {
        let config = create_test_config();
        let formatter = OutputFormatter::new(&config);
        let results = vec![create_test_result()];
        
        let output = formatter.format_as_json(&results).unwrap();
        assert!(output.contains("Test QR Code"));
        assert!(output.contains("confidence"));
    }
    
    #[test]
    fn test_csv_field_escaping() {
        let config = create_test_config();
        let formatter = OutputFormatter::new(&config);
        
        let escaped = formatter.escape_csv_field("Hello, World!");
        assert_eq!(escaped, "\"Hello, World!\"");
        
        let normal = formatter.escape_csv_field("Hello World");
        assert_eq!(normal, "Hello World");
    }
    
    #[test]
    fn test_output_stats() {
        let results = vec![create_test_result()];
        let stats = OutputStats::new(&results, OutputFormat::Json, 100);
        
        assert_eq!(stats.qr_codes_output, 1);
        assert_eq!(stats.total_characters, 12); // "Test QR Code" length
        assert_eq!(stats.output_size_bytes, 100);
    }
}