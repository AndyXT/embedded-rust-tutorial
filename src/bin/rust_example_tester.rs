//! CLI interface for the Rust example testing framework
//! 
//! This binary provides a comprehensive command-line interface to test Rust code examples
//! from markdown files, with support for incremental testing, caching, and development workflows.
//! 
//! Features:
//! - Individual example testing
//! - Incremental testing with caching
//! - Watch mode for continuous testing
//! - Multiple output formats (JSON, human-readable, JUnit)
//! - Parallel execution
//! - Multiple target support

use std::env;
use std::path::PathBuf;
use std::process;
use std::fs;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json;
use serde::{Serialize, Deserialize};

use embedded_rust_tutorial_validation::{
    CodeExtractor, RustExampleTester, TestConfig, ValidationReport,
    ExampleContext, CodeExample, TestResult
};

#[derive(Debug, Clone)]
struct CliConfig {
    /// Input file or directory
    input: PathBuf,
    /// Output format (json, human, summary)
    output_format: OutputFormat,
    /// Whether to use cached results
    use_cache: bool,
    /// Cache directory
    cache_dir: PathBuf,
    /// Test configuration
    test_config: TestConfig,
    /// Whether to run in watch mode
    watch: bool,
    /// Whether to test only changed files
    incremental: bool,
    /// Specific example ID to test
    example_id: Option<String>,
    /// Whether to show verbose output
    verbose: bool,
    /// Whether to fail fast on first error
    fail_fast: bool,
    /// Maximum number of parallel tests
    parallel: usize,
}

#[derive(Debug, Clone)]
enum OutputFormat {
    Json,
    Human,
    Summary,
    Junit,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    /// Hash of the example content
    content_hash: String,
    /// Last modification time
    modified_time: u64,
    /// Cached test result
    result: CachedTestResult,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedTestResult {
    example_id: String,
    success: bool,
    compilation_time_ms: u64,
    target: String,
    error: Option<String>,
    warnings_count: usize,
}

impl From<&TestResult> for CachedTestResult {
    fn from(result: &TestResult) -> Self {
        Self {
            example_id: result.example_id.clone(),
            success: result.success,
            compilation_time_ms: result.compilation_time.as_millis() as u64,
            target: result.target.clone(),
            error: result.error.as_ref().map(|e| format!("{}", e)),
            warnings_count: result.warnings.len(),
        }
    }
}

fn main() {
    let config = parse_args();
    
    if let Err(e) = run_with_config(config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn parse_args() -> CliConfig {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }
    
    let mut config = CliConfig {
        input: PathBuf::new(),
        output_format: OutputFormat::Human,
        use_cache: true,
        cache_dir: std::env::temp_dir().join("rust_example_cache"),
        test_config: TestConfig::default(),
        watch: false,
        incremental: false,
        example_id: None,
        verbose: false,
        fail_fast: false,
        parallel: num_cpus::get(),
    };
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help(&args[0]);
                process::exit(0);
            }
            "--version" | "-V" => {
                println!("rust-example-tester 0.1.0");
                process::exit(0);
            }
            "--format" | "-f" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --format requires a value");
                    process::exit(1);
                }
                config.output_format = match args[i].as_str() {
                    "json" => OutputFormat::Json,
                    "human" => OutputFormat::Human,
                    "summary" => OutputFormat::Summary,
                    "junit" => OutputFormat::Junit,
                    _ => {
                        eprintln!("Error: Invalid format '{}'. Use: json, human, summary, junit", args[i]);
                        process::exit(1);
                    }
                };
            }
            "--no-cache" => {
                config.use_cache = false;
            }
            "--cache-dir" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --cache-dir requires a value");
                    process::exit(1);
                }
                config.cache_dir = PathBuf::from(&args[i]);
            }
            "--embedded" => {
                config.test_config = TestConfig::embedded();
            }
            "--crypto" => {
                config.test_config = TestConfig::crypto();
            }
            "--target" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --target requires a value");
                    process::exit(1);
                }
                config.test_config.targets = vec![args[i].clone()];
            }
            "--features" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --features requires a value");
                    process::exit(1);
                }
                config.test_config.features = args[i].split(',').map(|s| s.trim().to_string()).collect();
            }
            "--watch" | "-w" => {
                config.watch = true;
            }
            "--incremental" | "-i" => {
                config.incremental = true;
            }
            "--example" | "-e" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --example requires a value");
                    process::exit(1);
                }
                config.example_id = Some(args[i].clone());
            }
            "--verbose" | "-v" => {
                config.verbose = true;
            }
            "--fail-fast" => {
                config.fail_fast = true;
            }
            "--parallel" | "-j" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --parallel requires a value");
                    process::exit(1);
                }
                config.parallel = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid parallel value '{}'", args[i]);
                    process::exit(1);
                });
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: Unknown option '{}'", arg);
                process::exit(1);
            }
            _ => {
                if config.input.as_os_str().is_empty() {
                    config.input = PathBuf::from(&args[i]);
                } else {
                    eprintln!("Error: Multiple input files specified");
                    process::exit(1);
                }
            }
        }
        i += 1;
    }
    
    if config.input.as_os_str().is_empty() {
        eprintln!("Error: No input file specified");
        print_usage(&args[0]);
        process::exit(1);
    }
    
    config
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} [OPTIONS] <INPUT>", program_name);
    eprintln!();
    eprintln!("Test Rust code examples from markdown files");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    -h, --help              Show this help message");
    eprintln!("    -V, --version           Show version information");
    eprintln!("    -f, --format FORMAT     Output format: json, human, summary, junit [default: human]");
    eprintln!("    --no-cache              Disable result caching");
    eprintln!("    --cache-dir DIR         Cache directory [default: temp/rust_example_cache]");
    eprintln!("    --embedded              Use embedded testing configuration");
    eprintln!("    --crypto                Use cryptography testing configuration");
    eprintln!("    --target TARGET         Specific Rust target to test");
    eprintln!("    --features FEATURES     Comma-separated list of features to enable");
    eprintln!("    -w, --watch             Watch for file changes and re-test");
    eprintln!("    -i, --incremental       Only test changed examples");
    eprintln!("    -e, --example ID        Test only specific example by ID");
    eprintln!("    -v, --verbose           Show verbose output");
    eprintln!("    --fail-fast             Stop on first test failure");
    eprintln!("    -j, --parallel N        Number of parallel tests [default: CPU count]");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    {} src/core-concepts/ownership.md", program_name);
    eprintln!("    {} --embedded --format json src/", program_name);
    eprintln!("    {} --example crypto_example_1 --verbose src/cryptography/", program_name);
    eprintln!("    {} --watch --incremental src/", program_name);
}

