# typesensei

**Client library for typesense search engine**

**This library is very much in WIP and should not be used**

- Derive macro
  - [x] `Typesense` derive macro
- Collections API
  - [x] retrieve collection info
  - [x] create new collection
  - [ ] update collection
  - [ ] delete collection
- Documents API
  - [x] retrieve document by id
  - [x] create, update, delete document by id
  - [x] batch create, update, delete document by id
  - [ ] import jsonl, json, csv files
  - [ ] update partial document
  - [ ] delete documents by query
  - [ ] export documents as jsonl
- Search API
  - [ ] search documents by query, query_by, filter_by, sort_by, etc.
  - [ ] multi-search
  - [ ] GeoSearch API
- Config API
  - [ ] generate new API key
  - [ ] delete API key
  - [ ] retrieve API key
  - [ ] list all keys
- Extra
  - [ ] Overrides
  - [ ] Collection Alias
  - [ ] Cluster operations
- Error
  - [ ] Translate error codes to error?
