use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;
use std::env;

use crate::error::{QRDecodeError, Result};
use crate::types::OutputFormat;

/// 命令行参数结构
#[derive(Debug, Clone)]
pub struct Args {
    /// 输入图像文件路径或目录路径（批量模式）
    pub input_path: PathBuf,
    /// 输出文件路径（可选）
    pub output_path: Option<PathBuf>,
    /// 输出格式
    pub output_format: OutputFormat,
    /// 是否启用图像预处理
    pub preprocess: bool,
    /// 是否显示详细信息
    pub verbose: bool,
    /// 是否静默模式
    pub quiet: bool,
    /// 是否显示位置信息
    pub show_position: bool,
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 是否保存预处理后的图像
    pub save_processed: bool,
    /// 预处理图像输出路径
    pub processed_output_path: Option<PathBuf>,
    /// 是否显示帮助信息
    pub show_help: bool,
    /// 是否显示版本信息
    pub show_version: bool,
    /// 是否启用暴力破解模式
    pub brute_force: bool,
    /// 预期的二维码数量
    pub expected_count: usize,
    /// 是否随机化参数
    pub randomize: bool,
    /// 是否启用反色处理
    pub invert: bool,
    /// 是否启用批量处理模式
    pub batch_mode: bool,
    /// 批量处理目录路径
    pub batch_directory: Option<PathBuf>,
    /// 是否递归处理子目录
    pub recursive: bool,
    /// 批量处理报告输出路径
    pub report_output: Option<PathBuf>,
    /// 是否显示进度条
    pub show_progress: bool,
    /// 是否启用彩色输出
    pub colored_output: bool,
}

impl Args {
    /// 从环境参数解析
    pub fn parse_from_env() -> Result<Self> {
        let args: Vec<String> = env::args().collect();
        
        // 处理帮助和版本参数
        if args.len() > 1 {
            match args[1].as_str() {
                "-h" | "--help" => {
                    return Ok(Args::help_args());
                }
                "-V" | "--version" => {
                    return Ok(Args::version_args());
                }
                _ => {}
            }
        }
        
        let matches = Self::create_command()
            .try_get_matches_from(&args)
            .map_err(|e| QRDecodeError::invalid_input(format!("参数解析错误: {}", e)))?;
        
        Self::from_matches(&matches)
    }
    
    /// 创建帮助参数
    fn help_args() -> Self {
        Args {
            input_path: PathBuf::new(),
            output_path: None,
            output_format: OutputFormat::Text,
            preprocess: false,
            verbose: false,
            quiet: false,
            show_position: false,
            min_confidence: 0.5,
            save_processed: false,
            processed_output_path: None,
            show_help: true,
            show_version: false,
            brute_force: false,
            expected_count: 1,
            randomize: false,
            invert: false,
            batch_mode: false,
            batch_directory: None,
            recursive: false,
            report_output: None,
            show_progress: true,
            colored_output: true,
        }
    }
    
    /// 创建版本参数
    fn version_args() -> Self {
        Args {
            input_path: PathBuf::new(),
            output_path: None,
            output_format: OutputFormat::Text,
            preprocess: false,
            verbose: false,
            quiet: false,
            show_position: false,
            min_confidence: 0.5,
            save_processed: false,
            processed_output_path: None,
            show_help: false,
            show_version: true,
            brute_force: false,
            expected_count: 1,
            randomize: false,
            invert: false,
            batch_mode: false,
            batch_directory: None,
            recursive: false,
            report_output: None,
            show_progress: true,
            colored_output: true,
        }
    }
    
