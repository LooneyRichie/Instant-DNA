use std::time::{Duration, Instant};
use std::process::Command;
use std::fs;
use std::path::Path;
use log::{info, warn};

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub tool_name: String,
    pub version: String,
    pub task: String,
    pub duration: Duration,
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug)]
pub struct BenchmarkSuite {
    pub results: Vec<BenchmarkResult>,
    pub test_data_path: String,
    pub iterations: usize,
}

impl BenchmarkSuite {
    pub fn new(test_data_path: String, iterations: usize) -> Self {
        Self {
            results: Vec::new(),
            test_data_path,
            iterations,
        }
    }

    pub fn run_all_benchmarks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🏁 Starting comprehensive DNA analysis benchmarks");
        info!("📊 Running {} iterations per tool", self.iterations);

        // Test data validation
        self.validate_test_data()?;

        // Benchmark tasks
        let tasks = vec![
            ("vcf_processing", "VCF file processing and parsing"),
            ("ancestry_analysis", "Ancestry estimation analysis"),
            ("variant_calling", "Variant calling and annotation"),
            ("sequence_alignment", "DNA sequence alignment"),
        ];

        for (task_id, task_desc) in tasks {
            info!("🧬 Benchmarking task: {}", task_desc);
            
            // Benchmark our tool
            self.benchmark_instant_dna(task_id)?;
            
            // Benchmark competitors
            self.benchmark_competitors(task_id)?;
        }

