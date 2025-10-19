use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::collections::HashMap;
use anyhow::{Result, Context};
use log::{info, warn, debug};
use regex::Regex;

#[derive(Debug, Clone)]
pub enum InputFormat {
    /// 23andMe raw data format
    TwentyThreeAndMe,
    /// AncestryDNA raw data format
    AncestryDna,
    /// MyHeritage raw data format
    MyHeritage,
    /// FamilyTreeDNA raw data format
    FamilyTreeDna,
    /// Generic CSV format
    Csv,
    /// Tab-delimited text
    Tab,
    /// PLINK PED/MAP format
    Plink,
    /// Auto-detect format
    Auto,
}

#[derive(Debug, Clone)]
pub struct SnpData {
    pub rsid: String,
    pub chromosome: String,
    pub position: u64,
    pub genotype: String,
    pub reference: Option<String>,
    pub alternate: Option<String>,
}

#[derive(Debug)]
pub struct RawDnaConverter {
    pub sample_name: String,
    pub input_format: InputFormat,
    pub output_compression: bool,
}

impl RawDnaConverter {
    pub fn new(sample_name: String, input_format: InputFormat) -> Self {
        Self {
            sample_name,
            input_format,
            output_compression: true,
        }
    }

    pub fn auto_detect_format(&self, input_path: &str) -> Result<InputFormat> {
        info!("🔍 Auto-detecting file format for: {}", input_path);
        
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().take(10).collect::<Result<Vec<_>, _>>()?;
        
        // Remove empty lines and comments
        lines.retain(|line| !line.trim().is_empty() && !line.starts_with('#'));
        
        if lines.is_empty() {
            return Ok(InputFormat::Auto);
        }

        let first_line = &lines[0];
        let header_lower = first_line.to_lowercase();
        
        // Check for 23andMe format
        if header_lower.contains("rsid") && header_lower.contains("chromosome") 
            && header_lower.contains("position") && header_lower.contains("genotype") {
            info!("✅ Detected 23andMe format");
            return Ok(InputFormat::TwentyThreeAndMe);
        }
        
        // Check for AncestryDNA format
        if header_lower.contains("rsid") && header_lower.contains("chrom") 
            && header_lower.contains("pos") && header_lower.contains("allele1") {
            info!("✅ Detected AncestryDNA format");
            return Ok(InputFormat::AncestryDna);
        }
        
        // Check for MyHeritage format
        if header_lower.contains("rsid") && header_lower.contains("chr") 
            && header_lower.contains("pos") && (header_lower.contains("result") || header_lower.contains("genotype")) {
            info!("✅ Detected MyHeritage format");
            return Ok(InputFormat::MyHeritage);
        }
        
        // Check for FamilyTreeDNA format
        if header_lower.contains("rsid") && header_lower.contains("chromosome") 
            && header_lower.contains("position") && header_lower.contains("result") {
            info!("✅ Detected FamilyTreeDNA format");
            return Ok(InputFormat::FamilyTreeDna);
        }
        
        // Check if it's CSV or tab-delimited
        let tab_count = first_line.matches('\t').count();
        let comma_count = first_line.matches(',').count();
        
        if comma_count > tab_count && comma_count > 2 {
            info!("✅ Detected CSV format");
            return Ok(InputFormat::Csv);
        } else if tab_count > 2 {
            info!("✅ Detected Tab-delimited format");
            return Ok(InputFormat::Tab);
        }
        
        warn!("⚠️ Could not auto-detect format, defaulting to generic CSV");
        Ok(InputFormat::Csv)
    }

    pub fn convert_to_vcf(&self, input_path: &str, output_path: &str) -> Result<()> {
        info!("🧬 Converting raw DNA data to VCF format");
        info!("📂 Input: {}", input_path);
        info!("📁 Output: {}", output_path);
        
        // Detect format if auto
        let format = match &self.input_format {
            InputFormat::Auto => self.auto_detect_format(input_path)?,
            _ => self.input_format.clone(),
        };
        
        info!("🔧 Using format: {:?}", format);
        
        // Parse input file
        let snp_data = self.parse_input_file(input_path, &format)?;
        info!("📊 Parsed {} SNPs", snp_data.len());
        
        // Write VCF file
        self.write_vcf_file(output_path, &snp_data)?;
        info!("✅ VCF conversion completed successfully");
        
        Ok(())
    }

