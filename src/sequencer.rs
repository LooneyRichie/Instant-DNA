use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::binary_optimizer::BinaryOptimizer;

/// High-speed DNA sequencer with real-time capabilities
pub struct Sequencer {
    quality_threshold: u8,
    max_read_length: usize,
    realtime_enabled: bool,
}

impl Sequencer {
    pub fn new(quality_threshold: u8, max_read_length: usize) -> Result<Self> {
        Ok(Self {
            quality_threshold,
            max_read_length,
            realtime_enabled: false,
        })
    }
    
    pub fn enable_realtime(&mut self) {
        self.realtime_enabled = true;
    }
    
    /// Process input file and return sequence data
    pub fn process_file(&self, input_path: &str, optimizer: &BinaryOptimizer) -> Result<Vec<SequenceData>> {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut sequences = Vec::new();
        
        // Determine file format from extension or content
        if input_path.ends_with(".fasta") || input_path.ends_with(".fa") {
            sequences = self.parse_fasta(reader, optimizer)?;
        } else if input_path.ends_with(".fastq") || input_path.ends_with(".fq") {
            sequences = self.parse_fastq(reader, optimizer)?;
        } else {
            // Try to auto-detect format
            sequences = self.parse_auto_detect(input_path, optimizer)?;
        }
        
        // Filter by quality threshold
        let filtered: Vec<_> = sequences
            .into_iter()
            .filter(|seq| seq.avg_quality() >= self.quality_threshold as f64)
            .collect();
        
        Ok(filtered)
    }
    
    fn parse_fasta(&self, reader: BufReader<File>, optimizer: &BinaryOptimizer) -> Result<Vec<SequenceData>> {
        let mut sequences = Vec::new();
        let mut current_header = String::new();
        let mut current_sequence = String::new();
        
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('>') {
                // Save previous sequence if exists
                if !current_sequence.is_empty() {
                    let seq_data = self.create_sequence_data(
                        &current_header,
                        &current_sequence,
                        None,
                        optimizer,
                    )?;
                    sequences.push(seq_data);
                }
                // Start new sequence
                current_header = line[1..].to_string();
                current_sequence.clear();
            } else {
                current_sequence.push_str(&line.trim().to_uppercase());
            }
        }
        
        // Don't forget the last sequence
        if !current_sequence.is_empty() {
            let seq_data = self.create_sequence_data(
                &current_header,
                &current_sequence,
                None,
                optimizer,
            )?;
            sequences.push(seq_data);
        }
        
        Ok(sequences)
    }
    
    fn parse_fastq(&self, reader: BufReader<File>, optimizer: &BinaryOptimizer) -> Result<Vec<SequenceData>> {
        let mut sequences = Vec::new();
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;
        
        // FASTQ format: 4 lines per read
        for chunk in lines.chunks_exact(4) {
            let header = &chunk[0][1..]; // Remove @ symbol
            let sequence = &chunk[1].to_uppercase();
            let quality = &chunk[3];
            
            if sequence.len() <= self.max_read_length {
                let quality_scores: Vec<u8> = quality
                    .bytes()
                    .map(|q| q.saturating_sub(33)) // Phred+33 encoding
                    .collect();
                
                let seq_data = self.create_sequence_data(
                    header,
                    sequence,
                    Some(quality_scores),
                    optimizer,
                )?;
                
                sequences.push(seq_data);
            }
        }
        
        Ok(sequences)
    }
    
    fn parse_auto_detect(&self, input_path: &str, optimizer: &BinaryOptimizer) -> Result<Vec<SequenceData>> {
        let file = File::open(input_path)?;
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;
        
        // Reset reader
        drop(reader);
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        
        if first_line.starts_with('>') {
            self.parse_fasta(reader, optimizer)
        } else if first_line.starts_with('@') {
            self.parse_fastq(reader, optimizer)
        } else {
            // Treat as raw DNA sequence
            self.parse_raw_dna(reader, optimizer)
        }
    }
    
    fn parse_raw_dna(&self, reader: BufReader<File>, optimizer: &BinaryOptimizer) -> Result<Vec<SequenceData>> {
        let mut sequences = Vec::new();
        let mut seq_counter = 1;
        
        for line in reader.lines() {
            let line = line?;
            let sequence = line.trim().to_uppercase();
            
            // Validate DNA sequence
            if sequence.chars().all(|c| matches!(c, 'A' | 'T' | 'G' | 'C' | 'N')) {
                let header = format!("sequence_{}", seq_counter);
                let seq_data = self.create_sequence_data(&header, &sequence, None, optimizer)?;
                sequences.push(seq_data);
                seq_counter += 1;
            }
        }
        
        Ok(sequences)
    }
    
    fn create_sequence_data(
        &self,
        header: &str,
        sequence: &str,
        quality_scores: Option<Vec<u8>>,
        optimizer: &BinaryOptimizer,
    ) -> Result<SequenceData> {
        // Convert to binary representation for processing
        let binary_seq = sequence
            .chars()
            .map(|c| match c {
                'A' => 0,
                'T' => 1,
                'G' => 2,
                'C' => 3,
                _ => 0, // Default unknown to A
            })
            .collect();
        
        let quality = quality_scores.unwrap_or_else(|| vec![40; sequence.len()]); // Default high quality
        
        Ok(SequenceData {
            id: header.to_string(),
            sequence: sequence.to_string(),
            binary_sequence: binary_seq,
            quality_scores: quality,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Represents a sequenced DNA fragment with quality information
#[derive(Debug, Clone)]
pub struct SequenceData {
    pub id: String,
    pub sequence: String,
    pub binary_sequence: Vec<u8>,
    pub quality_scores: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SequenceData {
    pub fn avg_quality(&self) -> f64 {
        if self.quality_scores.is_empty() {
            return 0.0;
        }
        
        let sum: u32 = self.quality_scores.iter().map(|&q| q as u32).sum();
        sum as f64 / self.quality_scores.len() as f64
    }
    
    pub fn gc_content(&self) -> f64 {
        let gc_count = self.sequence.chars()
            .filter(|&c| c == 'G' || c == 'C')
            .count();
        
        gc_count as f64 / self.sequence.len() as f64
    }
    
    pub fn length(&self) -> usize {
        self.sequence.len()
    }
}

/// Quality score utilities
pub struct QualityScore;

impl QualityScore {
    pub fn phred_to_probability(phred: u8) -> f64 {
        10.0_f64.powf(-(phred as f64) / 10.0)
    }
    
    pub fn probability_to_phred(prob: f64) -> u8 {
        (-10.0 * prob.log10()).round() as u8
    }
}