        Ok(())
    }

    fn validate_test_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let test_files = vec![
            "test_sample.vcf.gz",
            "reference_panel.txt",
            "benchmark_sequence.fasta",
        ];

        for file in test_files {
            let path = Path::new(&self.test_data_path).join(file);
            if !path.exists() {
                warn!("Test file not found: {}", path.display());
            }
        }

        Ok(())
    }

    fn benchmark_instant_dna(&mut self, task: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Benchmarking Instant DNA - {}", task);

        let mut total_duration = Duration::new(0, 0);
        let mut successful_runs = 0;

        for iteration in 1..=self.iterations {
            info!("  Iteration {}/{}", iteration, self.iterations);

            let start = Instant::now();
            let result = match task {
                "vcf_processing" => self.run_instant_dna_vcf(),
                "ancestry_analysis" => self.run_instant_dna_ancestry(),
                "variant_calling" => self.run_instant_dna_variants(),
                "sequence_alignment" => self.run_instant_dna_sequence(),
                _ => Ok(true),
            };

            let duration = start.elapsed();
            total_duration += duration;

            match result {
                Ok(true) => {
                    successful_runs += 1;
                    info!("    ✅ Success: {:.3}s", duration.as_secs_f64());
                }
                Ok(false) => {
                    warn!("    ❌ Failed: {:.3}s", duration.as_secs_f64());
                }
                Err(e) => {
                    warn!("    ❌ Error: {} ({:.3}s)", e, duration.as_secs_f64());
                }
            }
        }

        let avg_duration = total_duration / self.iterations as u32;
        let success_rate = (successful_runs as f64 / self.iterations as f64) * 100.0;

        self.results.push(BenchmarkResult {
            tool_name: "Instant DNA".to_string(),
            version: "2.0.0".to_string(),
            task: task.to_string(),
            duration: avg_duration,
            memory_mb: 0.0, // TODO: Implement memory monitoring
            cpu_percent: 0.0, // TODO: Implement CPU monitoring
            success: success_rate > 80.0,
            error_message: if success_rate < 80.0 {
                Some(format!("Success rate: {:.1}%", success_rate))
            } else {
                None
            },
        });

        info!("  📊 Average time: {:.3}s, Success rate: {:.1}%", 
              avg_duration.as_secs_f64(), success_rate);

        Ok(())
    }

    fn benchmark_competitors(&mut self, task: &str) -> Result<(), Box<dyn std::error::Error>> {
        let competitors = vec![
            ("PLINK", "plink"),
            ("BCFtools", "bcftools"),
            ("VCFtools", "vcftools"),
            ("GATK", "gatk"),
            ("SAMtools", "samtools"),
        ];

        for (name, command) in competitors {
            if self.is_tool_available(command) {
                info!("🔧 Benchmarking {} - {}", name, task);
                self.benchmark_competitor_tool(name, command, task)?;
            } else {
                warn!("⚠️  {} not available, skipping", name);
                self.results.push(BenchmarkResult {
                    tool_name: name.to_string(),
                    version: "N/A".to_string(),
                    task: task.to_string(),
                    duration: Duration::new(0, 0),
                    memory_mb: 0.0,
                    cpu_percent: 0.0,
                    success: false,
                    error_message: Some("Tool not installed".to_string()),
                });
            }
        }

        Ok(())
    }

    fn benchmark_competitor_tool(&mut self, name: &str, command: &str, task: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut total_duration = Duration::new(0, 0);
        let mut successful_runs = 0;

        for iteration in 1..=self.iterations {
            info!("  Iteration {}/{}", iteration, self.iterations);

            let start = Instant::now();
            let result = match task {
                "vcf_processing" => self.run_competitor_vcf(command),
                "ancestry_analysis" => self.run_competitor_ancestry(command),
                "variant_calling" => self.run_competitor_variants(command),
                "sequence_alignment" => self.run_competitor_alignment(command),
                _ => Ok(true),
            };

            let duration = start.elapsed();
            total_duration += duration;

            match result {
                Ok(true) => {
                    successful_runs += 1;
                    info!("    ✅ Success: {:.3}s", duration.as_secs_f64());
                }
                Ok(false) => {
                    warn!("    ❌ Failed: {:.3}s", duration.as_secs_f64());
                }
                Err(e) => {
                    warn!("    ❌ Error: {} ({:.3}s)", e, duration.as_secs_f64());
                }
            }
        }

        let avg_duration = total_duration / self.iterations as u32;
        let success_rate = (successful_runs as f64 / self.iterations as f64) * 100.0;
        let version = self.get_tool_version(command).unwrap_or("Unknown".to_string());

        self.results.push(BenchmarkResult {
            tool_name: name.to_string(),
            version,
            task: task.to_string(),
            duration: avg_duration,
            memory_mb: 0.0,
            cpu_percent: 0.0,
            success: success_rate > 80.0,
            error_message: if success_rate < 80.0 {
                Some(format!("Success rate: {:.1}%", success_rate))
            } else {
                None
            },
        });

        info!("  📊 Average time: {:.3}s, Success rate: {:.1}%", 
              avg_duration.as_secs_f64(), success_rate);

        Ok(())
    }

    fn is_tool_available(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn get_tool_version(&self, command: &str) -> Option<String> {
        let version_args = match command {
            "plink" => vec!["--version"],
            "bcftools" => vec!["--version"],
            "vcftools" => vec!["--version"],
            "gatk" => vec!["--version"],
            "samtools" => vec!["--version"],
            _ => vec!["--version"],
        };

        Command::new(command)
            .args(&version_args)
            .output()
            .ok()
            .and_then(|output| {
                String::from_utf8(output.stdout).ok()
                    .or_else(|| String::from_utf8(output.stderr).ok())
            })
            .and_then(|s| {
                s.lines().next().map(|line| {
                    line.split_whitespace()
                        .find(|word| word.chars().next().map_or(false, |c| c.is_digit(10)))
                        .unwrap_or("Unknown")
                        .to_string()
                })
            })
    }

    // Instant DNA benchmark methods
    fn run_instant_dna_vcf(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let vcf_path = format!("{}/test_sample.vcf.gz", self.test_data_path);
        let output_path = "/tmp/instant_dna_vcf_output.txt";
        let output = Command::new("./target/release/instant-dna")
            .args(&[
                "vcf",
                "--input", &vcf_path,
                "--output", output_path
            ])
            .output()?;

        Ok(output.status.success())
    }

    fn run_instant_dna_ancestry(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let vcf_path = format!("{}/test_sample.vcf.gz", self.test_data_path);
        let panel_path = format!("{}/reference_panel.txt", self.test_data_path);
        let output = Command::new("./target/release/instant-dna")
            .args(&[
                "ancestry",
                "--vcf", &vcf_path,
                "--panel", &panel_path,
                "--sample", "TEST_SAMPLE"
            ])
            .output()?;

        Ok(output.status.success())
    }

    fn run_instant_dna_variants(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let vcf_path = format!("{}/test_sample.vcf.gz", self.test_data_path);
        let output = Command::new("./target/release/instant-dna")
            .args(&[
                "variants",
                "--input", &vcf_path
            ])
            .output()?;

        Ok(output.status.success())
    }

    fn run_instant_dna_sequence(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let fasta_path = format!("{}/benchmark_sequence.fasta", self.test_data_path);
        let output = Command::new("./target/release/instant-dna")
            .args(&[
                "analyze",
                "--input", &fasta_path,
                "--format", "fasta"
            ])
            .output()?;

        Ok(output.status.success())
    }

    // Competitor benchmark methods
    fn run_competitor_vcf(&self, command: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let vcf_path = format!("{}/test_sample.vcf.gz", self.test_data_path);
        let args = match command {
            "bcftools" => vec!["stats", &vcf_path],
            "vcftools" => vec!["--gzvcf", &vcf_path, "--freq"],
            "plink" => vec!["--vcf", &vcf_path, "--freq", "--out", "/tmp/plink_test"],
            _ => return Ok(false),
        };

        let output = Command::new(command).args(&args).output()?;
        Ok(output.status.success())
    }

    fn run_competitor_ancestry(&self, command: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Most tools don't have direct ancestry analysis, so we'll do PCA analysis instead
        let vcf_path = format!("{}/test_sample.vcf.gz", self.test_data_path);
        let args = match command {
            "plink" => vec![
                "--vcf", &vcf_path,
                "--pca", "10",
                "--out", "/tmp/plink_pca"
            ],
            _ => return Ok(false), // Skip for tools without PCA
        };

        let output = Command::new(command).args(&args).output()?;
        Ok(output.status.success())
    }

    fn run_competitor_variants(&self, command: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let vcf_path = format!("{}/test_sample.vcf.gz", self.test_data_path);
        let args = match command {
            "bcftools" => vec!["view", "-H", &vcf_path],
            "vcftools" => vec!["--gzvcf", &vcf_path, "--site-pi"],
            _ => return Ok(false),
        };

        let output = Command::new(command).args(&args).output()?;
        Ok(output.status.success())
    }

    fn run_competitor_alignment(&self, _command: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Most VCF tools don't do sequence alignment, so we'll skip this
        Ok(false)
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("🏁 DNA Analysis Benchmark Results\n");
        report.push_str("=====================================\n\n");

        // Group results by task
        let mut tasks: std::collections::HashMap<String, Vec<&BenchmarkResult>> = std::collections::HashMap::new();
        for result in &self.results {
            tasks.entry(result.task.clone()).or_insert_with(Vec::new).push(result);
        }

        for (task, results) in tasks {
            report.push_str(&format!("📊 Task: {}\n", task));
            report.push_str("----------------------------------------\n");
            
            // Sort by duration (fastest first)
            let mut sorted_results = results;
            sorted_results.sort_by(|a, b| {
                if !a.success && b.success {
                    std::cmp::Ordering::Greater
                } else if a.success && !b.success {
                    std::cmp::Ordering::Less
                } else {
                    a.duration.cmp(&b.duration)
                }
            });

            let mut rank = 1;
            for result in sorted_results {
                let status = if result.success { "✅" } else { "❌" };
                let time_str = if result.success {
                    format!("{:.3}s", result.duration.as_secs_f64())
                } else {
                    "FAILED".to_string()
                };

                report.push_str(&format!(
                    "#{} {} {} ({}): {}\n",
                    rank, status, result.tool_name, result.version, time_str
                ));

                if let Some(error) = &result.error_message {
                    report.push_str(&format!("    Error: {}\n", error));
                }

                if result.success {
                    rank += 1;
                }
            }
            report.push_str("\n");
        }

        // Performance summary for Instant DNA
        report.push_str("🚀 Instant DNA Performance Summary\n");
        report.push_str("==================================\n");
        let instant_results: Vec<_> = self.results.iter()
            .filter(|r| r.tool_name == "Instant DNA" && r.success)
            .collect();

        if !instant_results.is_empty() {
            let avg_time: f64 = instant_results.iter()
                .map(|r| r.duration.as_secs_f64())
                .sum::<f64>() / instant_results.len() as f64;

            report.push_str(&format!("Average processing time: {:.3}s\n", avg_time));
            report.push_str(&format!("Successful tasks: {}/{}\n", instant_results.len(), self.results.iter().filter(|r| r.tool_name == "Instant DNA").count()));
            
            // Find speed improvements
            for task_name in ["vcf_processing", "ancestry_analysis", "variant_calling", "sequence_alignment"] {
                if let Some(instant_result) = self.results.iter().find(|r| r.tool_name == "Instant DNA" && r.task == task_name && r.success) {
                    let competitors: Vec<_> = self.results.iter()
                        .filter(|r| r.tool_name != "Instant DNA" && r.task == task_name && r.success)
                        .collect();
                    
                    if !competitors.is_empty() {
                        let fastest_competitor = competitors.iter()
                            .min_by(|a, b| a.duration.cmp(&b.duration))
                            .unwrap();
                        
                        let speedup = fastest_competitor.duration.as_secs_f64() / instant_result.duration.as_secs_f64();
                        if speedup > 1.0 {
                            report.push_str(&format!("{}x faster than {} in {}\n", 
                                speedup, fastest_competitor.tool_name, task_name));
                        }
                    }
                }
            }
        }

        report
    }

    pub fn save_report(&self, filename: &str) -> Result<(), std::io::Error> {
        fs::write(filename, self.generate_report())
    }
}