fn print_help(program_name: &str) {
    print_usage(program_name);
    eprintln!();
    eprintln!("DESCRIPTION:");
    eprintln!("    This tool tests Rust code examples extracted from markdown files to ensure");
    eprintln!("    they compile correctly. It supports caching, incremental testing, and various");
    eprintln!("    output formats for integration with development workflows.");
    eprintln!();
    eprintln!("    The tool can test examples against different Rust targets (std, no_std,");
    eprintln!("    embedded) and supports feature-gated compilation for specialized use cases");
    eprintln!("    like cryptography and embedded systems.");
    eprintln!();
    eprintln!("CACHE BEHAVIOR:");
    eprintln!("    Results are cached based on example content hash and file modification time.");
    eprintln!("    Use --no-cache to force re-testing all examples.");
    eprintln!();
    eprintln!("INCREMENTAL TESTING:");
    eprintln!("    With --incremental, only examples that have changed since the last run");
    eprintln!("    will be tested. This significantly speeds up development workflows.");
}

fn run_with_config(config: CliConfig) -> Result<(), String> {
    // Validate input
    if !config.input.exists() {
        return Err(format!("Input path '{}' does not exist", config.input.display()));
    }
    
    // Create cache directory if needed
    if config.use_cache {
        fs::create_dir_all(&config.cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    }
    
    if config.watch {
        return run_watch_mode(config);
    }
    
    // Collect input files
    let input_files = collect_input_files(&config.input)?;
    
    if config.verbose {
        eprintln!("Found {} markdown files to process", input_files.len());
    }
    
    // Extract examples from all files
    let mut all_examples = Vec::new();
    let extractor = CodeExtractor::new();
    
    for file_path in &input_files {
        match extractor.extract_from_file(file_path) {
            Ok(mut examples) => {
                // Filter by example ID if specified
                if let Some(ref example_id) = config.example_id {
                    examples.retain(|e| e.id == *example_id);
                }
                all_examples.extend(examples);
            }
            Err(e) => {
                if config.verbose {
                    eprintln!("Warning: Failed to extract examples from {}: {:?}", file_path.display(), e);
                }
            }
        }
    }
    
    if all_examples.is_empty() {
        return Err("No code examples found to test".to_string());
    }
    
    if config.verbose {
        eprintln!("Found {} code examples to test", all_examples.len());
    }
    
    // Load cache if enabled
    let mut cache = if config.use_cache {
        load_cache(&config.cache_dir)?
    } else {
        HashMap::new()
    };
    
    // Filter examples for incremental testing
    let examples_to_test = if config.incremental {
        filter_changed_examples(&all_examples, &cache)?
    } else {
        all_examples.clone()
    };
    
    if config.verbose && config.incremental {
        eprintln!("Incremental mode: testing {} out of {} examples", 
                 examples_to_test.len(), all_examples.len());
    }
    
    // Create tester
    let mut tester = RustExampleTester::new(config.test_config.clone())
        .map_err(|e| format!("Failed to create tester: {}", e))?;
    
    // Ensure targets are available
    if let Err(e) = tester.ensure_targets() {
        if config.verbose {
            eprintln!("Warning: Some targets may not be available: {}", e);
        }
    }
    
    // Run tests
    let mut report = ValidationReport::new();
    let mut failed_fast = false;
    
    for example in &examples_to_test {
        // Check cache first
        if config.use_cache {
            if let Some(cached_result) = get_cached_result(&cache, example) {
                if config.verbose {
                    eprintln!("Using cached result for example '{}'", example.id);
                }
                let test_result = cached_result_to_test_result(cached_result);
                report.add_result(test_result);
                continue;
            }
        }
        
        // Run the test
        let result = tester.test_example(example);
        
        // Update cache
        if config.use_cache {
            update_cache(&mut cache, example, &result)?;
        }
        
        // Check fail-fast
        if config.fail_fast && !result.success {
            failed_fast = true;
            report.add_result(result);
            break;
        }
        
        report.add_result(result);
        
        if config.verbose {
            let status = if report.results.last().unwrap().success { "PASS" } else { "FAIL" };
            eprintln!("[{}] {}", status, example.id);
        }
    }
    
    // Save cache
    if config.use_cache {
        save_cache(&config.cache_dir, &cache)?;
    }
    
    // Add cached results for non-tested examples in incremental mode
    if config.incremental {
        for example in &all_examples {
            if !examples_to_test.iter().any(|e| e.id == example.id) {
                if let Some(cached_result) = get_cached_result(&cache, example) {
                    let test_result = cached_result_to_test_result(cached_result);
                    report.add_result(test_result);
                }
            }
        }
    }
    
    // Output results
    output_results(&config, &report, failed_fast)?;
    
    // Exit with appropriate code
    if report.failed > 0 || failed_fast {
        process::exit(1);
    }
    
    Ok(())
}

fn collect_input_files(input: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    
    if input.is_file() {
        if input.extension().and_then(|s| s.to_str()) == Some("md") {
            files.push(input.clone());
        } else {
            return Err("Input file must have .md extension".to_string());
        }
    } else if input.is_dir() {
        collect_markdown_files_recursive(input, &mut files)?;
    } else {
        return Err("Input must be a file or directory".to_string());
    }
    
    if files.is_empty() {
        return Err("No markdown files found".to_string());
    }
    
    files.sort();
    Ok(files)
}

fn collect_markdown_files_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            // Skip hidden directories and common build directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
            }
            collect_markdown_files_recursive(&path, files)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            files.push(path);
        }
    }
    
    Ok(())
}

