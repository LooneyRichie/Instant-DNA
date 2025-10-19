use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::{Result, Context};
use flate2::read::GzDecoder;

#[derive(Debug, Clone)]
pub struct SNPVariant {
    pub chromosome: String,
    pub position: u64,
    pub id: String,
    pub reference: String,
    pub alternative: String,
    pub quality: f64,
    pub samples: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Population {
    pub name: String,
    pub samples: Vec<String>,
    pub ancestry: String,
    pub region: String,
}

pub struct VCFProcessor {
    pub variants: Vec<SNPVariant>,
    pub samples: Vec<String>,
    pub populations: HashMap<String, Population>,
}

impl VCFProcessor {
    pub fn new() -> Self {
        Self {
            variants: Vec::new(),
            samples: Vec::new(),
            populations: HashMap::new(),
        }
    }

    /// Parse VCF file (handles both .vcf and .vcf.gz)
    pub fn parse_vcf(&mut self, vcf_path: &Path) -> Result<()> {
        println!("🧬 Parsing VCF file: {}", vcf_path.display());
        
        let file = File::open(vcf_path)
            .with_context(|| format!("Failed to open VCF file: {}", vcf_path.display()))?;

        let reader: Box<dyn BufRead> = if vcf_path.extension().unwrap_or_default() == "gz" {
            Box::new(BufReader::new(GzDecoder::new(file)))
        } else {
            Box::new(BufReader::new(file))
        };

        let mut line_count = 0;
        let mut variant_count = 0;
        let mut header_parsed = false;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            // Skip comments and meta-information
            if line.starts_with("##") {
                continue;
            }

            // Parse header line
            if line.starts_with("#CHROM") && !header_parsed {
                self.parse_header(&line)?;
                header_parsed = true;
                continue;
            }

            // Parse variant data
            if !line.starts_with("#") && header_parsed {
                if let Ok(variant) = self.parse_variant_line(&line) {
                    self.variants.push(variant);
                    variant_count += 1;
                    
                    // Progress update every 10,000 variants
                    if variant_count % 10000 == 0 {
                        println!("   📊 Processed {} variants...", variant_count);
                    }
                }
            }
        }

        println!("✅ VCF parsing complete:");
        println!("   📁 Total lines: {}", line_count);
        println!("   🧬 Variants loaded: {}", variant_count);
        println!("   👥 Samples: {}", self.samples.len());

        Ok(())
    }

    /// Parse population panel file (e.g., from 1000 Genomes)
    pub fn parse_population_panel(&mut self, panel_path: &Path) -> Result<()> {
        println!("🌍 Loading population data: {}", panel_path.display());
        
        let file = File::open(panel_path)?;
        let reader = BufReader::new(file);

        let mut population_count = HashMap::new();
        
        for line in reader.lines() {
            let line = line?;
            if line.starts_with("sample") {
                continue; // Skip header
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                let sample = parts[0].to_string();
                let pop_code = parts[1].to_string();
                let super_pop = parts[2].to_string();
                let gender = parts[3].to_string();

                // Create population if it doesn't exist
                if !self.populations.contains_key(&pop_code) {
                    let pop_name = self.get_population_name(&pop_code);
                    self.populations.insert(pop_code.clone(), Population {
                        name: pop_name,
                        samples: Vec::new(),
                        ancestry: super_pop.clone(),
                        region: self.get_population_region(&super_pop),
                    });
                }

                // Add sample to population
                if let Some(pop) = self.populations.get_mut(&pop_code) {
                    pop.samples.push(sample);
                }

                *population_count.entry(pop_code.clone()).or_insert(0) += 1;
            }
        }

        println!("✅ Population data loaded:");
        for (pop_code, count) in population_count {
            if let Some(pop) = self.populations.get(&pop_code) {
                println!("   🌍 {} ({}): {} samples", pop.name, pop.ancestry, count);
            }
        }

        Ok(())
    }

