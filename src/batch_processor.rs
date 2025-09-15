use std::path::{Path, PathBuf};
use std::fs;
use std::time::{Duration, Instant};
use crate::error::QRDecodeError;
use crate::types::{QrResult, ProcessingConfig};
use crate::brute_force_decoder::BruteForceDecoder;

/// 批量处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 目录路径
    pub directory: PathBuf,
    /// 是否递归处理子目录
    pub recursive: bool,
    /// 输出报告文件路径
    pub output_report: Option<PathBuf>,
    /// 支持的图片格式
    pub supported_formats: Vec<String>,
    /// 预期二维码数量
    pub expected_count: usize,
    /// 是否随机化参数
    pub randomize: bool,
    /// 是否显示进度
    pub show_progress: bool,
    /// 是否启用彩色输出
    pub colored_output: bool,
    /// 是否详细输出
    pub verbose: bool,
    /// 是否安静模式
    pub quiet: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("."),
            recursive: false,
            output_report: None,
            supported_formats: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "bmp".to_string(),
                "tiff".to_string(),
                "webp".to_string(),
            ],
            expected_count: 1,
            randomize: false,
            show_progress: true,
            colored_output: true,
            verbose: false,
            quiet: false,
        }
    }
}

/// 批量处理结果
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// 文件路径
    pub file_path: PathBuf,
    /// 解码结果
    pub results: Vec<QrResult>,
    /// 处理时间
    pub processing_time: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 批量处理统计
#[derive(Debug)]
pub struct BatchStats {
    /// 总文件数
    pub total_files: usize,
    /// 已处理文件数
    pub processed_files: usize,
    /// 成功解码文件数
    pub successful_files: usize,
    /// 失败文件数
    pub failed_files: usize,
    /// 总解码结果数
    pub total_qr_codes: usize,
    /// 开始时间
    pub start_time: Instant,
    /// 总处理时间
    pub total_processing_time: Duration,
}

impl Default for BatchStats {
    fn default() -> Self {
        Self {
            total_files: 0,
            processed_files: 0,
            successful_files: 0,
            failed_files: 0,
            total_qr_codes: 0,
            start_time: Instant::now(),
            total_processing_time: Duration::from_secs(0),
        }
    }
}

impl BatchStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            ..Default::default()
        }
    }

    /// 计算处理速度（文件/秒）
    pub fn processing_speed(&self) -> f64 {
        if self.processed_files == 0 {
            return 0.0;
        }
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.processed_files as f64 / elapsed
    }

    /// 计算预估剩余时间
    pub fn estimated_remaining_time(&self) -> Duration {
        if self.processed_files == 0 || self.total_files == 0 {
            return Duration::from_secs(0);
        }
        let remaining_files = self.total_files - self.processed_files;
        let speed = self.processing_speed();
        if speed == 0.0 {
            return Duration::from_secs(0);
        }
        Duration::from_secs_f64(remaining_files as f64 / speed)
    }

    /// 计算进度百分比
    pub fn progress_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.processed_files as f64 / self.total_files as f64) * 100.0
    }
}

/// 批量处理器
pub struct BatchProcessor {
    config: BatchConfig,
    decoder: BruteForceDecoder,
}

impl BatchProcessor {
    /// 创建新的批量处理器
    pub fn new(config: BatchConfig) -> Result<Self, QRDecodeError> {
        let decoder = BruteForceDecoder::new()?;
        Ok(Self { config, decoder })
    }

