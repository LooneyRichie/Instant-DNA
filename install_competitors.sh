#!/bin/bash

# DNA Analysis Tools Installation Script for Benchmarking
# Installs popular bioinformatics tools for performance comparison

set -e

echo "🧬 Installing DNA Analysis Tools for Benchmarking"
echo "================================================"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo "❌ This script should not be run as root"
   exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install via conda if available
install_via_conda() {
    if command_exists conda; then
        echo "📦 Installing $1 via conda..."
        conda install -c bioconda "$1" -y
        return 0
    fi
    return 1
}

# Function to install via apt (Ubuntu/Debian)
install_via_apt() {
    if command_exists apt-get; then
        echo "📦 Installing $1 via apt..."
        sudo apt-get update -qq
        sudo apt-get install -y "$1"
        return 0
    fi
    return 1
}

# Function to install via yum (CentOS/RHEL)
install_via_yum() {
    if command_exists yum; then
        echo "📦 Installing $1 via yum..."
        sudo yum install -y "$1"
        return 0
    fi
    return 1
}

# Function to install tool with multiple methods
install_tool() {
    local tool="$1"
    local conda_name="$2"
    local apt_name="$3"
    
    if command_exists "$tool"; then
        echo "✅ $tool is already installed"
        return 0
    fi
    
    echo "🔧 Installing $tool..."
    
    if install_via_conda "$conda_name"; then
        return 0
    elif install_via_apt "$apt_name"; then
        return 0
    elif install_via_yum "$apt_name"; then
        return 0
    else
        echo "⚠️  Could not install $tool automatically"
        echo "   Please install manually from: https://github.com/samtools/samtools"
        return 1
    fi
}

echo "🔍 Checking for existing tools..."
echo

# Check and install PLINK
echo "1️⃣ PLINK (Population genetics analysis)"
if command_exists plink; then
    echo "✅ PLINK is already installed: $(plink --version 2>&1 | head -n1)"
else
    echo "📥 Downloading and installing PLINK..."
    cd /tmp
    wget -q https://s3.amazonaws.com/plink1-assets/plink_linux_x86_64_20231211.zip
    unzip -q plink_linux_x86_64_20231211.zip
    sudo mv plink /usr/local/bin/
    sudo chmod +x /usr/local/bin/plink
    echo "✅ PLINK installed successfully"
fi

echo

# Check and install BCFtools
echo "2️⃣ BCFtools (VCF/BCF manipulation)"
install_tool "bcftools" "bcftools" "bcftools"

echo

# Check and install VCFtools  
echo "3️⃣ VCFtools (VCF analysis)"
install_tool "vcftools" "vcftools" "vcftools"

echo

# Check and install SAMtools
echo "4️⃣ SAMtools (Sequence alignment processing)"
install_tool "samtools" "samtools" "samtools"

echo

# Check for GATK (optional, requires Java)
echo "5️⃣ GATK (Genome Analysis Toolkit)"
if command_exists gatk; then
    echo "✅ GATK is already installed"
elif command_exists java; then
    echo "📥 Installing GATK..."
    cd /tmp
    wget -q https://github.com/broadinstitute/gatk/releases/download/4.4.0.0/gatk-4.4.0.0.zip
    unzip -q gatk-4.4.0.0.zip
    sudo mv gatk-4.4.0.0 /opt/gatk
    sudo ln -sf /opt/gatk/gatk /usr/local/bin/gatk
    echo "✅ GATK installed successfully"
else
    echo "⚠️  Java not found, skipping GATK installation"
    echo "   Install Java 8+ and rerun to install GATK"
fi

echo
echo "🔧 Installing additional dependencies..."

# Install time command for benchmarking
if ! command_exists time; then
    if install_via_apt "time"; then
        echo "✅ time command installed"
    fi
fi

# Install htop for system monitoring
if ! command_exists htop; then
    if install_via_apt "htop"; then
        echo "✅ htop installed"
    fi
fi

echo
echo "✅ Installation complete!"
echo
echo "📊 Installed tools summary:"
echo "=========================="

# Check versions of installed tools
tools=("plink" "bcftools" "vcftools" "samtools" "gatk")
for tool in "${tools[@]}"; do
    if command_exists "$tool"; then
        case "$tool" in
            "plink")
                version=$(plink --version 2>&1 | head -n1 || echo "Unknown version")
                ;;
            "bcftools"|"samtools"|"vcftools")
                version=$($tool --version 2>&1 | head -n1 || echo "Unknown version")
                ;;
            "gatk")
                version=$($tool --version 2>&1 | grep -i version | head -n1 || echo "Unknown version")
                ;;
        esac
        echo "✅ $tool: $version"
    else
        echo "❌ $tool: Not installed"
    fi
done

echo
echo "🏁 Ready for competitive benchmarking!"
echo "Run: ./target/release/instant-dna benchmark --competitors --report"
echo
echo "💡 Tips:"
echo "   • Use --iterations 5 for more accurate results"
echo "   • Ensure you have downloaded 1000 Genomes data first"
echo "   • Monitor system resources during benchmarks"
