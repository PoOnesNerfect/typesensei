# typesensei

**Client library for typesense search engine**

- Derive macro
  - [x] `Typesense` derive macro
- Collections API
  - [x] retrieve collection info
  - [x] create new collection
  - [x] update collection
  - [x] delete collection
- Documents API
  - [x] retrieve document by id
  - [x] create, update, delete document by id
  - [x] batch create, update, delete document by id
  - [x] import jsonl, json, csv files
  - [x] update partial document
  - [ ] delete documents by query
  - [ ] export documents as jsonl
- Search API
  - [ ] search documents by query, query_by, filter_by, sort_by, etc.
  - [ ] multi-search
  - [ ] GeoSearch API
- Config API
  - [x] generate new API key
  - [x] delete API key
  - [x] retrieve API key
  - [x] list all keys
- Extra
  - [x] Overrides
  - [x] Collection Alias
  - [ ] Cluster operations
- Error
  - [ ] Translate error codes to error?
