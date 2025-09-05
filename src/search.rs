use anyhow::{anyhow, Result};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{self, *},
    tokenizer::*,
    Index, IndexWriter, Searcher, TantivyDocument,
};

use crate::types::System;

pub(crate) struct SearchIndex {
    fields: Fields,
    searcher: Searcher,
    query_parser: QueryParser,
}

impl SearchIndex {
    pub(crate) fn new<'a>(systems: impl IntoIterator<Item = &'a System>) -> Result<Self> {
        let mut builder = schema::Schema::builder();

        let tokenizer = TextAnalyzer::builder(NgramTokenizer::new(2, 3, false).unwrap())
            .filter(AsciiFoldingFilter)
            .filter(LowerCaser)
            .build();

        let text_field_indexing = TextFieldIndexing::default()
            .set_tokenizer("tok")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_field_indexing)
            .set_stored();

        let name = builder.add_text_field("name", text_options);
        let id = builder.add_i64_field("id", schema::INDEXED | schema::STORED);

        let schema = builder.build();

        let index = Index::create_in_ram(schema);
        index.tokenizers().register("tok", tokenizer);

        let mut writer: IndexWriter = index.writer(15_000_000)?;

        for system in systems {
            writer.add_document(doc! {
                name => system.name.clone(),
                id => system.id.0 as i64,
            })?;
        }
        writer.commit()?;

        let reader = index.reader()?;
        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(&index, vec![name, id]);

        Ok(Self {
            fields: Fields { id },
            searcher,
            query_parser,
        })
    }

    pub(crate) fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        use schema::document::Value;
        let query = self.query_parser.parse_query(query)?;
        let top_docs = self.searcher.search(&query, &TopDocs::with_limit(10))?;
        top_docs
            .into_iter()
            .map(|(score, address)| {
                let doc = self.searcher.doc::<TantivyDocument>(address)?;
                let id = doc
                    .get_first(self.fields.id)
                    .ok_or(anyhow!("missing id"))?
                    .as_i64()
                    .ok_or(anyhow!("error converting to i64"))?;
                Ok(SearchResult { id, _score: score })
            })
            .collect::<Result<Vec<_>>>()
    }
}

struct Fields {
    id: schema::Field,
}

pub(crate) struct SearchResult {
    pub(crate) id: i64,
    pub(crate) _score: f32,
}