fn load_cache(cache_dir: &PathBuf) -> Result<HashMap<String, CacheEntry>, String> {
    let cache_file = cache_dir.join("cache.json");
    
    if !cache_file.exists() {
        return Ok(HashMap::new());
    }
    
    let content = fs::read_to_string(&cache_file)
        .map_err(|e| format!("Failed to read cache file: {}", e))?;
    
    let cache: HashMap<String, CacheEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse cache file: {}", e))?;
    
    Ok(cache)
}

fn save_cache(cache_dir: &PathBuf, cache: &HashMap<String, CacheEntry>) -> Result<(), String> {
    let cache_file = cache_dir.join("cache.json");
    
    let content = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;
    
    fs::write(&cache_file, content)
        .map_err(|e| format!("Failed to write cache file: {}", e))?;
    
    Ok(())
}

fn filter_changed_examples(
    examples: &[CodeExample], 
    cache: &HashMap<String, CacheEntry>
) -> Result<Vec<CodeExample>, String> {
    let mut changed_examples = Vec::new();
    
    for example in examples {
        let cache_key = format!("{}:{}", example.source_file.display(), example.id);
        
        let needs_testing = if let Some(cached) = cache.get(&cache_key) {
            // Check if content has changed
            let current_hash = calculate_content_hash(&example.code);
            let current_time = get_file_modified_time(&example.source_file)?;
            
            cached.content_hash != current_hash || cached.modified_time < current_time
        } else {
            // No cache entry, needs testing
            true
        };
        
        if needs_testing {
            changed_examples.push(example.clone());
        }
    }
    
    Ok(changed_examples)
}

