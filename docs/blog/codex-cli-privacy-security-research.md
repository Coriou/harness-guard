# Codex CLI privacy and security research dossier

Status: preliminary source of truth for a later technical article and final
fact-check. This file is not a published blog post.

Research date: 2026-07-16

Product: Codex CLI

Latest stable version checked: 0.144.5

Harness Guard certified matcher: `<=0.144.5` through `0.144.5`

Operating systems in the certified rule and release binary: macOS and Linux

## Scope and non-claims

This dossier covers Codex CLI only. Harness Guard's current runtime and bundled
rules do not support Claude Code, Grok, or any other harness. Product-strategy
and legacy research files are not evidence of implemented support.

The one implemented audit concerns the user-level Codex setting
`history.persistence`. It is a local-storage question. It is not a proxy for
what is transmitted to OpenAI, vendor-side collection, model training,
telemetry, remote retention, or data residency.

Harness Guard must be described as a **local, execution-free,
per-finding-cited config auditor**. It does not secure Codex, produce a security
score, prove effective configuration across every layer, or prove remote vendor
behavior. No public rule-verification cadence is claimed.

No ambient Codex home, real configuration, transcript, `history.jsonl`, source
tree, credential, or secret was inspected for this research. Evidence retrieval
and package/release checks used official network sources outside any scan.
Product validation uses only synthetic roots.

## Evidence method

Technical facts use current official OpenAI documentation, official OpenAI
release metadata, or source code at the released annotated upstream tag. The live
documentation pages were retrieved on 2026-07-16 and normalized with the
repository's `scripts/freshness/normalize.sh` script. That script removes common
non-semantic page regions and markup, collapses whitespace, and computes
SHA-256. Its regex-based HTML normalization is approximate, so a hash is drift
evidence, not a content authenticity proof.

The latest npm version and GitHub release were checked separately. Tagged
source was used as an independent implementation check; it does not replace
the published configuration contract. Upstream issues are used only as search-
intent evidence. Their reports are not accepted as technical ground truth.

Evidence classes used below:

- `official-documentation`: current documentation published by OpenAI.
- `official-release-metadata`: OpenAI's npm package metadata or official GitHub
  release/tag metadata.
- `tagged-upstream-source`: source from the exact OpenAI Codex 0.144.5 release
  commit.
- `project-source-and-test`: Harness Guard runtime, rule, schema, synthetic
  fixture, or test evidence.
- `search-intent-only`: terminology or questions seen in current searches or
  upstream issue reports; never used to establish technical behavior.
- `inference`: a deliberately bounded conclusion from cited primary evidence.

## Source register

### Configuration and authentication documentation

