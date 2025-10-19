use std::io::{self, Write};
use std::collections::HashMap;
use anyhow::Result;
use log::info;

#[derive(Debug, Clone)]
pub struct ManualSnpEntry {
    pub rsid: String,
    pub chromosome: String,
    pub position: u64,
    pub genotype: String,
    pub confidence: f64,
    pub method: String,
}

#[derive(Debug)]
pub struct DiyDnaExtractor {
    pub sample_name: String,
    pub extraction_method: String,
    pub manual_entries: Vec<ManualSnpEntry>,
}

impl DiyDnaExtractor {
    pub fn new(sample_name: String) -> Self {
        Self {
            sample_name,
            extraction_method: "home_extraction".to_string(),
            manual_entries: Vec::new(),
        }
    }

    pub fn start_manual_entry_session(&mut self) -> Result<()> {
        println!("🧬 DIY DNA ANALYSIS - MANUAL SNP ENTRY");
        println!("=====================================");
        println!("👤 Sample: {}", self.sample_name);
        println!("🔬 Ready for manual SNP genotyping results");
        println!();
        
        self.show_diy_extraction_guide();
        self.show_manual_genotyping_guide();
        self.interactive_snp_entry()?;
        
        Ok(())
    }

    fn show_diy_extraction_guide(&self) {
        println!("🧪 DIY DNA EXTRACTION GUIDE");
        println!("============================");
        println!();
        println!("📋 Materials Needed:");
        println!("   • Q-tips (cotton swabs)");
        println!("   • Salt water (1 tsp salt + 1/4 cup warm water)");
        println!("   • Clear dish soap (1-2 drops)");
        println!("   • Rubbing alcohol (70% or higher)");
        println!("   • Small clear containers");
        println!("   • Magnifying glass or microscope (optional)");
        println!();
        println!("🔬 Extraction Steps:");
        println!("   1. Swab inside of cheek with Q-tip for 30 seconds");
        println!("   2. Swirl Q-tip in salt water for 30 seconds");
        println!("   3. Add 1-2 drops dish soap, mix gently");
        println!("   4. Slowly add cold rubbing alcohol down side of container");
        println!("   5. DNA will precipitate as white stringy material");
        println!("   6. Collect DNA strands for analysis");
        println!();
        println!("⚠️  Note: Home extraction yields crude DNA suitable for");
        println!("    educational purposes. Professional sequencing requires");
        println!("    lab-grade extraction and equipment.");
        println!();
    }

    fn show_manual_genotyping_guide(&self) {
        println!("🔍 MANUAL GENOTYPING GUIDE");
        println!("==========================");
        println!();
        println!("🧬 Common SNPs You Can Estimate:");
        println!();
        println!("📊 Physical Traits (easier to observe):");
        println!("   • rs12913832 (Eye color - brown/blue)");
        println!("   • rs1805007 (Hair color - red hair variant)");
        println!("   • rs4988235 (Lactose tolerance)");
        println!("   • rs17822931 (Earwax type - wet/dry)");
        println!("   • rs6152 (Hair texture - straight/curly)");
        println!();
        println!("🌍 Ancestry Markers (based on family history):");
        println!("   • rs3827760 (European ancestry marker)");
        println!("   • rs2814778 (African ancestry marker)");
        println!("   • rs671 (East Asian ancestry marker)");
        println!();
        println!("💡 How to Estimate:");
        println!("   1. Use family history and physical traits");
        println!("   2. Research SNP associations online");
        println!("   3. Make educated guesses based on phenotype");
        println!("   4. Enter results with confidence levels");
        println!();
        println!("🔬 For Advanced Users:");
        println!("   • Use restriction enzymes (if available)");
        println!("   • PCR amplification (with equipment)");
        println!("   • Gel electrophoresis patterns");
        println!("   • Microscopic DNA analysis");
        println!();
    }

    fn interactive_snp_entry(&mut self) -> Result<()> {
        println!("📝 MANUAL SNP ENTRY SESSION");
        println!("===========================");
        println!("Enter SNP data one by one. Type 'done' when finished.");
        println!("Type 'help' for format examples.");
        println!();

        loop {
            print!("Enter SNP data (or 'done'/'help'): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            if input.to_lowercase() == "done" {
                break;
            }
            
            if input.to_lowercase() == "help" {
                self.show_entry_format_help();
                continue;
            }

            if input.to_lowercase() == "preset" {
                self.show_preset_snps();
                continue;
            }

            match self.parse_snp_entry(input) {
                Ok(snp) => {
                    println!("✅ Added: {} -> {} ({}% confidence)", 
                        snp.rsid, snp.genotype, (snp.confidence * 100.0) as u8);
                    self.manual_entries.push(snp);
                }
                Err(e) => {
                    println!("❌ Error parsing SNP: {}. Type 'help' for format.", e);
                }
            }
        }

        self.show_entry_summary();
        Ok(())
    }

