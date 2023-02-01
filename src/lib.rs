#![deny(
unused_import_braces,
unused_imports,
unused_variables,
unused_allocation,
unused_extern_crates
)]
#![allow(dead_code, non_upper_case_globals)]

mod bloom;
mod bucket;
pub mod index;
mod shard;
mod trigrams;

pub fn get_search_index() -> index::SearchIndex {
    index::SearchIndex::default()
}
pub fn get_search_index_with_prob(prob: f64) -> index::SearchIndex {
    index::SearchIndex::default_with_prob(prob)
}


