use crate::cli::KDParams;
use crate::comparisons;
use crate::regions::Regions;
use noodles_vcf::{self as vcf};
use std::io::BufRead;

// Returns true if entry should be kept
pub fn filter_entry(entry: &vcf::Record, params: &KDParams) -> bool {
    if params.passonly & comparisons::entry_is_filtered(entry) {
        return false;
    }
    let size = comparisons::entry_size(entry);
    if params.sizemin > size || params.sizemax < size {
        return false;
    }

    // Need to check the variant is inside our regions, too.
    // If it is before, just false
    // If it is inside, great
    // If it start is after the current I need to pop_front and try again.
    true
}

pub struct VCFIter<R: BufRead> {
    pub m_vcf: vcf::reader::Reader<R>,
    pub m_header: vcf::Header,
    regions: Regions,
    kd_params: KDParams,
}

impl<R: BufRead> VCFIter<R> {
    pub fn new(
        m_vcf: vcf::reader::Reader<R>,
        m_header: vcf::Header,
        regions: Regions,
        kd_params: KDParams,
    ) -> Self {
        Self {
            m_vcf,
            m_header,
            regions,
            kd_params,
        }
    }
}

impl<R: BufRead> Iterator for VCFIter<R> {
    type Item = vcf::Record;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = vcf::Record::default();

        loop {
            match self.m_vcf.read_record(&self.m_header, &mut entry) {
                Ok(0) | Err(_) => return None,
                Ok(_) if filter_entry(&entry, &self.kd_params) => return Some(entry),
                _ => continue,
            }
        }
    }
}
/*
 * file_zipper to put them together
 * chunker to make the units for parsing
 * so chunk/zip can be one.. except that sometimes its 1 vcf and sometimes 2. So we want to keep
 * them separate.. except if I don't do double VCF, then I don't need both
 *
 * regions = build_region_tree
 * file1 = filter(vcf, filter settings, regions)
 * file2 = filter(vcf, filter settings, regions)
 * zipped = file_zipper([file1, file2])
 * chunks = chunker(zipped)
 */