    fn parse_input_file(&self, input_path: &str, format: &InputFormat) -> Result<Vec<SnpData>> {
        let file = File::open(input_path)
            .with_context(|| format!("Failed to open input file: {}", input_path))?;
        let reader = BufReader::new(file);
        
        match format {
            InputFormat::TwentyThreeAndMe => self.parse_23andme(reader),
            InputFormat::AncestryDna => self.parse_ancestry_dna(reader),
            InputFormat::MyHeritage => self.parse_myheritage(reader),
            InputFormat::FamilyTreeDna => self.parse_familytree_dna(reader),
            InputFormat::Csv => self.parse_csv(reader),
            InputFormat::Tab => self.parse_tab(reader),
            InputFormat::Plink => self.parse_plink(input_path),
            InputFormat::Auto => unreachable!("Auto format should be resolved by now"),
        }
    }

    fn parse_23andme(&self, reader: BufReader<File>) -> Result<Vec<SnpData>> {
        let mut snps = Vec::new();
        let mut lines = reader.lines();
        
        // Skip comments and find header
        let mut header_line = String::new();
        for line in lines.by_ref() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            header_line = line;
            break;
        }
        
        let headers: Vec<&str> = header_line.split('\t').collect();
        let rsid_idx = headers.iter().position(|&h| h.to_lowercase() == "rsid").unwrap_or(0);
        let chr_idx = headers.iter().position(|&h| h.to_lowercase() == "chromosome").unwrap_or(1);
        let pos_idx = headers.iter().position(|&h| h.to_lowercase() == "position").unwrap_or(2);
        let genotype_idx = headers.iter().position(|&h| h.to_lowercase() == "genotype").unwrap_or(3);
        
        for line in lines {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 4 {
                continue;
            }
            
            let chromosome = self.normalize_chromosome(fields.get(chr_idx).unwrap_or(&"0"));
            if chromosome == "0" {
                continue; // Skip invalid chromosomes
            }
            
            let genotype = fields.get(genotype_idx).unwrap_or(&"--").to_string();
            if genotype == "--" || genotype == "0" {
                continue; // Skip no-calls
            }
            
            snps.push(SnpData {
                rsid: fields.get(rsid_idx).unwrap_or(&"").to_string(),
                chromosome,
                position: fields.get(pos_idx).unwrap_or(&"0").parse().unwrap_or(0),
                genotype,
                reference: None,
                alternate: None,
            });
        }
        
