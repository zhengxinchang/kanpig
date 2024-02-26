extern crate pretty_env_logger;

use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(author = "ACEnglish", version)]
pub struct ArgParser {
    #[command(flatten)]
    pub io: IOParams,

    #[command(flatten)]
    pub kd: KDParams,
}

#[derive(clap::Args, Clone, Debug)]
pub struct IOParams {
    /// VCF to genotype
    #[arg(short, long)]
    pub input: std::path::PathBuf,

    /// Reads to genotype
    #[arg(short, long)]
    pub bam: std::path::PathBuf,

    /// Reference bam is aligned to
    #[arg(short = 'f', long)]
    pub reference: std::path::PathBuf,

    /// Output vcf (unsorted)
    #[arg(short, long)]
    pub out: std::path::PathBuf,

    /// Regions to analyze
    #[arg(long)]
    pub bed: Option<std::path::PathBuf>,

    /// Sample to apply genotypes to, default first column
    #[arg(long)]
    pub sample: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct KDParams {
    /// Kmer size for featurization
    #[arg(long, default_value_t = 4)]
    pub kmer: u8,

    /// Minimum distance between variants to create independent graphs
    #[arg(long, default_value_t = 100)]
    pub chunksize: u64,

    /// Only analyze reads with PASS FILTER
    #[arg(long, default_value_t = false)]
    pub passonly: bool,

    /// Minimum size of variant to analyze
    #[arg(long, default_value_t = 20)]
    pub sizemin: u64,

    // change this to 10k? also filtered variants are just plain gone...
    /// Maximum size of variant to analyze
    #[arg(long, default_value_t = 50000)]
    pub sizemax: u64,

    /// Maximum number of paths in a graph to traverse
    #[arg(long, default_value_t = 1000)]
    pub maxpaths: usize,

    /// Minimum cosine similarity for paths
    #[arg(long, default_value_t = 0.90)]
    pub cossim: f32,

    /// Minimum size similarity for paths
    #[arg(long, default_value_t = 0.90)]
    pub pctsize: f32,

    /// Size threshold above which weighted cosine similarity is used
    #[arg(long, default_value_t = 2000)]
    pub wcoslen: usize,

    /// Number of attempts through the graph (experimental)
    #[arg(long, default_value_t = 5)]
    pub n_tries: usize,
}

impl ArgParser {
    /// Validate command line arguments
    pub fn validate(&self) -> bool {
        let mut is_ok = true;

        if !self.io.input.exists() {
            error!("--input does not exist");
            is_ok = false;
        }

        if !self.io.bam.exists() {
            error!("--bam does not exist");
            is_ok = false;
        }

        if !self.io.reference.exists() {
            error!("--reference does not exist");
            is_ok = false;
        }

        if let Some(bed_file) = &self.io.bed {
            if !bed_file.exists() {
                error!("--bed does not exist");
                is_ok = false;
            }
        }

        if self.kd.sizemin < 20 {
            warn!("--sizemin is recommended to be at least 20");
        }

        if self.kd.kmer >= 8 {
            warn!("--kmer above 8 becomes memory intensive");
        }

        is_ok
    }
}