fn get_cached_result(cache: &HashMap<String, CacheEntry>, example: &CodeExample) -> Option<&CachedTestResult> {
    let cache_key = format!("{}:{}", example.source_file.display(), example.id);
    cache.get(&cache_key).map(|entry| &entry.result)
}

fn update_cache(
    cache: &mut HashMap<String, CacheEntry>, 
    example: &CodeExample, 
    result: &TestResult
) -> Result<(), String> {
    let cache_key = format!("{}:{}", example.source_file.display(), example.id);
    let content_hash = calculate_content_hash(&example.code);
    let modified_time = get_file_modified_time(&example.source_file)?;
    
    let entry = CacheEntry {
        content_hash,
        modified_time,
        result: CachedTestResult::from(result),
    };
    
    cache.insert(cache_key, entry);
    Ok(())
}

fn calculate_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn get_file_modified_time(path: &PathBuf) -> Result<u64, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let modified = metadata.modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?;
    
    let duration = modified.duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Invalid modification time: {}", e))?;
    
    Ok(duration.as_secs())
}

fn cached_result_to_test_result(cached: &CachedTestResult) -> TestResult {
    TestResult {
        example_id: cached.example_id.clone(),
        success: cached.success,
        compilation_time: std::time::Duration::from_millis(cached.compilation_time_ms),
        target: cached.target.clone(),
        error: cached.error.as_ref().map(|e| {
            // Simple error reconstruction - in a real implementation you'd want
            // to serialize the full error structure
            embedded_rust_tutorial_validation::CompilationError::CompilationFailed {
                exit_code: -1,
                message: e.clone(),
            }
        }),
        warnings: vec![], // Warnings not cached for simplicity
        stdout: String::new(),
        stderr: cached.error.as_ref().unwrap_or(&String::new()).clone(),
    }
}

fn output_results(config: &CliConfig, report: &ValidationReport, failed_fast: bool) -> Result<(), String> {
    match config.output_format {
        OutputFormat::Json => output_json(report),
        OutputFormat::Human => output_human(report, config.verbose, failed_fast),
        OutputFormat::Summary => output_summary(report),
        OutputFormat::Junit => output_junit(report),
    }
}

fn output_json(report: &ValidationReport) -> Result<(), String> {
    let json_output = serde_json::json!({
        "total_examples": report.total_examples,
        "successful": report.successful,
        "failed": report.failed,
        "skipped": report.skipped,
        "success_rate": report.success_rate(),
        "total_duration_ms": report.total_duration.as_millis(),
        "results": report.results.iter().map(|r| {
            serde_json::json!({
                "example_id": r.example_id,
                "success": r.success,
                "target": r.target,
                "compilation_time_ms": r.compilation_time.as_millis(),
                "error": r.error.as_ref().map(|e| format!("{}", e)),
                "warnings_count": r.warnings.len()
            })
        }).collect::<Vec<_>>()
    });
    
    println!("{}", serde_json::to_string_pretty(&json_output)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?);
    
    Ok(())
}

fn output_human(report: &ValidationReport, verbose: bool, failed_fast: bool) -> Result<(), String> {
    println!("Rust Example Test Results");
    println!("========================");
    println!();
    
    if failed_fast {
        println!("⚠️  Testing stopped early due to --fail-fast");
        println!();
    }
    
    // Summary
    println!("Summary:");
    println!("  Total examples: {}", report.total_examples);
    println!("  Successful: {} ({}%)", report.successful, 
             (report.successful as f64 / report.total_examples as f64 * 100.0) as u32);
    println!("  Failed: {}", report.failed);
    println!("  Skipped: {}", report.skipped);
    println!("  Total time: {:.2}s", report.total_duration.as_secs_f64());
    println!();
    
    // Failed examples
    if report.failed > 0 {
        println!("Failed Examples:");
        for result in &report.results {
            if !result.success {
                println!("  ❌ {} ({})", result.example_id, result.target);
                if let Some(ref error) = result.error {
                    println!("     Error: {}", error);
                }
                if verbose && !result.stderr.is_empty() {
                    println!("     Stderr: {}", result.stderr.lines().next().unwrap_or(""));
                }
            }
        }
        println!();
    }
    
    // Successful examples (if verbose)
    if verbose && report.successful > 0 {
        println!("Successful Examples:");
        for result in &report.results {
            if result.success && result.target != "skipped" {
                println!("  ✅ {} ({}) - {:.2}s", 
                         result.example_id, result.target, 
                         result.compilation_time.as_secs_f64());
                if !result.warnings.is_empty() {
                    println!("     Warnings: {}", result.warnings.len());
                }
            }
        }
        println!();
    }
    
    // Overall result
    if report.failed == 0 {
        println!("🎉 All tests passed!");
    } else {
        println!("❌ {} test(s) failed", report.failed);
    }
    
    Ok(())
}