        Ok(snps)
    }

    fn parse_ancestry_dna(&self, reader: BufReader<File>) -> Result<Vec<SnpData>> {
        let mut snps = Vec::new();
        let mut lines = reader.lines();
        
        // Find header
        let mut header_line = String::new();
        for line in lines.by_ref() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            header_line = line;
            break;
        }
        
        let headers: Vec<&str> = header_line.split('\t').collect();
        let rsid_idx = headers.iter().position(|&h| h.to_lowercase().contains("rsid")).unwrap_or(0);
        let chr_idx = headers.iter().position(|&h| h.to_lowercase().contains("chrom")).unwrap_or(1);
        let pos_idx = headers.iter().position(|&h| h.to_lowercase().contains("pos")).unwrap_or(2);
        let allele1_idx = headers.iter().position(|&h| h.to_lowercase().contains("allele1")).unwrap_or(3);
        let allele2_idx = headers.iter().position(|&h| h.to_lowercase().contains("allele2")).unwrap_or(4);
        
        for line in lines {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 5 {
                continue;
            }
            
            let chromosome = self.normalize_chromosome(fields.get(chr_idx).unwrap_or(&"0"));
            if chromosome == "0" {
                continue;
            }
            
            let allele1 = fields.get(allele1_idx).unwrap_or(&"0");
            let allele2 = fields.get(allele2_idx).unwrap_or(&"0");
            
            if *allele1 == "0" || *allele2 == "0" {
                continue; // Skip no-calls
            }
            
            let genotype = format!("{}{}", allele1, allele2);
            
            snps.push(SnpData {
                rsid: fields.get(rsid_idx).unwrap_or(&"").to_string(),
                chromosome,
                position: fields.get(pos_idx).unwrap_or(&"0").parse().unwrap_or(0),
                genotype,
                reference: None,
                alternate: None,
            });
        }
        
        Ok(snps)
    }

    fn parse_myheritage(&self, reader: BufReader<File>) -> Result<Vec<SnpData>> {
        // Similar to 23andMe but with slightly different column names
        let mut snps = Vec::new();
        let mut lines = reader.lines();
        
        let mut header_line = String::new();
        for line in lines.by_ref() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            header_line = line;
            break;
        }
        
        // MyHeritage uses comma separation
        let headers: Vec<&str> = if header_line.contains(',') {
            header_line.split(',').collect()
        } else {
            header_line.split('\t').collect()
        };
        
        let rsid_idx = headers.iter().position(|&h| h.to_lowercase().contains("rsid")).unwrap_or(0);
        let chr_idx = headers.iter().position(|&h| h.to_lowercase().contains("chr")).unwrap_or(1);
        let pos_idx = headers.iter().position(|&h| h.to_lowercase().contains("pos")).unwrap_or(2);
        let result_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("result") || h_lower.contains("genotype")
        }).unwrap_or(3);
        
        for line in lines {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = if line.contains(',') {
                line.split(',').collect()
            } else {
                line.split('\t').collect()
            };
            
            if fields.len() < 4 {
                continue;
            }
            
            let chromosome = self.normalize_chromosome(fields.get(chr_idx).unwrap_or(&"0"));
            if chromosome == "0" {
                continue;
            }
            
            let genotype = fields.get(result_idx).unwrap_or(&"--").trim_matches('"').to_string();
            if genotype == "--" || genotype.is_empty() {
                continue;
            }
            
            snps.push(SnpData {
                rsid: fields.get(rsid_idx).unwrap_or(&"").trim_matches('"').to_string(),
                chromosome,
                position: fields.get(pos_idx).unwrap_or(&"0").parse().unwrap_or(0),
                genotype,
                reference: None,
                alternate: None,
            });
        }
        
        Ok(snps)
    }

    fn parse_familytree_dna(&self, reader: BufReader<File>) -> Result<Vec<SnpData>> {
        // Similar structure to 23andMe
        self.parse_23andme(reader)
    }

    fn parse_csv(&self, reader: BufReader<File>) -> Result<Vec<SnpData>> {
        // Generic CSV parser - try to detect columns automatically
        let mut snps = Vec::new();
        let mut lines = reader.lines();
        
        let mut header_line = String::new();
        for line in lines.by_ref() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            header_line = line;
            break;
        }
        
        let headers: Vec<&str> = header_line.split(',').collect();
        
        // Try to find relevant columns
        let rsid_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("rsid") || h_lower.contains("snp") || h_lower.contains("marker")
        });
        
        let chr_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("chr") || h_lower.contains("chrom")
        });
        
        let pos_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("pos") || h_lower.contains("location")
        });
        
        let genotype_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("genotype") || h_lower.contains("result") || h_lower.contains("call")
        });
        
        if rsid_idx.is_none() || chr_idx.is_none() || pos_idx.is_none() || genotype_idx.is_none() {
            warn!("⚠️ Could not identify all required columns in CSV");
            return Ok(snps);
        }
        
        for line in lines {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = line.split(',').collect();
            
            let chromosome = self.normalize_chromosome(fields.get(chr_idx.unwrap()).unwrap_or(&"0"));
            if chromosome == "0" {
                continue;
            }
            
            let genotype = fields.get(genotype_idx.unwrap()).unwrap_or(&"--").trim_matches('"').to_string();
            if genotype == "--" || genotype.is_empty() {
                continue;
            }
            
            snps.push(SnpData {
                rsid: fields.get(rsid_idx.unwrap()).unwrap_or(&"").trim_matches('"').to_string(),
                chromosome,
                position: fields.get(pos_idx.unwrap()).unwrap_or(&"0").parse().unwrap_or(0),
                genotype,
                reference: None,
                alternate: None,
            });
        }
        
        Ok(snps)
    }

    fn parse_tab(&self, reader: BufReader<File>) -> Result<Vec<SnpData>> {
        // Similar to CSV but tab-separated
        let mut snps = Vec::new();
        let mut lines = reader.lines();
        
        let mut header_line = String::new();
        for line in lines.by_ref() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            header_line = line;
            break;
        }
        
        let headers: Vec<&str> = header_line.split('\t').collect();
        
        // Find columns (same logic as CSV)
        let rsid_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("rsid") || h_lower.contains("snp") || h_lower.contains("marker")
        }).unwrap_or(0);
        
        let chr_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("chr") || h_lower.contains("chrom")
        }).unwrap_or(1);
        
        let pos_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("pos") || h_lower.contains("location")
        }).unwrap_or(2);
        
        let genotype_idx = headers.iter().position(|&h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("genotype") || h_lower.contains("result") || h_lower.contains("call")
        }).unwrap_or(3);
        
        for line in lines {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() <= genotype_idx {
                continue;
            }
            
            let chromosome = self.normalize_chromosome(fields.get(chr_idx).unwrap_or(&"0"));
            if chromosome == "0" {
                continue;
            }
            
            let genotype = fields.get(genotype_idx).unwrap_or(&"--").to_string();
            if genotype == "--" || genotype.is_empty() {
                continue;
            }
            
            snps.push(SnpData {
                rsid: fields.get(rsid_idx).unwrap_or(&"").to_string(),
                chromosome,
                position: fields.get(pos_idx).unwrap_or(&"0").parse().unwrap_or(0),
                genotype,
                reference: None,
                alternate: None,
            });
        }
        
        Ok(snps)
    }

    fn parse_plink(&self, _input_path: &str) -> Result<Vec<SnpData>> {
        // TODO: Implement PLINK PED/MAP format parsing
        warn!("⚠️ PLINK format parsing not yet implemented");
        Ok(Vec::new())
    }

    fn normalize_chromosome(&self, chr_str: &str) -> String {
        let chr_str = chr_str.trim();
        
        // Handle different chromosome representations
        match chr_str.to_lowercase().as_str() {
            "x" | "chrx" | "23" => "X".to_string(),
            "y" | "chry" | "24" => "Y".to_string(),
            "m" | "mt" | "chrm" | "chrmt" | "25" => "MT".to_string(),
            _ => {
                // Extract numeric part
                let re = Regex::new(r"(\d+)").unwrap();
                if let Some(cap) = re.captures(chr_str) {
                    if let Some(num) = cap.get(1) {
                        let chr_num: u32 = num.as_str().parse().unwrap_or(0);
                        if chr_num >= 1 && chr_num <= 22 {
                            return chr_num.to_string();
                        }
                    }
                }
                "0".to_string() // Invalid chromosome
            }
        }
    }

    fn genotype_to_vcf(&self, genotype: &str) -> (String, Vec<String>) {
        // Convert genotype to VCF REF/ALT format
        let genotype = genotype.trim().to_uppercase();
        
        if genotype.len() == 2 {
            let allele1 = genotype.chars().nth(0).unwrap_or('N');
            let allele2 = genotype.chars().nth(1).unwrap_or('N');
            
            if allele1 == allele2 {
                // Homozygous
                (allele1.to_string(), vec![])
            } else {
                // Heterozygous
                (allele1.to_string(), vec![allele2.to_string()])
            }
        } else if genotype.contains('/') || genotype.contains('|') {
            // Phased or unphased genotype (A/T or A|T)
            let separator = if genotype.contains('/') { '/' } else { '|' };
            let alleles: Vec<&str> = genotype.split(separator).collect();
            
            if alleles.len() == 2 {
                let allele1 = alleles[0].trim();
                let allele2 = alleles[1].trim();
                
                if allele1 == allele2 {
                    (allele1.to_string(), vec![])
                } else {
                    (allele1.to_string(), vec![allele2.to_string()])
                }
            } else {
                ("N".to_string(), vec![])
            }
        } else {
            ("N".to_string(), vec![])
        }
    }

    fn write_vcf_file(&self, output_path: &str, snps: &[SnpData]) -> Result<()> {
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path))?;
        let mut writer = std::io::BufWriter::new(file);
        
        // Write VCF header
        writeln!(writer, "##fileformat=VCFv4.3")?;
        writeln!(writer, "##fileDate={}", chrono::Utc::now().format("%Y%m%d"))?;
        writeln!(writer, "##source=InstantDNA_RawConverter_v2.0.0")?;
        writeln!(writer, "##reference=GRCh37")?;
        writeln!(writer, "##INFO=<ID=RS,Number=1,Type=String,Description=\"dbSNP RS identifier\">")?;
        writeln!(writer, "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">")?;
        writeln!(writer, "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{}", self.sample_name)?;
        
        // Group SNPs by chromosome and sort by position
        let mut chr_snps: HashMap<String, Vec<&SnpData>> = HashMap::new();
        for snp in snps {
            chr_snps.entry(snp.chromosome.clone()).or_default().push(snp);
        }
        
        // Sort chromosomes numerically
        let mut chromosomes: Vec<String> = chr_snps.keys().cloned().collect();
        chromosomes.sort_by(|a, b| {
            match (a.parse::<u32>(), b.parse::<u32>()) {
                (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
                _ => a.cmp(b)
            }
        });
        
        for chromosome in chromosomes {
            if let Some(mut chr_snp_list) = chr_snps.remove(&chromosome) {
                // Sort by position
                chr_snp_list.sort_by_key(|snp| snp.position);
                
                for snp in chr_snp_list {
                    let (ref_allele, alt_alleles) = self.genotype_to_vcf(&snp.genotype);
                    
                    let alt_str = if alt_alleles.is_empty() {
                        ".".to_string()
                    } else {
                        alt_alleles.join(",")
                    };
                    
                    let genotype = if alt_alleles.is_empty() {
                        "0/0".to_string() // Homozygous reference
                    } else {
                        "0/1".to_string() // Heterozygous
                    };
                    
                    let info = if !snp.rsid.is_empty() {
                        format!("RS={}", snp.rsid)
                    } else {
                        ".".to_string()
                    };
                    
                    writeln!(writer, "{}\t{}\t{}\t{}\t{}\t60\tPASS\t{}\tGT\t{}",
                        snp.chromosome,
                        snp.position,
                        if snp.rsid.is_empty() { "." } else { &snp.rsid },
                        ref_allele,
                        alt_str,
                        info,
                        genotype
                    )?;
                }
            }
        }
        
        writer.flush()?;
        Ok(())
    }

    pub fn get_conversion_stats(&self, input_path: &str) -> Result<ConversionStats> {
        let format = match &self.input_format {
            InputFormat::Auto => self.auto_detect_format(input_path)?,
            _ => self.input_format.clone(),
        };
        
        let snps = self.parse_input_file(input_path, &format)?;
        
        let mut chromosome_counts: HashMap<String, usize> = HashMap::new();
        let mut total_snps = 0;
        let mut valid_snps = 0;
        
        for snp in &snps {
            total_snps += 1;
            if snp.chromosome != "0" && !snp.genotype.contains("--") && !snp.genotype.is_empty() {
                valid_snps += 1;
                *chromosome_counts.entry(snp.chromosome.clone()).or_insert(0) += 1;
            }
        }
        
        Ok(ConversionStats {
            detected_format: format,
            total_lines: total_snps,
            valid_snps,
            chromosome_counts,
            sample_name: self.sample_name.clone(),
        })
    }
}

#[derive(Debug)]
pub struct ConversionStats {
    pub detected_format: InputFormat,
    pub total_lines: usize,
    pub valid_snps: usize,
    pub chromosome_counts: HashMap<String, usize>,
    pub sample_name: String,
}

impl ConversionStats {
    pub fn print_summary(&self) {
        println!("🧬 Raw DNA Conversion Statistics");
        println!("===============================");
        println!("📂 Detected Format: {:?}", self.detected_format);
        println!("👤 Sample Name: {}", self.sample_name);
        println!("📊 Total Lines: {}", self.total_lines);
        println!("✅ Valid SNPs: {}", self.valid_snps);
        println!("📈 Success Rate: {:.1}%", (self.valid_snps as f64 / self.total_lines as f64) * 100.0);
        println!();
        println!("🧬 SNPs by Chromosome:");
        
        let mut chromosomes: Vec<_> = self.chromosome_counts.iter().collect();
        chromosomes.sort_by(|a, b| {
            match (a.0.parse::<u32>(), b.0.parse::<u32>()) {
                (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
                _ => a.0.cmp(b.0)
            }
        });
        
        for (chr, count) in chromosomes {
            println!("   Chr {}: {} SNPs", chr, count);
        }
    }
}