    fn show_entry_format_help(&self) {
        println!();
        println!("📋 SNP ENTRY FORMAT EXAMPLES:");
        println!("===============================");
        println!();
        println!("Format: rsid,chromosome,position,genotype,confidence,method");
        println!();
        println!("Examples:");
        println!("rs12913832,15,28365618,GG,0.9,visual_trait");
        println!("rs1805007,16,89986091,CC,0.7,family_history");
        println!("rs4988235,2,136608646,GG,0.8,phenotype");
        println!("rs671,12,112241766,GG,0.6,ancestry");
        println!();
        println!("💡 Shortcuts:");
        println!("• Type 'preset' to see common SNPs with known positions");
        println!("• Confidence: 0.1 (low) to 1.0 (high certainty)");
        println!("• Method: visual_trait, family_history, phenotype, ancestry, lab_test");
        println!();
    }

    fn show_preset_snps(&self) {
        println!();
        println!("🧬 PRESET COMMON SNPs");
        println!("=====================");
        println!();
        println!("Copy and paste these, modifying genotype and confidence:");
        println!();
        println!("👁️  Eye Color (rs12913832):");
        println!("rs12913832,15,28365618,GG,0.9,visual_trait  # Brown eyes");
        println!("rs12913832,15,28365618,AA,0.9,visual_trait  # Blue eyes");
        println!();
        println!("🦰 Red Hair (rs1805007):");
        println!("rs1805007,16,89986091,CC,0.8,visual_trait   # No red hair");
        println!("rs1805007,16,89986091,CT,0.8,visual_trait   # Carrier");
        println!("rs1805007,16,89986091,TT,0.9,visual_trait   # Red hair");
        println!();
        println!("🥛 Lactose Tolerance (rs4988235):");
        println!("rs4988235,2,136608646,GG,0.7,phenotype      # Lactose tolerant");
        println!("rs4988235,2,136608646,AA,0.7,phenotype      # Lactose intolerant");
        println!();
        println!("👂 Earwax Type (rs17822931):");
        println!("rs17822931,16,48258198,GG,0.9,phenotype     # Wet earwax");
        println!("rs17822931,16,48258198,AA,0.9,phenotype     # Dry earwax");
        println!();
        println!("🌍 Ancestry Markers:");
        println!("rs3827760,7,2723432,GG,0.6,ancestry         # European marker");
        println!("rs2814778,1,202136319,CC,0.6,ancestry       # African marker");
        println!("rs671,12,112241766,GG,0.6,ancestry          # East Asian marker");
        println!();
    }

    fn parse_snp_entry(&self, input: &str) -> Result<ManualSnpEntry> {
        let parts: Vec<&str> = input.split(',').collect();
        
        if parts.len() != 6 {
            return Err(anyhow::anyhow!("Expected 6 comma-separated values"));
        }

        let rsid = parts[0].trim().to_string();
        let chromosome = parts[1].trim().to_string();
        let position: u64 = parts[2].trim().parse()
            .map_err(|_| anyhow::anyhow!("Invalid position number"))?;
        let genotype = parts[3].trim().to_uppercase();
        let confidence: f64 = parts[4].trim().parse()
            .map_err(|_| anyhow::anyhow!("Invalid confidence value (0.0-1.0)"))?;
        let method = parts[5].trim().to_string();

        // Validate genotype format
        if genotype.len() != 2 || !genotype.chars().all(|c| "ATCG".contains(c)) {
            return Err(anyhow::anyhow!("Genotype must be 2 letters (A,T,C,G only)"));
        }

        // Validate confidence range
        if !(0.0..=1.0).contains(&confidence) {
            return Err(anyhow::anyhow!("Confidence must be between 0.0 and 1.0"));
        }

        Ok(ManualSnpEntry {
            rsid,
            chromosome,
            position,
            genotype,
            confidence,
            method,
        })
    }

