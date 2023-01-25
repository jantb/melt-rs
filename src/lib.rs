#![deny(
unused_import_braces,
unused_imports,
unused_variables,
unused_allocation,
unused_crate_dependencies,
unused_extern_crates
)]
#![allow(dead_code, non_upper_case_globals)]

mod bloom;
mod bucket;
pub mod index;
mod shard;
mod trigrams;

pub fn get_search_index(thread :u8) -> index::SearchIndex {
    index::SearchIndex::load_from_json(thread)
}


