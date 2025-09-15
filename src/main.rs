//! QR Code Decoder - 基于 OpenCV-Rust 的二维码解码器
//! 
//! 这是一个命令行工具，用于从图像文件中检测和解码二维码。

use std::env;
use std::process;

mod cli;
mod error;
mod image_processor;
mod enhanced_processor;
mod brute_force_decoder;
mod output;
mod qr_decoder;
mod wechat_qr_decoder;
mod types;
mod batch_processor;
mod progress_display;

use cli::Args;
use error::{QRDecodeError, Result};
use image_processor::ImageProcessor;
use enhanced_processor::EnhancedImageProcessor;
use brute_force_decoder::BruteForceDecoder;
use output::OutputFormatter;
use qr_decoder::QRDecoder;
use types::ProcessingConfig;
use batch_processor::{BatchProcessor, BatchConfig};
use progress_display::ProgressDisplay;

fn main() {
    // 设置 panic hook 以提供更好的错误信息
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("❌ 程序发生严重错误: {}", panic_info);
        eprintln!("请检查输入参数和文件路径是否正确");
    }));
    
    // 解析命令行参数
    let args = match Args::parse_from_env() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("❌ 参数解析错误: {}", err);
            eprintln!("\n使用 --help 查看帮助信息");
            process::exit(1);
        }
    };
    
    // 处理帮助和版本请求
    if args.show_help {
        Args::print_help();
        process::exit(0);
    }
    
    if args.show_version {
        Args::print_version();
        process::exit(0);
    }
    
    // 验证参数
    if let Err(err) = args.validate() {
        eprintln!("❌ 参数验证失败: {}", err);
        process::exit(1);
    }
    
    // 检查是否为批量处理模式
    if args.is_batch_mode() {
        // 批量处理模式
        match process_batch(&args) {
            Ok(()) => {
                if !args.quiet {
                    eprintln!("✅ 批量处理完成");
                }
            }
            Err(err) => {
                eprintln!("❌ 批量处理失败: {}", err);
                process::exit(1);
            }
        }
    } else {
        // 单文件处理模式
        let config = match ProcessingConfig::from_args(&args) {
            Ok(config) => config,
            Err(err) => {
                eprintln!("❌ 配置错误: {}", err);
                process::exit(1);
            }
        };
        
        // 执行处理
        match process_image(&config) {
            Ok(()) => {
                if config.verbose {
                    eprintln!("✅ 处理完成");
                }
            }
            Err(err) => {
                let formatter = OutputFormatter::new(&config);
                formatter.output_error(&err);
                
                // 根据错误类型设置不同的退出码
                let exit_code = match &err {
                    QRDecodeError::IoError(_) => 2,
                    QRDecodeError::OpenCVError(_) => 3,
                    QRDecodeError::InvalidInput(_) => 4,
                    QRDecodeError::UnsupportedFormat(_) => 5,
                    QRDecodeError::ImageProcessingError(_) => 6,
                    QRDecodeError::OutputError(_) => 7,
                    _ => 1,
                };
                
                process::exit(exit_code);
            }
        }
    }
}

fn process_image(config: &ProcessingConfig) -> Result<()> {
    // 创建输出格式化器
    let formatter = OutputFormatter::new(config);
    
    formatter.output_progress("🚀 开始处理图像...");
    
    // 验证输入文件存在
    if !config.input_path.exists() {
        return Err(QRDecodeError::invalid_input(format!(
            "输入文件不存在: {}",
            config.input_path.display()
        )));
    }
    
    // 创建输出目录（如果需要）
    if let Some(output_path) = &config.output_path {
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| QRDecodeError::invalid_input(format!(
                        "无法创建输出目录 {}: {}",
                        parent.display(), e
                    )))?;
            }
        }
    }
    
    // 加载和预处理图像
    let processor = ImageProcessor::new(config);
    let image = processor.load_image(&config.input_path)?;
    
    formatter.output_progress("📷 图像加载完成");
    
    let processed_image = if config.preprocess {
        formatter.output_progress("🔧 开始图像预处理...");
        let processed = processor.preprocess_image(&image)?;
        formatter.output_progress("✨ 图像预处理完成");
        processed
    } else {
        image
    };
    
    // 保存预处理后的图像（如果需要）
    if config.save_processed {
        if let Some(output_path) = &config.processed_output_path {
            processor.save_image(&processed_image, output_path)?;
            formatter.output_progress(&format!("💾 预处理图像已保存到: {}", output_path.display()));
        }
    }
    
    formatter.output_progress("🔍 开始增强二维码检测和解码...");
    
    // 使用增强图像处理器进行解码
     let mut enhanced_processor = EnhancedImageProcessor::new(config.clone())?;
    let filtered_results = enhanced_processor.decode_with_transforms(&processed_image)?;
    
    // 如果增强解码没有找到结果且启用了暴力破解，尝试暴力破解解码
    let final_results = if filtered_results.is_empty() && config.brute_force {
        formatter.output_progress("🔨 开始暴力破解解码...");
        let mut brute_force_decoder = BruteForceDecoder::new()?;
        let brute_results = brute_force_decoder.detect_and_decode(&processed_image)?;
        formatter.output_progress(&format!(
            "💪 暴力破解解码完成，找到 {} 个二维码",
            brute_results.len()
        ));
        brute_results
    } else {
        filtered_results
    };
    
    // 打印变换统计信息
    if config.verbose {
        enhanced_processor.print_transform_stats();
    }
    
    formatter.output_progress(&format!(
        "🎯 解码完成，找到 {} 个二维码（置信度 >= {:.2}）",
        final_results.len(),
        config.min_confidence
    ));
    
    // 输出结果
     formatter.output_results(&final_results)?;
     formatter.output_summary(&final_results)?;
    
    // 如果没有找到二维码，返回特定错误
    if final_results.is_empty() {
        return Err(QRDecodeError::invalid_input("未找到二维码".to_string()));
    }
    
    Ok(())
}

