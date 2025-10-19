#!/bin/bash

# 🧬 1000 Genomes Data Download Script
# This script downloads real SNP data from the 1000 Genomes Project

echo "🧬 1000 Genomes Project Data Download"
echo "====================================="
echo ""

# Create data directory
mkdir -p real_snp_data/1000genomes
cd real_snp_data/1000genomes

echo "📂 Creating data directory: real_snp_data/1000genomes"
echo ""

# Download population panel (small file, fast download)
echo "🌍 Step 1: Downloading population panel..."
echo "File: integrated_call_samples_v3.20130502.ALL.panel"
wget -q --show-progress ftp://ftp.1000genomes.ebi.ac.uk/vol1/ftp/release/20130502/integrated_call_samples_v3.20130502.ALL.panel

if [ $? -eq 0 ]; then
    echo "✅ Population panel downloaded successfully"
    echo "   👥 Contains sample population assignments"
    echo ""
else
    echo "❌ Failed to download population panel"
    exit 1
fi

# Download chromosome 22 VCF (smallest chromosome, ~200MB compressed)
echo "🧬 Step 2: Downloading chromosome 22 SNP data..."
echo "File: ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz"
echo "Size: ~200MB (this may take a few minutes)"
echo ""

wget -q --show-progress ftp://ftp.1000genomes.ebi.ac.uk/vol1/ftp/release/20130502/ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Chromosome 22 VCF downloaded successfully"
    echo ""
else
    echo "❌ Failed to download chromosome 22 VCF"
    exit 1
fi

# Show file information
echo "📊 Downloaded Files:"
echo "==================="
ls -lh *.panel *.vcf.gz 2>/dev/null | while read line; do
    echo "   $line"
done
echo ""

# Quick file analysis
echo "🔍 Quick File Analysis:"
echo "======================"

# Population panel analysis
if [ -f "integrated_call_samples_v3.20130502.ALL.panel" ]; then
    SAMPLE_COUNT=$(wc -l < integrated_call_samples_v3.20130502.ALL.panel)
    echo "   👥 Population Panel: $((SAMPLE_COUNT-1)) samples"
    
    echo "   🌍 Population distribution:"
    tail -n +2 integrated_call_samples_v3.20130502.ALL.panel | cut -f2 | sort | uniq -c | sort -nr | head -10 | while read count pop; do
        echo "      $pop: $count samples"
    done
    echo ""
fi

# VCF file analysis
if [ -f "ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz" ]; then
    echo "   🧬 VCF File Analysis:"
    
    # Count variants (this may take a moment)
    VARIANT_COUNT=$(zcat ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz | grep -v "^#" | wc -l)
    echo "      Variants on chromosome 22: $VARIANT_COUNT"
    
    # Show sample count from header
    SAMPLE_COUNT_VCF=$(zcat ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz | head -1000 | grep "^#CHROM" | tr '\t' '\n' | tail -n +10 | wc -l)
    echo "      Samples in VCF: $SAMPLE_COUNT_VCF"
    echo ""
fi

echo "🎯 Ready for Analysis!"
echo "====================="
echo ""
echo "📝 To analyze this data with instant_dna:"
echo ""
echo "1. Process VCF file:"
echo "   cargo run --release -- vcf --input real_snp_data/1000genomes/ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz --panel real_snp_data/1000genomes/integrated_call_samples_v3.20130502.ALL.panel --frequencies"
echo ""
echo "2. Ancestry analysis (example with first sample):"
echo "   cargo run --release -- ancestry --vcf real_snp_data/1000genomes/ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz --panel real_snp_data/1000genomes/integrated_call_samples_v3.20130502.ALL.panel --sample HG00096 --detailed"
echo ""
echo "🌍 Available populations include:"
echo "   • CHB: Han Chinese in Beijing"
echo "   • CEU: Utah residents with European ancestry" 
echo "   • YRI: Yoruba in Ibadan, Nigeria"
echo "   • JPT: Japanese in Tokyo"
echo "   • And 22 more populations!"
echo ""
echo "⚠️  Important Notes:"
echo "   • This is chromosome 22 only (smallest chromosome)"
echo "   • For full genome: download other chromosomes (1-21, X, Y)"
echo "   • Each full chromosome: 1-10 GB compressed"
echo "   • Complete dataset: ~1 TB total"
echo ""
echo "🔬 Academic Use Only:"
echo "   • Follow 1000 Genomes data usage policies"
echo "   • Do not attempt to re-identify individuals"
echo "   • Educational and research purposes only"
echo ""

# Create a summary file
cat > DATASET_INFO.txt << EOF
🧬 1000 Genomes Project Data - Chromosome 22
===========================================

Downloaded: $(date)

Files:
• integrated_call_samples_v3.20130502.ALL.panel
  - Population assignments for all samples
  - 2,504 individuals from 26 populations
  
• ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz
  - SNP variants for chromosome 22
  - ~1.1 million variants
  - 2,504 samples

Usage Examples:
1. Process VCF:
   cargo run --release -- vcf -i ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz -p integrated_call_samples_v3.20130502.ALL.panel --frequencies

2. Ancestry Analysis:
   cargo run --release -- ancestry --vcf ALL.chr22.phase3_shapeit2_mvncall_integrated_v5a.20130502.genotypes.vcf.gz --panel integrated_call_samples_v3.20130502.ALL.panel --sample HG00096

Populations Available:
• AFR: African (661 individuals)
• AMR: Ad Mixed American (347 individuals)  
• EAS: East Asian (504 individuals)
• EUR: European (503 individuals)
• SAS: South Asian (489 individuals)

Data Source: International Genome Sample Resource (IGSR)
URL: https://www.internationalgenome.org/
License: Open access for research and educational use
EOF

echo "📋 Dataset information saved to: DATASET_INFO.txt"
echo ""
echo "🚀 Download complete! Ready to analyze real genomic data."