    /// 创建 clap 命令
    fn create_command() -> Command {
        Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .disable_help_flag(true)  // 禁用默认的帮助标志，我们自己处理
            .disable_version_flag(true)  // 禁用默认的版本标志，我们自己处理
            .arg(
                Arg::new("input")
                    .help("输入图像文件路径")
                    .required_unless_present("batch")
                    .index(1)
                    .value_parser(clap::value_parser!(PathBuf))
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("输出文件路径")
                    .value_parser(clap::value_parser!(PathBuf))
            )
            .arg(
                Arg::new("format")
                    .short('f')
                    .long("format")
                    .help("输出格式 [text|json|csv|verbose]")
                    .value_parser(["text", "json", "csv", "verbose"])
                    .default_value("text")
            )
            .arg(
                Arg::new("preprocess")
                    .short('p')
                    .long("preprocess")
                    .help("启用图像预处理")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("显示详细信息")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("quiet")
                    .short('q')
                    .long("quiet")
                    .help("静默模式")
                    .action(clap::ArgAction::SetTrue)
                    .conflicts_with("verbose")
            )
            .arg(
                Arg::new("show-position")
                    .long("show-position")
                    .help("显示二维码位置信息")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("min-confidence")
                    .long("min-confidence")
                    .help("最小置信度阈值 (0.0-1.0)")
                    .value_parser(clap::value_parser!(f32))
                    .default_value("0.5")
            )
            .arg(
                Arg::new("save-processed")
                    .long("save-processed")
                    .help("保存预处理后的图像到指定路径")
                    .value_parser(clap::value_parser!(PathBuf))
            )
            .arg(
                Arg::new("brute-force")
                    .short('b')
                    .long("brute-force")
                    .help("启用暴力破解模式")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("expected-count")
                    .short('e')
                    .long("expected-count")
                    .help("预期的二维码数量")
                    .value_parser(clap::value_parser!(usize))
                    .default_value("1")
            )
            .arg(
                Arg::new("randomize")
                    .short('r')
                    .long("randomize")
                    .help("随机化参数组合")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("invert")
                    .short('i')
                    .long("invert")
                    .help("启用反色处理")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("batch")
                    .long("batch")
                    .help("启用批量处理模式")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("directory")
                    .short('d')
                    .long("directory")
                    .help("批量处理目录路径")
                    .value_parser(clap::value_parser!(PathBuf))
                    .requires("batch")
            )
            .arg(
                Arg::new("recursive")
                    .long("recursive")
                    .help("递归处理子目录")
                    .action(clap::ArgAction::SetTrue)
                    .requires("batch")
            )
            .arg(
                Arg::new("report-output")
                    .long("report-output")
                    .help("批量处理报告输出文件路径")
                    .value_parser(clap::value_parser!(PathBuf))
            )
            .arg(
                Arg::new("no-progress")
                    .long("no-progress")
                    .help("禁用进度显示")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("no-color")
                    .long("no-color")
                    .help("禁用彩色输出")
                    .action(clap::ArgAction::SetTrue)
            )
    }
    
    /// 从 ArgMatches 创建 Args
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // 批量处理参数
        let batch_mode = matches.get_flag("batch");
        
        let input_path = if batch_mode {
            // 批量模式下，input可以为空，使用默认路径
            matches.get_one::<PathBuf>("input")
                .cloned()
                .unwrap_or_else(|| PathBuf::from("."))
        } else {
            matches.get_one::<PathBuf>("input")
                .ok_or_else(|| QRDecodeError::invalid_input("缺少输入文件路径".to_string()))?
                .clone()
        };
        
        let output_path = matches.get_one::<PathBuf>("output").cloned();
        
        let output_format = match matches.get_one::<String>("format").unwrap().as_str() {
            "text" => OutputFormat::Text,
            "json" => OutputFormat::Json,
            "csv" => OutputFormat::Csv,
            "verbose" => OutputFormat::Verbose,
            _ => return Err(QRDecodeError::invalid_input("无效的输出格式".to_string())),
        };
        
        let preprocess = matches.get_flag("preprocess");
        let verbose = matches.get_flag("verbose");
        let quiet = matches.get_flag("quiet");
        let show_position = matches.get_flag("show-position");
        
        let min_confidence = *matches.get_one::<f32>("min-confidence").unwrap();
        
        let (save_processed, processed_output_path) = if let Some(path) = matches.get_one::<PathBuf>("save-processed") {
            (true, Some(path.clone()))
        } else {
            (false, None)
        };
        
        let brute_force = matches.get_flag("brute-force");
        let expected_count = *matches.get_one::<usize>("expected-count").unwrap();
        let randomize = matches.get_flag("randomize");
        let invert = matches.get_flag("invert");
        