| ID | Official source | Publisher | Retrieved | Reproducibility evidence | Archive |
| --- | --- | --- | --- | --- | --- |
| S01 | [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference) | OpenAI | 2026-07-16 | Semantic SHA-256 `de1707a407f3cadaa1bc4e86a3d289b9e64f0ce70b1edbd0b8914e98f84d185d` | [2026-07-16 snapshot](https://web.archive.org/web/20260716115618/https://learn.chatgpt.com/docs/config-file/config-reference) |
| S02 | [Codex advanced configuration](https://learn.chatgpt.com/docs/config-file/config-advanced) | OpenAI | 2026-07-16 | Semantic SHA-256 `85675e13af7727a5941bbfa2224c02f2039b322f308f3229db142f8f829b2a0f` | [2026-07-16 snapshot](https://web.archive.org/web/20260716061819/https://learn.chatgpt.com/docs/config-file/config-advanced) |
| S03 | [Codex configuration basics](https://learn.chatgpt.com/docs/config-file/config-basic) | OpenAI | 2026-07-16 | Semantic SHA-256 `9823d3632a18076df3a6283fb20fd05ee598348f797ddac5f9b15733b7014165` | No current snapshot was located |
| S04 | [Codex environment variables](https://learn.chatgpt.com/docs/config-file/environment-variables) | OpenAI | 2026-07-16 | Semantic SHA-256 `0df1241a77777e3a7049ebf611b473148bf20829e57860672f4251525522ca7e` | Not captured |
| S05 | [Codex authentication](https://learn.chatgpt.com/docs/auth) | OpenAI | 2026-07-16 | Semantic SHA-256 `f48eae3772b3068c2b65c55aca8a61d0a67b76b3c4b2998e75e83a7d42f908eb` | Not captured |
| S06 | [Data controls in the OpenAI platform](https://developers.openai.com/api/docs/guides/your-data) | OpenAI | 2026-07-16 | Semantic SHA-256 `03d408bcc3ae1e2c509b2dc12bf1e2b8453eb79bad8f52e0e42366028038bc07` from the canonical page returned by the former Platform URL | Not captured |

The article may link to sections such as
[history persistence](https://learn.chatgpt.com/docs/config-file/config-advanced#history-persistence),
[config and state locations](https://learn.chatgpt.com/docs/config-file/config-advanced#config-and-state-locations),
and
[configuration precedence](https://learn.chatgpt.com/docs/config-file/config-basic#configuration-precedence).
The hashes above cover the full canonical pages, not only those anchors.

### Version and tagged-source evidence

| ID | Official source | Publisher | Retrieved | Reproducibility evidence |
| --- | --- | --- | --- | --- |
| S07 | [npm metadata for `@openai/codex` 0.144.5](https://registry.npmjs.org/@openai%2Fcodex/latest) | OpenAI package via npm registry | 2026-07-16 | Response SHA-256 `1fab330b5011a33dcb23e42a88d88f18449cca2792723045ab0ae7352555d94d`; package tarball SHA-1 `f9a2a0c7a013990d0cd8cf3680a22a960578792a`; registry integrity `sha512-jjB+K+OMv572mKhS+2QuLxWXDJNdpwbPenf+V+8bdq7wg4Scqt3cn6WEekD8wPqDVZqck0HSX17K9rD9kbDJQA==` |
| S08 | [Official Codex 0.144.5 release](https://github.com/openai/codex/releases/tag/rust-v0.144.5) | OpenAI | 2026-07-16 | Time-specific GitHub release API response SHA-256 `7fc826dd9429f280e62b401dae7a30ebcd10ab7de2d3614c5654e652d3dc8865`; published `2026-07-16T02:54:48Z`; `draft=false`; `prerelease=false`. The mutable response hash is supporting retrieval evidence; S09's tag commit is the stable release pin. |
| S09 | [`rust-v0.144.5` tag](https://github.com/openai/codex/tree/rust-v0.144.5) | OpenAI | 2026-07-16 | Annotated tag resolves to commit `87db9bc18ba5bc82c1cb4e4381b44f693ee35623`; tag-ref response SHA-256 `d3d6696c357ed5f63843cd78327856d5282668a3a49e9322c96cdde8878864ef`; tag-object response SHA-256 `f69f014af9139d5696993625ba383dc8a623847ff9469593acba1ae9969f759a` |
| S10 | [0.144.5 history configuration types](https://github.com/openai/codex/blob/87db9bc18ba5bc82c1cb4e4381b44f693ee35623/codex-rs/config/src/types.rs#L184) | OpenAI | 2026-07-16 | File SHA-256 `3c3a34242900238cd8de0c9a2acc08e70dfb279def613a24c0686abe7a298e34` |
| S11 | [0.144.5 message-history implementation](https://github.com/openai/codex/blob/87db9bc18ba5bc82c1cb4e4381b44f693ee35623/codex-rs/message-history/src/lib.rs) | OpenAI | 2026-07-16 | File SHA-256 `17a9e5b4bbbf2afed13da6c1112be130e5e5e9e559b6db81d84c5d7fc90b4485` |
| S12 | [0.144.5 message-history tests](https://github.com/openai/codex/blob/87db9bc18ba5bc82c1cb4e4381b44f693ee35623/codex-rs/message-history/src/tests.rs) | OpenAI | 2026-07-16 | File SHA-256 `612b792909fa97e191dbec2a9b9d0e0fb8ef500e43931439f3153ce703bb75d9` |
| S13 | [0.144.5 generated configuration schema](https://github.com/openai/codex/blob/87db9bc18ba5bc82c1cb4e4381b44f693ee35623/codex-rs/core/config.schema.json) | OpenAI | 2026-07-16 | File SHA-256 `841e0ab1c1bd2fea736ba2d46212ab5bedc06dce9fd83bbafbf50b57b9056d17` |
| S14 | [0.144.5 core configuration module](https://github.com/openai/codex/blob/87db9bc18ba5bc82c1cb4e4381b44f693ee35623/codex-rs/core/src/config/mod.rs) | OpenAI | 2026-07-16 | File SHA-256 `538e507c7541e29f743775cea0c322e4878a597f192b72663ee573fdf6ef1a57` |

## Data-location model

The later article should keep these four questions separate.

| Question | What the current evidence establishes | What it does not establish |
| --- | --- | --- |
| Data stored locally | Codex state uses `CODEX_HOME`; when local history persistence is enabled, `history.jsonl` is one documented state file. Other state can include configuration, credentials depending on credential-storage mode, sessions, logs, and caches. | That `history.jsonl` is the only local state; that every listed file exists in every installation; that disabling history removes prior files or other state. |
| Data transmitted to a vendor | Changing `history.persistence` is not documented as a network or request-payload control. Codex authentication can use ChatGPT or an API key. | The exact payload sent by every Codex feature, model, provider, tool, connector, or authentication path. This research did not capture traffic. |
| Vendor-side collection, training, and retention | For API-key use, the API data-controls documentation defines training, abuse-monitoring, application-state, retention, and eligibility rules. | That the local history setting changes any of those policies; that API policy can be applied unchanged to ChatGPT-authenticated use; that every Codex request uses the same endpoint or storage behavior. |
| Account-, auth-, plan-, policy-, or geography-dependent facts | OpenAI's Codex authentication documentation says ChatGPT sign-in follows ChatGPT workspace controls, while API-key use follows API organization controls. API residency and retention controls have eligibility, project, endpoint, feature, and region conditions. | A universal retention/training answer for a person whose authentication method, workspace, plan, admin settings, endpoint use, agreement, and region are unknown. Harness Guard does not infer any of these from local files. |

## Claim ledger

### C01 — Current implemented harness coverage

- **Precise claim:** Harness Guard currently implements Codex CLI only, with
  one bundled rule for the user-level `history.persistence` setting. Claude
  Code and Grok are not implemented support.
- **Source and publisher:** Harness Guard `CONTEXT.md`, `README.md`, CLI runtime,
  bundled rule loader, and `rules/codex/history-persist-01.json`; Harness Guard
  project.
- **Retrieval date:** Local repository inspected 2026-07-16.
- **Reproducibility evidence:** `README.md` current-scope table; the only bundled
  rule is `codex-history-persist-01`; final public commit must pin this state.
- **Context:** Harness Guard current preview on its supported macOS and Linux
  targets.
- **Evidence class:** `project-source-and-test`.
- **Independent check:** Runtime tool parsing and bundled rule enumeration were
  checked separately from product-strategy documents.
- **Limitations and unknowns:** This is implemented coverage, not a statement
  about future roadmap. Historical planning documents mention other harnesses
  but do not ship them.
- **Intended article section:** Project scope and limitations.

### C02 — Latest certified Codex CLI version

- **Precise claim:** On 2026-07-16 the npm `latest` dist-tag for
  `@openai/codex` was 0.144.5, and OpenAI published a non-draft,
  non-prerelease 0.144.5 GitHub release. Harness Guard's human-certified matcher
  covers versions through exactly 0.144.5; later versions must degrade to
  `stale-ruleset` until separately certified.
- **Source and publisher:** S07 and S08; OpenAI/npm and OpenAI GitHub repository.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S07/S08 response hashes and S09 tag commit.
- **Context:** Official stable Codex CLI npm package and Rust release; Harness
  Guard ruleset `2026.07.16`.
- **Evidence class:** `official-release-metadata` plus human certification.
- **Independent check:** npm dist-tag, GitHub release flags, annotated tag, live
  docs, tagged configuration types, schema, implementation, and tests were
  checked independently.
- **Limitations and unknowns:** Package metadata is time-sensitive. The matcher
  syntax is a conservative product contract, not a claim that every historical
  Codex release has been exhaustively replayed. No version above 0.144.5 is
  certified here.
- **Intended article section:** Version applicability and methodology.

### C03 — Setting name, allowed values, and default

- **Precise claim:** The Codex setting is `history.persistence`. Its documented
  values are `save-all` and `none`; its documented and tagged-source default is
  `save-all`.
- **Source and publisher:** S01, S02, S10, and S13; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S01/S02 semantic hashes and S10/S13 tagged-file
  hashes.
- **Context:** Codex CLI configuration through 0.144.5; setting is represented
  under the `[history]` TOML table.
- **Evidence class:** `official-documentation` and `tagged-upstream-source`.
- **Independent check:** The tagged Rust enum uses kebab-case serialization,
  marks `SaveAll` as default, and the generated schema contains the same enum
  and default.
- **Limitations and unknowns:** A default at the merged-configuration level is
  not proof that an unset user file produces that value after all other config
  layers are considered.
- **Intended article section:** Exact default behavior; configuration reference.

### C04 — Local history path and custom `CODEX_HOME`

- **Precise claim:** Codex documents its state root as `CODEX_HOME`, defaulting
  to `~/.codex`. When history persistence is enabled, the documented local
  history path is `CODEX_HOME/history.jsonl`. Tagged 0.144.5 source constructs
  the path by joining the configured Codex home with `history.jsonl`.
- **Source and publisher:** S02, S04, and S11; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S02/S04 semantic hashes and S11 tagged-file hash.
- **Context:** The Codex documentation uses `~/.codex` as cross-platform
  shorthand. Harness Guard's current certified rule and release binary support
  macOS and Linux; a custom `CODEX_HOME` changes the inspected root.
- **Evidence class:** `official-documentation` and `tagged-upstream-source`.
- **Independent check:** Documentation state-location list and exact tagged
  `codex_home.join("history.jsonl")` implementation agree.
- **Limitations and unknowns:** Do not render a real absolute home directory or
  username. Do not assume a platform-specific expanded path that the official
  docs do not state. A custom root must be accounted for in remediation and
  verification instructions.
- **Intended article section:** Storage paths; custom `CODEX_HOME`; OS notes.

### C05 — What the local history records contain

- **Precise claim:** OpenAI documentation describes `history.jsonl` as storing
  local session transcripts. Tagged 0.144.5 source serializes a JSON-line
  history entry containing a session identifier, Unix timestamp, and text.
- **Source and publisher:** S02 and S11; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S02 semantic hash and S11 tagged-file hash.
- **Context:** The message-history component in Codex CLI 0.144.5.
- **Evidence class:** `official-documentation` and `tagged-upstream-source`.
- **Independent check:** Tagged tests append and retrieve synthetic entries;
  no real history file was opened.
- **Limitations and unknowns:** Do not claim that this file is a complete record
  of every session event, tool call, remote request, or other Codex state.
  `history.jsonl` must never be inspected by Harness Guard.
- **Intended article section:** What Codex stores locally.

### C06 — Meaning of `persistence = "none"`

- **Precise claim:** Configuring `[history] persistence = "none"` is the
  documented way to disable local history persistence. In tagged 0.144.5
  source, the append path returns without writing when persistence is `None`.
- **Source and publisher:** S02, S10, and S11; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S02, S10, and S11 hashes.
- **Context:** Codex CLI 0.144.5 local message-history append behavior, subject
  to merged configuration precedence.
- **Evidence class:** `official-documentation` and `tagged-upstream-source`.
- **Independent check:** Documentation instruction, enum definition, and
  implementation branch agree.
- **Limitations and unknowns:** This establishes prevention of new writes by
  that component when the effective value is `none`. It does not establish
  deletion of an existing `history.jsonl`, deletion of session/SQLite/log/cache
  state, or any remote-policy change. No deletion command is certified by this
  research.
- **Intended article section:** How to disable persistence; limitations.

### C07 — Meaning and limits of `history.max_bytes`

- **Precise claim:** `history.max_bytes` is an optional byte-size limit. When a
  positive configured limit is exceeded, tagged 0.144.5 source drops oldest
  JSON lines and rewrites the retained tail toward a soft target while always
  retaining the newest entry.
- **Source and publisher:** S01, S02, S10, S11, S12, and S13; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** Registered documentation and tagged-file hashes.
- **Context:** Codex CLI 0.144.5 local `history.jsonl` compaction.
- **Evidence class:** `official-documentation` and `tagged-upstream-source`.
- **Independent check:** Tagged tests verify eviction of older entries and
  compaction below the configured limit in their synthetic cases.
- **Limitations and unknowns:** This is a size control, not a time-based
  retention promise. It does not disable persistence or prove deletion from
  backups, other stores, or vendor systems. Tagged source treats zero as no
  enforcement, while the public prose does not prominently explain that edge;
  the article should discuss only positive values. Because the newest entry is
  always retained, one entry larger than the configured limit can remain above
  the nominal cap.
- **Intended article section:** What `history.max_bytes` does and does not do.

### C08 — Configuration precedence limits a user-file audit

- **Precise claim:** Codex resolves configuration from highest to lowest as CLI
  overrides, trusted project config, selected profile, user config, system
  config, and built-in defaults. Therefore reading only
  `CODEX_HOME/config.toml` does not prove the effective value for every project,
  profile, or invocation.
- **Source and publisher:** S03, with profile and override detail in S02; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S03 and S02 semantic hashes.
- **Context:** Current Codex configuration documentation; profile behavior notes
  include version-specific changes from Codex 0.134.0 onward.
- **Evidence class:** `official-documentation`.
- **Independent check:** The ordering was checked on the conceptual basics page
  and cross-checked against the advanced profile and one-off override sections.
- **Limitations and unknowns:** Harness Guard currently reads only the user-level
  file. An explicit user-level `none` or `save-all` describes that layer; an
  unset user-level value is `unknown`, not proof that the built-in default wins.
- **Intended article section:** Config precedence; why local verification is
  conservative.

### C09 — `history.jsonl` is not all local Codex state

- **Precise claim:** OpenAI documents additional local state under
  `CODEX_HOME`, including configuration, credentials when file-based storage is
  used, and other state such as logs and caches; current environment-variable
  documentation also names sessions and SQLite-backed state.
- **Source and publisher:** S02, S04, and S05; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S02/S04/S05 semantic hashes.
- **Context:** Codex local surfaces; the exact files vary with configuration,
  credential store, features, and surface.
- **Evidence class:** `official-documentation`.
- **Independent check:** Config/state, environment-variable, and authentication
  pages describe overlapping state categories.
- **Limitations and unknowns:** The article must not imply that disabling
  message history makes the entire Codex home ephemeral or removes credentials,
  sessions, SQLite state, logs, caches, or package metadata. Harness Guard does
  not inspect those stores.
- **Intended article section:** What else Codex may store; project limitations.

### C10 — Local persistence is not vendor collection

- **Precise claim:** `history.persistence` controls a local Codex message-history
  write path. The cited config documentation and tagged code do not connect it
  to request transmission, vendor collection, training, telemetry, or remote
  retention.
- **Source and publisher:** S01, S02, S10, and S11; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** Registered source hashes.
- **Context:** Codex CLI through 0.144.5.
- **Evidence class:** `inference` from official documentation and exact tagged
  implementation.
- **Independent check:** The local write behavior was compared with the separate
  authentication and API data-control sources S05/S06.
- **Limitations and unknowns:** Absence of a documented link is not proof that
  no other data flow exists. No network capture was performed. The safe article
  wording is that changing local history **does not establish** any change in
  remote behavior.
- **Intended article section:** Local storage versus remote collection.

### C11 — Authentication selects a different policy context

- **Precise claim:** Codex supports ChatGPT sign-in and API-key sign-in for local
  work. OpenAI says ChatGPT sign-in follows the selected ChatGPT workspace's
  controls, while API-key use follows the API organization's retention and
  data-sharing controls.
- **Source and publisher:** S05; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S05 semantic hash.
- **Context:** Codex CLI, desktop app, and IDE local work; Codex cloud has a
  different authentication constraint and is outside this article's local CLI
  scope.
- **Evidence class:** `official-documentation`.
- **Independent check:** S05 was checked against S06's organization/project
  model for API controls.
- **Limitations and unknowns:** Authentication method, workspace, account type,
  plan, administrative settings, agreement, geography, and enabled features
  cannot safely be inferred from the local history setting. Harness Guard does
  not inspect credentials to infer them.
- **Intended article section:** Auth/account-dependent unknowns.

### C12 — API-key remote policy is separate and conditional

- **Precise claim:** OpenAI's API documentation says API data is not used to
  train OpenAI models unless the customer explicitly opts in. It separately
  documents abuse-monitoring logs and application state; default abuse-
  monitoring retention can be up to 30 days, subject to stated legal and safety
  exceptions. Modified Abuse Monitoring and Zero Data Retention require
  eligibility and approval, and storage behavior can vary by endpoint and
  feature.
- **Source and publisher:** S06; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S06 semantic hash.
- **Context:** OpenAI API organizations/projects, not a universal statement
  about ChatGPT-authenticated Codex. Residency additionally depends on project,
  region, endpoint, model, feature, and eligibility.
- **Evidence class:** `official-documentation`.
- **Independent check:** The top-level training statement, abuse-monitoring
  section, retention-control eligibility text, endpoint table, and residency
  limitations were checked within the same current canonical page; S05 supplies
  the auth-to-policy routing.
- **Limitations and unknowns:** This dossier does not establish which API
  endpoints and optional features a particular Codex 0.144.5 session uses.
  Therefore it does not assign an endpoint-specific retention period to Codex
  CLI. Third-party providers, MCP servers, proxies, and tools may have separate
  policies.
- **Intended article section:** Vendor-side training and retention; API-key
  context.

### C13 — A safe configuration example

- **Precise claim:** The minimally documented configuration for disabling new
  local message-history persistence is:

  ```toml
  [history]
  persistence = "none"
  ```

- **Source and publisher:** S02; OpenAI.
- **Retrieval date:** 2026-07-16.
- **Reproducibility evidence:** S02 semantic hash; S10/S11 independently confirm
  parsing and implementation semantics for 0.144.5.
- **Context:** Place in the intended Codex configuration layer. Harness Guard's
  remediation targets `CODEX_HOME/config.toml`, normally `~/.codex/config.toml`,
  because the implemented audit is user-scoped.
- **Evidence class:** `official-documentation` and `tagged-upstream-source`.
- **Independent check:** The later tutorial-validation agent must run all
  non-destructive examples against a synthetic temporary `CODEX_HOME` and must
  not execute Codex or inspect a real config.
- **Limitations and unknowns:** A higher-precedence trusted-project or CLI
  override can change the effective value. This does not delete existing
  history and does not alter known or unknown remote policies. No destructive
  cleanup command is approved by this dossier.
- **Intended article section:** Safe setup tutorial; disabling persistence.

### C14 — What Harness Guard can report

- **Precise claim:** For a supported version, Harness Guard can report the
  normalized user-level observation `none`, `save-all`, or `unset` without
  exposing raw config. Explicit `save-all` is a cited warning; explicit `none`
  is a cited pass about the inspected user layer; unset is `unknown` because
  uninspected layers may determine the effective value. Out-of-range versions
  remain conservative rather than silently passing.
- **Source and publisher:** Current Harness Guard rule, schemas, evaluator,
  synthetic fixtures, and report tests; Harness Guard project. Vendor semantics
  are sourced by S01-S03.
- **Retrieval date:** Repository state inspected 2026-07-16.
- **Reproducibility evidence:** `rules/codex/history-persist-01.json`, synthetic
  fixture goldens, and workspace tests; final article must pin its examples to
  the final public commit.
- **Context:** Harness Guard ruleset `2026.07.16`; Codex versions through
  0.144.5; user scope only.
- **Evidence class:** `project-source-and-test` plus cited
  `official-documentation`.
- **Independent check:** Terminal and JSON outputs share validated report
  structs, and the release workflow separately validates synthetic cases.
- **Limitations and unknowns:** The tool does not inspect effective merged
  configuration, `history.max_bytes`, prior history content, auth method, or
  remote behavior. A pass is not a security guarantee.
- **Intended article section:** Verifying posture without exposing config;
  optional automated check; limitations.

## Search-intent research

This section shapes questions and terminology only. It is not evidence for any
technical claim above.

### Query set

Current research targeted these queries on 2026-07-16:

- `Codex CLI security`
- `Codex CLI privacy`
- `Codex CLI history`
- `Codex history.jsonl`
- `disable Codex history`
- `Codex CODEX_HOME`
- `Codex config.toml`
- `what does Codex CLI store`
- `Codex data retention`
- `secure Codex CLI setup`

The actual questions implied by those terms cluster as follows:

1. What does Codex store locally by default?
2. Where is `history.jsonl`, especially with a custom `CODEX_HOME` or on a
   different OS?
3. How can a developer stop future local history writes?
4. Does disabling local history remove existing data?
5. What does `history.max_bytes` limit, and is it a retention period?
6. Which config file wins when user, profile, project, system, or CLI settings
   disagree?
7. Is local history the same thing as data sent to OpenAI?
8. How do ChatGPT sign-in, API-key use, workspace plan, organization policy,
   and geography change remote data handling?
9. How can a developer verify posture without pasting or printing config
   contents?
10. What does an `unknown`, `stale-ruleset`, or nonzero audit exit mean?

These are preferable article headings and FAQ questions to repeating keyword
variants. The answer should be useful without Harness Guard; the project should
appear only as an optional automated check.

### Upstream issue terminology

Two bounded GitHub API searches of the official `openai/codex` issue tracker
corroborated interest in history scope and ephemeral behavior:

- Exact query `repo:openai/codex "history.persistence" is:issue` returned three
  issues on 2026-07-16. Response SHA-256:
  `f84d90cb0f2c374503b75a66fc7182b206d2e40a1d5d7e2111b181bd1bba2ade`.
  Examples include
  [#21202, prompt-recall history scope](https://github.com/openai/codex/issues/21202)
  and
  [#26283, ephemeral execution and transcript replay](https://github.com/openai/codex/issues/26283).
- Exact query `repo:openai/codex "history.jsonl" is:issue` returned 28 issues
  on 2026-07-16. Response SHA-256:
  `7a337fe62da13943b112a2eee1cee86182d8fb37002b6935f77002379937564b`.

Issue titles and counts are volatile and some reports concern other Codex
surfaces or other JSONL state. They justify plain-language FAQ coverage only;
they do not verify a file path, default, vulnerability, or current behavior.
SERP snippets, community answers, Reddit, and third-party blogs were excluded
from technical evidence.

## Article-safe conclusions

The later draft may state these conclusions when it preserves their scope:

- Codex CLI 0.144.5 defaults `history.persistence` to `save-all`.
- Local message history is documented at `CODEX_HOME/history.jsonl` when
  persistence is enabled; `CODEX_HOME` defaults to `~/.codex`.
- `[history] persistence = "none"` disables new local message-history writes in
  the tagged implementation when it is the effective merged setting.
- `history.max_bytes` is a positive byte-size compaction control, not a time-
  retention policy and not a substitute for disabling persistence.
- Reading only user-level `config.toml` cannot prove the effective setting
  across all profiles, trusted projects, and CLI overrides.
- Disabling local history does not establish anything about data transmission,
  vendor collection, training, telemetry, or remote retention.
- Remote-policy interpretation depends at minimum on authentication route and
  the applicable ChatGPT workspace or API organization controls; more specific
  answers can also depend on plan, administrator choices, agreement, project,
  endpoint, feature, and geography.
- Harness Guard can perform an optional synthetic-tested, normalized,
  per-finding-cited user-config check, but its pass/unknown/finding results must
  be interpreted within its explicit scope.

## Claims the article must not make

- Do not call local `history.jsonl` vendor collection or remote retention.
- Do not claim that `persistence = "none"` deletes an existing file or any
  other state.
- Do not claim that disabling local history changes model training, telemetry,
  data transmission, remote retention, or residency.
- Do not claim that `history.max_bytes` is a duration, guarantees a hard maximum
  in every case, or removes data from backups or vendor systems.
- Do not claim one universal OpenAI retention/training policy without the
  user's authentication, workspace/account, plan, organization/project
  controls, feature/endpoint, agreement, and geography.
- Do not infer authentication from `auth.json`, a keyring, environment, or
  another local artifact.
- Do not tell readers to print, paste, upload, or share `config.toml`,
  `auth.json`, `history.jsonl`, session files, logs, or secrets.
- Do not recommend or execute a destructive cleanup command without separate
  primary evidence, explicit warnings, and user intent.
- Do not imply Harness Guard covers Claude Code, Grok, system config, profiles,
  trusted-project config, CLI overrides, or `history.max_bytes`.
- Do not say Harness Guard secures Codex, proves privacy, produces a security
  score, or proves remote behavior.
- Do not claim a public evidence-refresh cadence while the freshness workflows
  remain default-off.

## Unresolved unknowns and required final checks

1. **Exact transmitted payloads:** This research did not capture network
   traffic and does not enumerate the exact request payload for every Codex
   feature, tool, provider, or authentication path.
2. **ChatGPT-authenticated remote retention:** The authentication page routes
   policy interpretation to workspace controls, but this dossier does not
   establish one retention period for personal, Business, Enterprise, Edu, or
   geographically restricted ChatGPT workspaces.
3. **Endpoint-specific API retention:** S06 is explicit that endpoint and
   feature behavior differs. The exact endpoint/feature mix of a given Codex
   session was not certified, so no single endpoint row is assigned to Codex
   CLI generally.
4. **Effective merged configuration:** Harness Guard does not inspect the
   system, selected profile, trusted-project, or CLI layers. User-file results
   must remain layer-scoped.
5. **Existing-history deletion:** Primary sources reviewed here establish how
   to stop future writes, not a safe universal deletion workflow. No deletion
   command should appear as an ordinary tutorial step.
6. **Other local state:** Disabling `history.jsonl` persistence does not
   establish the absence or lifecycle of sessions, SQLite state, logs, caches,
   credentials, worktrees, or standalone-package metadata.
7. **Zero value for `history.max_bytes`:** Tagged source treats zero as no
   enforcement, but the public prose does not foreground this edge. Recommend
   only positive examples and state the version scope.
8. **OS path expansion:** Official docs use `~/.codex` as the default shorthand
   and document `CODEX_HOME`. The final article should avoid inventing an
   unsupported absolute path. Harness Guard does not currently support Windows;
   shell examples should be tested separately for Bash and Zsh against
   synthetic roots on macOS or Linux.
9. **Tutorial validation:** Every non-destructive command and sanitized
   Harness Guard example must be run by an independent validation agent against
   a temporary synthetic `HOME`/`CODEX_HOME`. Codex itself must not be executed
   as part of Harness Guard scan validation.
10. **Final project pin:** Product-output claims and examples must be pinned to
    the final public Harness Guard commit after local validation and private CI
    are green.
11. **Future versions:** Any Codex version newer than 0.144.5 requires fresh
    official evidence and a new human certification checkpoint. Until then it
    must remain `stale-ruleset`/unverified.

## Final fact-check checklist

Before a later agent turns this dossier into publication-ready copy, it must:

1. Re-query the npm `latest` dist-tag and official OpenAI release list.
2. Confirm the article still names only the versions certified by the bundled
   rule.
3. Re-fetch S01-S06 and compare fresh normalized hashes with this register.
4. Re-check the exact article wording against the cited source section rather
   than relying only on matching hashes.
5. Verify all path examples honor custom `CODEX_HOME` and reveal no real home or
   username.
6. Verify no statement merges local storage, transmission, collection,
   training, telemetry, or retention into one concept.
7. Verify every remote-policy statement identifies its auth/account/plan/
   project/feature/geography scope or is explicitly unknown.
8. Run every non-destructive tutorial command in isolated synthetic roots and
   preserve outputs without raw config.
9. Compare sanitized terminal and JSON examples with the final binary and final
   report schema, including exit codes 0, 1, and 2.
10. Confirm the public repository URL, final commit, README, rule, licenses, and
    CI before adding repository links to the article.
11. Keep Harness Guard optional and retain the positioning “local,
    execution-free, per-finding-cited config auditor.”
12. Leave destructive deletion guidance out unless separately sourced and
    explicitly approved.