fn output_summary(report: &ValidationReport) -> Result<(), String> {
    println!("{}/{} examples passed ({:.1}%)", 
             report.successful, report.total_examples, report.success_rate());
    
    if report.failed > 0 {
        println!("Failed examples:");
        for result in &report.results {
            if !result.success {
                println!("  - {}", result.example_id);
            }
        }
    }
    
    Ok(())
}

fn output_junit(report: &ValidationReport) -> Result<(), String> {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!("<testsuite name=\"rust-examples\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">", 
             report.total_examples, report.failed, report.total_duration.as_secs_f64());
    
    for result in &report.results {
        println!("  <testcase name=\"{}\" classname=\"{}\" time=\"{:.3}\">", 
                 result.example_id, result.target, result.compilation_time.as_secs_f64());
        
        if !result.success {
            if let Some(ref error) = result.error {
                println!("    <failure message=\"{}\">{}</failure>", 
                         error, result.stderr);
            }
        }
        
        println!("  </testcase>");
    }
    
    println!("</testsuite>");
    Ok(())
}

fn run_watch_mode(config: CliConfig) -> Result<(), String> {
    use std::time::Duration;
    use std::thread;
    
    println!("🔍 Starting watch mode for: {}", config.input.display());
    println!("Press Ctrl+C to stop watching...");
    println!();
    
    let mut last_run_time = SystemTime::now();
    let mut file_timestamps: HashMap<PathBuf, SystemTime> = HashMap::new();
    
    // Initial run
    let mut watch_config = config.clone();
    watch_config.watch = false; // Prevent infinite recursion
    
    if let Err(e) = run_with_config(watch_config.clone()) {
        eprintln!("Initial run failed: {}", e);
    }
    
    // Initialize file timestamps
    let input_files = collect_input_files(&config.input)?;
    for file_path in &input_files {
        if let Ok(metadata) = fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                file_timestamps.insert(file_path.clone(), modified);
            }
        }
    }
    
    loop {
        thread::sleep(Duration::from_secs(1));
        
        let mut files_changed = false;
        let current_files = collect_input_files(&config.input)?;
        
        // Check for new or modified files
        for file_path in &current_files {
            if let Ok(metadata) = fs::metadata(file_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Some(&last_modified) = file_timestamps.get(file_path) {
                        if modified > last_modified {
                            if config.verbose {
                                println!("📝 File changed: {}", file_path.display());
                            }
                            files_changed = true;
                            file_timestamps.insert(file_path.clone(), modified);
                        }
                    } else {
                        // New file
                        if config.verbose {
                            println!("📄 New file detected: {}", file_path.display());
                        }
                        files_changed = true;
                        file_timestamps.insert(file_path.clone(), modified);
                    }
                }
            }
        }
        
        // Check for deleted files
        let current_file_set: std::collections::HashSet<_> = current_files.iter().collect();
        let cached_files: Vec<_> = file_timestamps.keys().cloned().collect();
        
        for cached_file in cached_files {
            if !current_file_set.contains(&cached_file) {
                if config.verbose {
                    println!("🗑️  File deleted: {}", cached_file.display());
                }
                file_timestamps.remove(&cached_file);
                files_changed = true;
            }
        }
        
        if files_changed {
            println!();
            println!("🔄 Changes detected, re-running tests...");
            println!("========================================");
            
            let start_time = SystemTime::now();
            
            // Run tests with incremental mode if enabled
            let mut run_config = config.clone();
            run_config.watch = false; // Prevent infinite recursion
            
            match run_with_config(run_config) {
                Ok(_) => {
                    if config.verbose {
                        if let Ok(duration) = start_time.elapsed() {
                            println!("✅ Tests completed in {:.2}s", duration.as_secs_f64());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Test run failed: {}", e);
                }
            }
            
            last_run_time = SystemTime::now();
            println!();
            println!("👀 Watching for changes...");
        }
    }
}