        // batch_mode已在前面定义
        let batch_directory = matches.get_one::<PathBuf>("directory").cloned();
        let recursive = matches.get_flag("recursive");
        let report_output = matches.get_one::<PathBuf>("report-output").cloned();
        let show_progress = !matches.get_flag("no-progress");
        let colored_output = !matches.get_flag("no-color");
        
        Ok(Args {
            input_path,
            output_path,
            output_format,
            preprocess,
            verbose,
            quiet,
            show_position,
            min_confidence,
            save_processed,
            processed_output_path,
            show_help: false,
            show_version: false,
            brute_force,
            expected_count,
            randomize,
            invert,
            batch_mode,
            batch_directory,
            recursive,
            report_output,
            show_progress,
            colored_output,
        })
    }
    
    /// 验证参数
    pub fn validate(&self) -> Result<()> {
        // 如果是帮助或版本请求，跳过验证
        if self.show_help || self.show_version {
            return Ok(());
        }
        
        // 批量处理模式验证
        if self.batch_mode {
            // 验证批量处理目录
            let directory = self.batch_directory.as_ref().unwrap_or(&self.input_path);
            if !directory.exists() {
                return Err(QRDecodeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("批量处理目录不存在: {}", directory.display())
                )));
            }
            
            if !directory.is_dir() {
                return Err(QRDecodeError::InvalidInput(
                    format!("批量处理路径必须是目录: {}", directory.display())
                ));
            }
        } else {
            // 单文件模式验证
            if !self.input_path.exists() {
                return Err(QRDecodeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("输入文件不存在: {}", self.input_path.display())
                )));
            }
            
            // 验证输入文件格式
            if !Self::is_supported_format(&self.input_path) {
                return Err(QRDecodeError::UnsupportedFormat(format!(
                    "不支持的文件格式: {}\n支持的格式: jpg, jpeg, png, bmp, tiff, tif, webp",
                    self.input_path.display()
                )));
            }
        }
        
        // 验证置信度范围
        if !(0.0..=1.0).contains(&self.min_confidence) {
            return Err(QRDecodeError::InvalidInput(
                "置信度阈值必须在 0.0 到 1.0 之间".to_string()
            ));
        }
        
        // 验证输出目录可写
        if let Some(output_path) = &self.output_path {
            if let Some(parent) = output_path.parent() {
                if parent.exists() && !Self::is_directory_writable(parent) {
                    return Err(QRDecodeError::IoError(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("输出目录不可写: {}", parent.display())
                    )));
                }
            }
        }
        
        // 验证预处理输出路径
        if let Some(processed_path) = &self.processed_output_path {
            if let Some(parent) = processed_path.parent() {
                if parent.exists() && !Self::is_directory_writable(parent) {
                    return Err(QRDecodeError::IoError(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("预处理图像输出目录不可写: {}", parent.display())
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    /// 检查是否为支持的图像格式
    pub fn is_supported_format(path: &PathBuf) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "bmp" | "tiff" | "tif" | "webp")
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// 获取支持的格式列表
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["jpg", "jpeg", "png", "bmp", "tiff", "tif", "webp"]
    }
    
    /// 获取批量处理目录路径
    pub fn get_batch_directory(&self) -> Option<&PathBuf> {
        if self.batch_mode {
            self.batch_directory.as_ref().or(Some(&self.input_path))
        } else {
            None
        }
    }
    
    /// 是否为批量处理模式
    pub fn is_batch_mode(&self) -> bool {
        self.batch_mode
    }
    
    /// 是否启用递归处理
    pub fn is_recursive(&self) -> bool {
        self.recursive && self.batch_mode
    }
    
    /// 获取报告输出路径
    pub fn get_report_output(&self) -> Option<&PathBuf> {
        self.report_output.as_ref()
    }
    
    /// 是否显示进度
    pub fn should_show_progress(&self) -> bool {
        self.show_progress && !self.quiet
    }
    
    /// 是否启用彩色输出
    pub fn is_colored_output(&self) -> bool {
        self.colored_output && !self.quiet
    }
    
    /// 检查目录是否可写
    fn is_directory_writable(path: &std::path::Path) -> bool {
        // 尝试在目录中创建临时文件来测试写权限
        let test_file = path.join(".qr_decoder_write_test_temp");
        match std::fs::File::create(&test_file) {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                true
            }
            Err(_) => false,
        }
    }
    
    /// 显示帮助信息
    pub fn print_help() {
        println!("{}", env!("CARGO_PKG_DESCRIPTION"));
        println!();
        println!("用法:");
        println!("  {} [选项] <输入文件>", env!("CARGO_PKG_NAME"));
        println!("  {} --batch [选项] [目录]", env!("CARGO_PKG_NAME"));
        println!();
        println!("基本选项:");
        println!("  -o, --output <文件>        输出文件路径");
        println!("  -f, --format <格式>        输出格式 [text|json|csv|verbose]");
        println!("  -p, --preprocess           启用图像预处理");
        println!("  -v, --verbose              详细输出");
        println!("  -q, --quiet                静默模式");
        println!("  --show-position            显示二维码位置信息");
        println!("  --min-confidence <值>      最小置信度阈值 (0.0-1.0)");
        println!("  --save-processed <文件>    保存预处理后的图像");
        println!("  -h, --help                 显示此帮助信息");
        println!("  -V, --version              显示版本信息");
        println!();
        println!("暴力破解选项:");
        println!("  -b, --brute-force          启用暴力破解模式");
        println!("  -e, --expected-count <数>  预期的二维码数量");
        println!("  -r, --randomize            随机化参数组合");
        println!("  -i, --invert               启用反色处理");
        println!();
        println!("批量处理选项:");
        println!("  --batch                    启用批量处理模式");
        println!("  -d, --directory <目录>     批量处理目录路径");
        println!("  --recursive                递归处理子目录");
        println!("  --report-output <文件>     批量处理报告输出文件路径");
        println!("  --no-progress              禁用进度显示");
        println!("  --no-color                 禁用彩色输出");
        println!();
        println!("支持的图像格式:");
        println!("  {}", Self::supported_formats().join(", "));
        println!();
        println!("示例:");
        println!("  {} image.jpg", env!("CARGO_PKG_NAME"));
        println!("  {} -f json -o result.json image.png", env!("CARGO_PKG_NAME"));
        println!("  {} --preprocess --verbose image.jpg", env!("CARGO_PKG_NAME"));
        println!("  {} --min-confidence 0.8 --show-position image.png", env!("CARGO_PKG_NAME"));
        println!("  {} --batch -d ./test --recursive", env!("CARGO_PKG_NAME"));
        println!("  {} --batch --directory ./images --report-output report.json", env!("CARGO_PKG_NAME"));
    }
    
    /// 显示版本信息
    pub fn print_version() {
        println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        println!("基于 opencv-rust 的二维码解码器");
        println!("作者: {}", env!("CARGO_PKG_AUTHORS"));
        println!("许可证: MIT");
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_supported_formats() {
        assert!(Args::is_supported_format(&PathBuf::from("test.jpg")));
        assert!(Args::is_supported_format(&PathBuf::from("test.png")));
        assert!(Args::is_supported_format(&PathBuf::from("test.JPEG")));
        assert!(Args::is_supported_format(&PathBuf::from("test.bmp")));
        assert!(Args::is_supported_format(&PathBuf::from("test.tiff")));
        assert!(Args::is_supported_format(&PathBuf::from("test.webp")));
        assert!(!Args::is_supported_format(&PathBuf::from("test.txt")));
        assert!(!Args::is_supported_format(&PathBuf::from("test.pdf")));
        assert!(!Args::is_supported_format(&PathBuf::from("test")));
    }
    
    #[test]
    fn test_supported_formats_list() {
        let formats = Args::supported_formats();
        assert!(formats.contains(&"jpg"));
        assert!(formats.contains(&"png"));
        assert!(formats.contains(&"webp"));
        assert_eq!(formats.len(), 7);
    }
    
    #[test]
    fn test_help_args() {
        let args = Args::help_args();
        assert!(args.show_help);
        assert!(!args.show_version);
    }
    
    #[test]
    fn test_version_args() {
        let args = Args::version_args();
        assert!(!args.show_help);
        assert!(args.show_version);
    }
}