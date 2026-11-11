use bionetic_search::{concept::ConceptId, corpus::seed_corpus, engine::{SearchEngine, SearchEngineConfig}, query::QueryBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SearchEngine::new(seed_corpus(), SearchEngineConfig::default());
    let query = QueryBuilder::new()
        .concept(ConceptId(0))
        .concept(ConceptId(1))
        .limit(5)
        .build();
    let result = engine.search(&query)?;
    println!("{} hits in {}us", result.hits.len(), result.latency_us);
    for (rank, hit) in result.hits.iter().enumerate() {
        println!("#{rank} score={:.3} {}", hit.score, hit.title);
        println!("  {}", hit.snippet);
    }
    Ok(())
}