fn process_batch(args: &Args) -> Result<()> {
    // 获取批量处理目录
    let directory = args.get_batch_directory()
        .ok_or_else(|| QRDecodeError::invalid_input("批量处理模式需要指定目录路径".to_string()))?;
    
    // 创建批量处理配置
    let batch_config = BatchConfig {
        directory: directory.clone(),
        recursive: args.is_recursive(),
        output_report: args.get_report_output().map(|p| p.clone()),
        supported_formats: Args::supported_formats().iter().map(|s| s.to_string()).collect(),
        expected_count: 1,
        randomize: false,
        show_progress: args.should_show_progress(),
        colored_output: args.is_colored_output(),
        verbose: args.verbose,
        quiet: args.quiet,
    };
    
    // 创建批量处理器
    let mut batch_processor = BatchProcessor::new(batch_config);
    
    // 执行批量处理
    let mut batch_processor = batch_processor?;
    let batch_result = batch_processor.process_batch(|stats, current_file| {
        // 显示进度信息
        if !args.quiet {
            let progress = stats.progress_percentage();
            let speed = stats.processing_speed();
            let remaining = stats.estimated_remaining_time();
            
            println!(
                "🔍 [{}/{}] ({:.1}%) {} - 速度: {:.1} 文件/秒 - 预计剩余: {:.0}秒",
                stats.processed_files,
                stats.total_files,
                progress,
                current_file,
                speed,
                remaining.as_secs_f64()
            );
        }
    })?;
    
    // 结果已在批量处理过程中实时显示
    
    // 创建统计信息
     let mut stats = crate::batch_processor::BatchStats::new();
     stats.total_files = batch_result.len();
     stats.processed_files = batch_result.len();
     stats.successful_files = batch_result.iter().filter(|r| r.success).count();
     stats.failed_files = batch_result.len() - stats.successful_files;
     stats.total_qr_codes = batch_result.iter().map(|r| r.results.len()).sum();
     stats.total_processing_time = batch_result.iter().map(|r| r.processing_time).sum();
    
    // 输出批量处理结果
    if !args.quiet {
        println!("\n✅ 批量处理完成!");
        println!("📊 处理统计:");
        println!("   - 总文件数: {}", stats.total_files);
        println!("   - 成功解码: {}", stats.successful_files);
        println!("   - 解码失败: {}", stats.failed_files);
        println!("   - 总二维码数: {}", stats.total_qr_codes);
        println!("   - 处理速度: {:.2} 文件/秒", stats.processing_speed());
        println!("   - 总耗时: {:.2} 秒", stats.total_processing_time.as_secs_f64());
        
        if stats.failed_files > 0 && args.verbose {
            println!("\n❌ 失败的文件:");
            for result in &batch_result {
                if !result.success {
                    println!("   {}: {}", result.file_path.display(), result.error.as_ref().unwrap_or(&"未知错误".to_string()));
                }
            }
        }
    }
    
    // 生成报告（如果指定了输出路径）
    if let Some(report_path) = args.get_report_output() {
        let report = batch_processor.generate_report(&batch_result, &stats);
        batch_processor.save_report(&report)?;
        if !args.quiet {
            println!("📄 批量处理报告已保存到: {}", report_path.display());
        }
    }
    
    Ok(())
}

/// 显示版本信息
fn show_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("基于 opencv-rust 的二维码解码器");
    println!("作者: {}", env!("CARGO_PKG_AUTHORS"));
}

/// 显示帮助信息
fn show_help() {
    println!("{}\n", env!("CARGO_PKG_DESCRIPTION"));
    println!("用法:");
    println!("  {} [选项] <输入文件>\n", env!("CARGO_PKG_NAME"));
    println!("选项:");
    println!("  -o, --output <文件>        输出文件路径");
    println!("  -f, --format <格式>        输出格式 [text|json|csv|verbose]");
    println!("  -p, --preprocess           启用图像预处理");
    println!("  -v, --verbose              详细输出");
    println!("  -q, --quiet                静默模式");
    println!("  --show-position            显示二维码位置信息");
    println!("  --min-confidence <值>      最小置信度阈值 (0.0-1.0)");
    println!("  --save-processed <文件>    保存预处理后的图像");
    println!("  -h, --help                 显示此帮助信息");
    println!("  -V, --version              显示版本信息\n");
    println!("示例:");
    println!("  {} image.jpg", env!("CARGO_PKG_NAME"));
    println!("  {} -f json -o result.json image.png", env!("CARGO_PKG_NAME"));
    println!("  {} --preprocess --verbose image.jpg", env!("CARGO_PKG_NAME"));
}