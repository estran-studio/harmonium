//! Harmonium Lab — extraction d'ADN musical, corpus, comparaison et rendu de profils.
//!
//! This crate provides tools for:
//! - Ingesting music scores (MusicXML/PDMX) and extracting Musical DNA
//! - Comparing generated music against reference corpora
//! - Building style profiles from music collections
//!
//! ## Modules
//!
//! - `dna_types` - Musical DNA intermediate representation (MusicalDNA, GlobalMetrics, etc.)
//! - `ingest` - MusicXML parsing and DNA extraction from scores
//! - `dna` - DNA comparison and similarity metrics
//! - `benchmark` - Style profile management and benchmarking

pub mod benchmark;
pub mod dna;
pub mod dna_types;
pub mod ingest;
pub mod render;

// Re-export key types
pub use benchmark::DNAProfile;
pub use dna::DNAComparator;
pub use dna_types::{GlobalMetrics, MusicalDNA};
pub use ingest::MusicXMLIngester;
