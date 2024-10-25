pub mod cache;
pub mod config;
pub mod evaluation;
pub mod metrics;
pub mod ngram_mapper;
pub mod ngrams;
pub mod results;
pub mod sval;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