    /// 收集所有需要处理的图片文件
    pub fn collect_image_files(&self) -> Result<Vec<PathBuf>, QRDecodeError> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.config.directory, &mut files)?;
        Ok(files)
    }

    /// 递归收集文件
    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), QRDecodeError> {
        if !dir.exists() {
            return Err(QRDecodeError::invalid_input(format!("目录不存在: {:?}", dir)));
        }

        if !dir.is_dir() {
            return Err(QRDecodeError::invalid_input(format!("路径不是目录: {:?}", dir)));
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| QRDecodeError::decode_error(format!("读取目录失败: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| QRDecodeError::decode_error(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();

            if path.is_file() {
                if self.is_supported_image(&path) {
                    files.push(path);
                }
            } else if path.is_dir() && self.config.recursive {
                self.collect_files_recursive(&path, files)?;
            }
        }

        Ok(())
    }

    /// 检查文件是否为支持的图片格式
    fn is_supported_image(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.config.supported_formats
                    .iter()
                    .any(|fmt| fmt.eq_ignore_ascii_case(ext_str));
            }
        }
        false
    }

    /// 处理单个文件
    pub fn process_file(&mut self, file_path: &Path) -> BatchResult {
        let start_time = Instant::now();
        
        match self.decoder.decode_with_brute_force(
            file_path,
            self.config.expected_count,
            self.config.randomize,
        ) {
            Ok(results) => {
                let processing_time = start_time.elapsed();
                BatchResult {
                    file_path: file_path.to_path_buf(),
                    results: results.clone(),
                    processing_time,
                    success: !results.is_empty(),
                    error: None,
                }
            }
            Err(e) => {
                let processing_time = start_time.elapsed();
                BatchResult {
                    file_path: file_path.to_path_buf(),
                    results: Vec::new(),
                    processing_time,
                    success: false,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    /// 批量处理所有文件
    pub fn process_batch<F>(&mut self, progress_callback: F) -> Result<Vec<BatchResult>, QRDecodeError>
    where
        F: Fn(&BatchStats, &str),
    {
        let files = self.collect_image_files()?;
        let mut stats = BatchStats::new();
        stats.total_files = files.len();
        
        let mut results = Vec::new();

        for file_path in &files {
            let file_name = file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知文件");
            
            progress_callback(&stats, &format!("正在处理: {}", file_name));
            
            let result = self.process_file(file_path);
            
            // 立即显示每个文件的处理结果
            if !self.config.quiet {
                if result.success {
                    println!("\n✅ {}", file_name);
                    println!("   📁 路径: {}", result.file_path.display());
                    println!("   🎯 检测到 {} 个二维码", result.results.len());
                    println!("   ⏱️  处理时间: {:.3} 秒", result.processing_time.as_secs_f64());
                    
                    for (i, qr_result) in result.results.iter().enumerate() {
                        println!("   📄 二维码 {}: {}", i + 1, qr_result.content);
                        if self.config.verbose {
                            if let Some(points) = &qr_result.points {
                                println!("      📍 位置: {:?}", points);
                            }
                        }
                    }
                } else {
                    println!("\n❌ {}", file_name);
                    println!("   📁 路径: {}", result.file_path.display());
                    println!("   ⏱️  处理时间: {:.3} 秒", result.processing_time.as_secs_f64());
                    if let Some(error) = &result.error {
                        println!("   🚫 错误: {}", error);
                    }
                }
            }
            
            // 更新统计信息
            stats.processed_files += 1;
            stats.total_processing_time += result.processing_time;
            
            if result.success {
                stats.successful_files += 1;
                stats.total_qr_codes += result.results.len();
            } else {
                stats.failed_files += 1;
            }
            
            results.push(result);
        }

        Ok(results)
    }

    /// 生成批量处理报告
    pub fn generate_report(&self, results: &[BatchResult], stats: &BatchStats) -> String {
        let mut report = String::new();
        
        report.push_str("=== 批量二维码解码报告 ===\n\n");
        
        // 统计信息
        report.push_str(&format!("处理目录: {:?}\n", self.config.directory));
        report.push_str(&format!("递归处理: {}\n", if self.config.recursive { "是" } else { "否" }));
        report.push_str(&format!("总文件数: {}\n", stats.total_files));
        report.push_str(&format!("成功解码: {}\n", stats.successful_files));
        report.push_str(&format!("解码失败: {}\n", stats.failed_files));
        report.push_str(&format!("总二维码数: {}\n", stats.total_qr_codes));
        report.push_str(&format!("处理速度: {:.2} 文件/秒\n", stats.processing_speed()));
        report.push_str(&format!("总耗时: {:.2} 秒\n\n", stats.start_time.elapsed().as_secs_f64()));
        
        // 详细结果
        report.push_str("=== 详细结果 ===\n\n");
        
        for result in results {
            let file_name = result.file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知文件");
            
            report.push_str(&format!("文件: {}\n", file_name));
            report.push_str(&format!("路径: {:?}\n", result.file_path));
            report.push_str(&format!("状态: {}\n", if result.success { "成功" } else { "失败" }));
            report.push_str(&format!("耗时: {:.3} 秒\n", result.processing_time.as_secs_f64()));
            
            if result.success {
                report.push_str(&format!("解码数量: {}\n", result.results.len()));
                for (i, qr_result) in result.results.iter().enumerate() {
                    report.push_str(&format!("  二维码 {}: {}\n", i + 1, qr_result.content));
                    if let Some(points) = &qr_result.points {
                        report.push_str(&format!("  坐标: {:?}\n", points));
                    }
                }
            } else if let Some(error) = &result.error {
                report.push_str(&format!("错误: {}\n", error));
            }
            
            report.push_str("\n");
        }
        
        report
    }

    /// 保存报告到文件
    pub fn save_report(&self, report: &str) -> Result<(), QRDecodeError> {
        if let Some(report_path) = &self.config.output_report {
            fs::write(report_path, report)
                .map_err(|e| QRDecodeError::output_error(format!("保存报告失败: {}", e)))?;
        }
        Ok(())
    }
}