    /// Calculate allele frequency for a population
    pub fn calculate_allele_frequency(&self, variant_index: usize, population: &str) -> Option<f64> {
        if variant_index >= self.variants.len() {
            return None;
        }

        let variant = &self.variants[variant_index];
        let pop = self.populations.get(population)?;
        
        let mut alt_alleles = 0;
        let mut total_alleles = 0;

        // Count alleles in population samples
        for sample in &pop.samples {
            if let Some(sample_index) = self.samples.iter().position(|s| s == sample) {
                if sample_index < variant.samples.len() {
                    let genotype = &variant.samples[sample_index];
                    // Parse genotype (e.g., "0|1", "1|1", "0|0")
                    if let Some((allele1, allele2)) = self.parse_genotype(genotype) {
                        total_alleles += 2;
                        if allele1 == 1 { alt_alleles += 1; }
                        if allele2 == 1 { alt_alleles += 1; }
                    }
                }
            }
        }

        if total_alleles > 0 {
            Some(alt_alleles as f64 / total_alleles as f64)
        } else {
            None
        }
    }

    /// Compare genetic similarity between two samples
    pub fn calculate_genetic_similarity(&self, sample1: &str, sample2: &str) -> Result<f64> {
        let sample1_idx = self.samples.iter().position(|s| s == sample1)
            .context("Sample 1 not found")?;
        let sample2_idx = self.samples.iter().position(|s| s == sample2)
            .context("Sample 2 not found")?;

        let mut matches = 0;
        let mut total_compared = 0;

        for variant in &self.variants {
            if sample1_idx < variant.samples.len() && sample2_idx < variant.samples.len() {
                let gt1 = &variant.samples[sample1_idx];
                let gt2 = &variant.samples[sample2_idx];
                
                // Compare genotypes
                if gt1 == gt2 {
                    matches += 1;
                }
                total_compared += 1;
            }
        }

        if total_compared > 0 {
            Ok(matches as f64 / total_compared as f64)
        } else {
            Ok(0.0)
        }
    }

    /// Get ancestry estimation for a sample
    pub fn estimate_ancestry(&self, sample: &str) -> Result<HashMap<String, f64>> {
        let mut ancestry_scores = HashMap::new();
        
        // Get unique super populations
        let super_pops: std::collections::HashSet<String> = self.populations
            .values()
            .map(|p| p.ancestry.clone())
            .collect();

        for super_pop in super_pops {
            let mut total_similarity = 0.0;
            let mut population_count = 0;

            // Calculate similarity to all populations in this super population
            for (pop_code, population) in &self.populations {
                if population.ancestry == super_pop {
                    // Compare with first few samples from this population
                    for pop_sample in population.samples.iter().take(10) {
                        if let Ok(similarity) = self.calculate_genetic_similarity(sample, pop_sample) {
                            total_similarity += similarity;
                            population_count += 1;
                        }
                    }
                }
            }

            if population_count > 0 {
                ancestry_scores.insert(super_pop, total_similarity / population_count as f64);
            }
        }

        Ok(ancestry_scores)
    }

    // Helper functions
    fn parse_header(&mut self, header_line: &str) -> Result<()> {
        let fields: Vec<&str> = header_line.split('\t').collect();
        
        // Standard VCF fields: CHROM, POS, ID, REF, ALT, QUAL, FILTER, INFO, FORMAT
        if fields.len() > 9 {
            // Sample names start from index 9
            for i in 9..fields.len() {
                self.samples.push(fields[i].to_string());
            }
        }

        Ok(())
    }