    fn show_entry_summary(&self) {
        println!();
        println!("📊 MANUAL ENTRY SUMMARY");
        println!("========================");
        println!("👤 Sample: {}", self.sample_name);
        println!("🧬 Total SNPs entered: {}", self.manual_entries.len());
        println!();

        if !self.manual_entries.is_empty() {
            println!("📋 Entered SNPs:");
            for (i, snp) in self.manual_entries.iter().enumerate() {
                println!("  {}. {} (chr{}) -> {} [{}% conf, {}]", 
                    i + 1, snp.rsid, snp.chromosome, snp.genotype,
                    (snp.confidence * 100.0) as u8, snp.method);
            }
            println!();

            // Calculate average confidence
            let avg_confidence = self.manual_entries.iter()
                .map(|s| s.confidence)
                .sum::<f64>() / self.manual_entries.len() as f64;
            
            println!("📈 Average confidence: {:.1}%", avg_confidence * 100.0);
            
            // Group by method
            let mut method_counts: HashMap<String, usize> = HashMap::new();
            for snp in &self.manual_entries {
                *method_counts.entry(snp.method.clone()).or_insert(0) += 1;
            }
            
            println!("🔬 Methods used:");
            for (method, count) in method_counts {
                println!("   • {}: {} SNPs", method, count);
            }
        }
    }

    pub fn export_to_vcf(&self, output_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::BufWriter;

        if self.manual_entries.is_empty() {
            return Err(anyhow::anyhow!("No SNP entries to export"));
        }

        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        // Write VCF header
        writeln!(writer, "##fileformat=VCFv4.3")?;
        writeln!(writer, "##fileDate={}", chrono::Utc::now().format("%Y%m%d"))?;
        writeln!(writer, "##source=InstantDNA_DIY_Manual_Entry_v2.0.0")?;
        writeln!(writer, "##reference=GRCh37")?;
        writeln!(writer, "##INFO=<ID=RS,Number=1,Type=String,Description=\"dbSNP RS identifier\">")?;
        writeln!(writer, "##INFO=<ID=CONF,Number=1,Type=Float,Description=\"Manual entry confidence\">")?;
        writeln!(writer, "##INFO=<ID=METHOD,Number=1,Type=String,Description=\"Manual genotyping method\">")?;
        writeln!(writer, "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">")?;
        writeln!(writer, "##NOTE=DIY home extraction and manual genotyping")?;
        writeln!(writer, "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{}", self.sample_name)?;

        // Sort entries by chromosome and position
        let mut sorted_entries = self.manual_entries.clone();
        sorted_entries.sort_by(|a, b| {
            match a.chromosome.cmp(&b.chromosome) {
                std::cmp::Ordering::Equal => a.position.cmp(&b.position),
                other => other,
            }
        });

        for snp in sorted_entries {
            let (ref_allele, alt_allele) = self.split_genotype(&snp.genotype);
            let alt_str = if alt_allele == ref_allele { "." } else { &alt_allele };
            let genotype = if alt_allele == ref_allele { "0/0" } else { "0/1" };
            let quality = (snp.confidence * 100.0) as u8;
            
            let info = format!("RS={};CONF={:.2};METHOD={}", 
                snp.rsid, snp.confidence, snp.method);

            writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\tPASS\t{}\tGT\t{}",
                snp.chromosome, snp.position, snp.rsid, 
                ref_allele, alt_str, quality, info, genotype)?;
        }

        info!("✅ Exported {} manual SNP entries to VCF", self.manual_entries.len());
        Ok(())
    }

    fn split_genotype(&self, genotype: &str) -> (String, String) {
        if genotype.len() == 2 {
            let allele1 = genotype.chars().nth(0).unwrap().to_string();
            let allele2 = genotype.chars().nth(1).unwrap().to_string();
            (allele1, allele2)
        } else {
            ("N".to_string(), "N".to_string())
        }
    }

    pub fn load_diy_kit_markers(&mut self) {
        println!("📦 LOADING DIY DNA ANALYSIS KIT MARKERS");
        println!("========================================");
        println!();
        println!("Loading common SNPs for home analysis...");
        
        // Pre-populate with common, easily observable SNPs
        let diy_snps = vec![
            ("rs12913832", "15", 28365618, "Eye color (HERC2)"),
            ("rs1805007", "16", 89986091, "Red hair (MC1R)"),
            ("rs4988235", "2", 136608646, "Lactose tolerance"),
            ("rs17822931", "16", 48258198, "Earwax type (ABCC11)"),
            ("rs6152", "12", 56372758, "Hair texture"),
            ("rs3827760", "7", 2723432, "European ancestry"),
            ("rs2814778", "1", 202136319, "African ancestry"),
            ("rs671", "12", 112241766, "Asian ancestry"),
            ("rs1426654", "15", 48426484, "Skin pigmentation"),
            ("rs16891982", "5", 33951693, "Eye color (SLC45A2)"),
        ];

        println!("📋 Available DIY markers:");
        for (rsid, chr, pos, desc) in &diy_snps {
            println!("   • {} (chr{}:{}) - {}", rsid, chr, pos, desc);
        }
        
        println!();
        println!("💡 Use these RSIDs when entering your manual genotype data!");
        println!("   Each marker above can be estimated from physical traits");
        println!("   or family history information.");
    }
}
