use crate::kanpig::traverse::{get_one_to_one, prune_graph};
use crate::kanpig::{
    brute_force_find_path, metrics::overlaps, Haplotype, KDParams, KdpVcf, PathScore,
};
use itertools::Itertools;
use noodles_vcf::{self as vcf};
use petgraph::graph::{DiGraph, NodeIndex};

/// Every --input variant is placed inside a node is turned into a graph.
#[derive(Debug)]
pub struct VarNode {
    pub name: String,
    pub start: u64,
    pub end: u64,
    pub size: i64,
    pub coverage: (Option<u64>, Option<u64>),
    pub seqsim: (Option<f32>, Option<f32>),
    pub sizesim: (Option<f32>, Option<f32>),
    pub entry: Option<vcf::Record>,
    pub kfeat: Vec<f32>,
}

impl VarNode {
    pub fn new(entry: vcf::Record, kmer: u8) -> Self {
        // Want to make a hash for these names for debugging, I think.
        let name = "".to_string();
        let (start, end) = entry.boundaries();
        let (kfeat, size) = entry.to_kfeat(kmer);
        Self {
            name,
            start,
            end,
            size,
            entry: Some(entry),
            coverage: (None, None),
            seqsim: (None, None),
            sizesim: (None, None),
            kfeat,
        }
    }

    /// For the 'src' and 'snk' nodes, just need the name
    pub fn new_anchor(name: &str, kmer: u8) -> Self {
        Self {
            name: name.to_string(),
            start: 0,
            end: 0,
            size: 0,
            entry: None,
            coverage: (None, None),
            seqsim: (None, None),
            sizesim: (None, None),
            kfeat: vec![0f32; 4_usize.pow(kmer.into())],
        }
    }
}

#[derive(Debug)]
pub struct Variants {
    pub chrom: String,
    pub start: u64,
    pub end: u64,
    pub node_indices: Vec<NodeIndex>,
    pub graph: DiGraph<VarNode, ()>,
}

/// Build a graph of all variants in a chunk.
/// Assumes variants are ordered by position (small to large)
/// Variants will have edges to every downstream variant that it does not overlap
/// The graph has an upstream 'src' node that point to every variant node
/// The graph has a dnstream 'snk' node that is pointed to by every variant node and 'src'
impl Variants {
    pub fn new(mut variants: Vec<vcf::Record>, kmer: u8) -> Self {
        if variants.is_empty() {
            panic!("Cannot create a graph from no variants");
        }

        let mut graph = DiGraph::new();

        let (chrom, start, end) = Variants::get_region(&variants);
        let mut node_indices = Vec::<NodeIndex<_>>::with_capacity(variants.len() + 2);
        node_indices.push(graph.add_node(VarNode::new_anchor("src", kmer)));

        node_indices.append(
            &mut variants
                .drain(..) // drain lets us move the entry without a clone
                .map(|entry| graph.add_node(VarNode::new(entry, kmer)))
                .collect(),
        );

        node_indices.push(graph.add_node(VarNode::new_anchor("snk", kmer)));

        for pair in node_indices.iter().combinations(2) {
            if let [Some(up_node), Some(dn_node)] =
                [graph.node_weight(*pair[0]), graph.node_weight(*pair[1])]
            {
                if !overlaps(up_node.start, up_node.end, dn_node.start, dn_node.end) {
                    graph.add_edge(*pair[0], *pair[1], ());
                }
            }
        }

        Variants {
            chrom,
            start,
            end,
            node_indices,
            graph,
        }
    }

    /// Again, TR aware, we need to set the bounds for doing the pileup
    /// to the TR boundaries.
    fn get_region(entries: &Vec<vcf::Record>) -> (String, u64, u64) {
        let chrom = entries[0].chromosome().to_string();
        let mut min_start = u64::MAX;
        let mut max_end = 0;

        for e in entries {
            let (start, end) = e.boundaries();
            if start < min_start {
                min_start = start;
            }
            if end > max_end {
                max_end = end;
            }
        }

        (chrom, min_start, max_end)
    }

    // Find the path through this graph that best fits
    // the haplotype push coverage onto the VarNodes
    pub fn apply_coverage(&self, hap: &Haplotype, params: &KDParams) -> PathScore {
        // if there are no variants in the hap, we don't want to apply the coverage.
        if hap.n == 0 {
            PathScore {
                coverage: Some(hap.coverage),
                ..Default::default()
            }
        } else {
            let partial_matches = if params.prune || params.try_exact {
                get_one_to_one(&self.graph, hap, params)
            } else {
                vec![]
            };

            let skip_edges = if params.prune {
                prune_graph(
                    &self.graph,
                    &partial_matches,
                    &self.node_indices[0],
                    self.node_indices.last().unwrap(),
                )
            } else {
                vec![]
            };

            if params.try_exact {
                let mut ret = partial_matches.iter().max().cloned().unwrap();
                ret.coverage = Some(hap.coverage);
                ret
            } else {
                let mut ret = brute_force_find_path(&self.graph, hap, params, &skip_edges);
                ret.coverage = Some(hap.coverage);
                ret
            }
        }
    }
}