    fn parse_variant_line(&self, line: &str) -> Result<SNPVariant> {
        let fields: Vec<&str> = line.split('\t').collect();
        
        if fields.len() < 9 {
            return Err(anyhow::anyhow!("Invalid VCF line format"));
        }

        let quality = fields[5].parse::<f64>().unwrap_or(0.0);
        
        // Parse sample genotypes
        let mut sample_genotypes = Vec::new();
        for i in 9..fields.len() {
            sample_genotypes.push(fields[i].to_string());
        }

        Ok(SNPVariant {
            chromosome: fields[0].to_string(),
            position: fields[1].parse::<u64>()?,
            id: fields[2].to_string(),
            reference: fields[3].to_string(),
            alternative: fields[4].to_string(),
            quality,
            samples: sample_genotypes,
        })
    }

    fn parse_genotype(&self, genotype: &str) -> Option<(u8, u8)> {
        // Handle different genotype formats: "0|1", "0/1", "1|1", etc.
        if genotype.contains('|') {
            let parts: Vec<&str> = genotype.split('|').collect();
            if parts.len() == 2 {
                let allele1 = parts[0].parse::<u8>().ok()?;
                let allele2 = parts[1].parse::<u8>().ok()?;
                return Some((allele1, allele2));
            }
        } else if genotype.contains('/') {
            let parts: Vec<&str> = genotype.split('/').collect();
            if parts.len() == 2 {
                let allele1 = parts[0].parse::<u8>().ok()?;
                let allele2 = parts[1].parse::<u8>().ok()?;
                return Some((allele1, allele2));
            }
        }
        None
    }

    fn get_population_name(&self, pop_code: &str) -> String {
        match pop_code {
            "CHB" => "Han Chinese in Beijing".to_string(),
            "JPT" => "Japanese in Tokyo".to_string(),
            "CHS" => "Southern Han Chinese".to_string(),
            "CDX" => "Chinese Dai in Xishuangbanna".to_string(),
            "KHV" => "Kinh in Ho Chi Minh City".to_string(),
            "CEU" => "Utah residents with European ancestry".to_string(),
            "TSI" => "Toscani in Italia".to_string(),
            "FIN" => "Finnish in Finland".to_string(),
            "GBR" => "British in England and Scotland".to_string(),
            "IBS" => "Iberian populations in Spain".to_string(),
            "YRI" => "Yoruba in Ibadan, Nigeria".to_string(),
            "LWK" => "Luhya in Webuye, Kenya".to_string(),
            "GWD" => "Gambian in Western Division".to_string(),
            "MSL" => "Mende in Sierra Leone".to_string(),
            "ESN" => "Esan in Nigeria".to_string(),
            "ASW" => "African Ancestry in Southwest US".to_string(),
            "ACB" => "African Caribbean in Barbados".to_string(),
            "MXL" => "Mexican Ancestry in Los Angeles".to_string(),
            "PUR" => "Puerto Rican in Puerto Rico".to_string(),
            "CLM" => "Colombian in Medellin".to_string(),
            "PEL" => "Peruvian in Lima".to_string(),
            _ => pop_code.to_string(),
        }
    }

    fn get_population_region(&self, super_pop: &str) -> String {
        match super_pop {
            "EAS" => "East Asia".to_string(),
            "EUR" => "Europe".to_string(),
            "AFR" => "Africa".to_string(),
            "AMR" => "Americas".to_string(),
            "SAS" => "South Asia".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Print statistics about loaded data
    pub fn print_statistics(&self) {
        println!("\n📊 VCF Dataset Statistics:");
        println!("   🧬 Total variants: {}", self.variants.len());
        println!("   👥 Total samples: {}", self.samples.len());
        println!("   🌍 Populations: {}", self.populations.len());
        
        // Chromosome distribution
        let mut chr_counts = HashMap::new();
        for variant in &self.variants {
            *chr_counts.entry(variant.chromosome.clone()).or_insert(0) += 1;
        }
        
        println!("\n🧬 Variants by chromosome:");
        for (chr, count) in chr_counts {
            println!("   Chr {}: {} variants", chr, count);
        }

        // Population distribution
        println!("\n🌍 Population distribution:");
        for (code, pop) in &self.populations {
            println!("   {} ({}): {} samples", code, pop.name, pop.samples.len());
        }
    }
}
