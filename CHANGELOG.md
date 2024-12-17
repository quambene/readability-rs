<!-- markdownlint-disable MD041 MD034 -->

### v0.5.0

- fixed
  - Fix `ParseHtml` errors
- added
  - Add `ExtractOptions` to `extract()`
- changed
  - Restrict public interface
  - Make strict parsing configurable in `ParseOptions`
  - Distribute content score among all candidate parents
  - Make `min_candidate_length`, `positive_candidate_weight`,
    `positive_candidate_weight`, `max_candidate_parents`, and `candidate_score`
    configurable for `ScorerOptions`
  - Ignore candidates above the `body` tag in DOM
- removed
  - Remove `scrape()`

### v0.4.0

- fixed
  - Fix missing linebreaks for paragraph conversion
- added
  - Add `ReadabilityError`
- changed
  - Make `Scorer` customizable via `extract_with_scorer()`
  - Improve content score
  - Upgrade to 2021 edition

### v0.3.0

Forked from [kumabook/readability](https://github.com/kumabook/readability)
