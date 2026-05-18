---
name: scenario-design
description: Codex 側のシナリオ設計作業プロトコル。必須要件、受け入れテスト観点、システムテスト分類、受け入れ条件、検証入口を task 内成果物 に固定する基準を提供する。
---
# シナリオ設計

## 目的

`scenario-design` は作業プロトコルである。
`designer` agent が必須要件、シナリオ、受け入れ条件を固定するための、観測点、テスト語彙、fake / stub、検証コマンド、リスク の見方を提供する。

実行境界、正本、引き継ぎ、停止 / 戻し は [design-bundle](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/design-bundle/SKILL.md) を参照する。

## 対応ロール

- `designer` が使う。
- 呼び出し元は `implement_lane` または人間とする。
- 返却先は 人間レビュー または `implement_lane` とする。
- 担当成果物は `scenario-design` の出力規約で固定する。

## 入力規約

- task 内成果物: 呼び出し元から渡された設計成果物。
- 根拠参照: シナリオ設計の根拠にする要件、候補成果物、観測事実。
- 承認状態: 呼び出し元が渡す承認済みまたは未承認の状態。

## 外部参照規約

- エージェント実行定義と実行境界は [designer.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/designer.toml) に従う。
- 要件正本: [spec.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/spec.md) とする。
- architecture 正本: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) とする。
- ER 正本: [er.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/er.md) と [diagrams/er](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/diagrams/er/) とする。
- 画面正本: [screen-design](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/screen-design/README.md) とする。
- 上位シナリオ詳細仕様正本: [detail-specs](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/detail-specs/README.md) とする。
- scenario 正本: [scenario-tests](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/scenario-tests/README.md) とする。
- シナリオ設計雛形: [scenario-design.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-design/assets/scenario-design.md)
- 候補網羅 JSON 雛形: [scenario-design.candidate-coverage.json](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-design/assets/scenario-design.candidate-coverage.json)
- 候補生成 観点別 skill: [actor-goal](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-actor-goal-generation/SKILL.md)、[lifecycle](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-lifecycle-generation/SKILL.md)、[state-transition](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-state-transition-generation/SKILL.md)、[失敗](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-failure-generation/SKILL.md)、[external-integration](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-external-integration-generation/SKILL.md)、[operation-audit](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-operation-audit-generation/SKILL.md)
- 実行定義 skill: [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/design-bundle/SKILL.md)
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

### シナリオ候補生成

シナリオ 候補生成は `implement_lane` が `designer` の前に指揮する。
`designer` は候補生成器を再 起動 せず、作業計画フォルダに揃った 候補成果物 を統合する。

候補生成 agent は次の 6 体に固定する。

| agent | 出力ファイル | 観点 |
| --- | --- | --- |
| `scenario_actor_goal_generator` | `scenario-candidates.actor-goal.md` | アクター目的ベース |
| `scenario_lifecycle_generator` | `scenario-candidates.lifecycle.md` | ライフサイクルベース |
| `scenario_state_transition_generator` | `scenario-candidates.state-transition.md` | 状態遷移ベース |
| `scenario_failure_generator` | `scenario-candidates.failure.md` | 異常系 |
| `scenario_external_integration_generator` | `scenario-candidates.external-integration.md` | 外部連携 |
| `scenario_operation_audit_generator` | `scenario-candidates.operation-audit.md` | 運用・監査 |

各 候補ファイル は、対応する観点別 skill の `assets/scenario-candidates.viewpoint.md` で書く。
必須項目は `根拠要件`、`観点`、`候補 シナリオ id`、`実行者`、`trigger`、`expected 結果`、`observable point`、`related detail requirement type`、`adoption hint` とする。

`designer` は候補を読んで、最終 シナリオ表 の前に `scenario-design.candidate-coverage.json` を作る。
この JSON は `生成 agent`、`candidates`、`conflicts`、`final_mapping`、`unresolved_questions` を持つ。

候補 の `decision` は次に固定する。

- `adopted`
- `merged`
- `rejected`
- `conflicted`
- `needs_human_decision`

`adopted` と `merged` は `final_scenario_id` を持つ。
`rejected` は `decision_rationale` を持つ。
`conflicted` と `needs_human_decision` は `question_id` を持ち、質問票へ出す。

### 競合処理

競合は `scenario-design.questions.md` に流す。
質問票は詳細要求タイプ未決と シナリオ 候補競合を同じファイルにまとめる。

競合検知対象は次にする。

- 同じ要求から異なる正常系 結果 が出ている
- 状態遷移の前提が 生成 agent 間で矛盾している
- 異常系が正常系の受け入れ条件を否定している
- 外部連携の失敗扱いが lifecycle と矛盾している
- 運用・監査の保存対象が `security_requirement` / `data_requirement` と衝突している
- UI / API / lower-level の検証段階が シナリオ 間で食い違っている

未解決競合が 1 件でもあれば シナリオ 完了 にしない。

### 詳細要求タイプ

抽象要件は、シナリオ を作る前に詳細要求タイプへ展開する。
展開目的は「AI が推測で埋めた判断」を検出し、人間に確認すべき未決だけを質問票へ出すことである。

| 観点 | 問い | 要求タイプ |
| --- | --- | --- |
| 正常系 | 何が成功すればよいか | `success_requirement` |
| 代替系 | 別ルートで成功する条件は何か | `alternative_success_requirement` |
| 例外系 | 何が失敗し、どう扱うか | `failure_handling_requirement` |
| 境界値 | 最小、最大、空、重複、期限はどう扱うか | `boundary_requirement` |
| 状態 | どの状態なら実行可能か | `state_requirement` |
| データ | 何を作成、更新、保存するか | `data_requirement` |
| 整合性 | どの結果が同時に成立すべきか | `consistency_requirement` |
| 権限 | 誰ができて、誰ができないか | `authorization_requirement` |
| セキュリティ | 漏洩、越権、改ざんをどう防ぐか | `security_requirement` |
| 競合 | 同時実行時にどうなるか | `concurrency_requirement` |
| 冪等性 | 再送、再実行でどうなるか | `冪等性_requirement` |
| 観測性 | ログ、監査、メトリクスに何を残すか | `observability_requirement` |
| 回復 | 失敗後にどう復旧するか | `recovery_requirement` |
| 性能 | どの量、時間まで許容するか | `performance_requirement` |
| 回帰 | 既存仕様に何を壊してはいけないか | `compatibility_requirement` |
| テスト容易性 | どう検証できるべきか | `testability_requirement` |

要件種別ごとに、各詳細要求タイプを `必須`、`条件付き`、`optional`、`not_applicable` に分類する。
常に全タイプを必須にせず、対象外にする場合も理由を明示する。

### 明示性判定

各詳細要求タイプは次のいずれかに分類する。

- `explicit`: 根拠成果物 に明示されている
- `derived`: 明示情報から機械的に導出できる
- `not_applicable`: 対象外の理由が明示されている
- `deferred`: 延期理由、担当者、再確認条件が明示されている
- `needs_human_decision`: AI が推測すれば埋められるが、人間判断が必要である

`needs_human_decision` が 1 件でも残る場合は シナリオ 完了にしない。
未決項目だけを質問票へまとめ、人間回答後に詳細要求タイプの明示状態を再評価する。
`not_applicable` と `deferred` は理由が空なら通さない。

repo-local 判定条件 は [requirement_gate.py](/Users/iorishibata/Repositories/AITranslationEngineJP/scripts/scenario/requirement_gate.py) を使う。
active task 全体は `python3 scripts/harness/run.py --suite scenario-gate` で検査する。
単体ファイルは `python3 scripts/scenario/requirement_gate.py docs/exec-plans/active/<task-id>/scenario-design.md --report-out docs/exec-plans/active/<task-id>/scenario-design.requirement-gate.md --questionnaire-out docs/exec-plans/active/<task-id>/scenario-design.questions.md` で検査する。

`scenario-design.requirement-coverage.json` がある場合、判定条件 はその JSON を読む。
旧形式の fenced JSON は互換用に読めるが、新規 成果物 では使わない。
`scenario-design.candidate-coverage.json` は新規 成果物 で必須とする。
判定条件 は `needs_human_decision`、候補の `conflicted`、候補の `needs_human_decision`、未解決競合だけを検査する。
判定条件 は質問票の項目数、選択肢数、説明文、推奨理由、内部用語の読みやすさを検査しない。
質問票の読みやすさと仕様質問としての妥当性は `designer` が判断する。
判定条件 が出す `scenario-design.questions.md` は未回答 ID 一覧にとどめる。
人間向け質問本文は `designer` が同じ正本へ再編集する。

### 質問票

質問票は、明示的ではない判断のうち、人間が決める必要のある仕様境界を対象にする。
人間が全 成果物 を読み直さなくても答えられるように、決める仕様、決定済み、未確定、選択肢、AI 推奨だけを添える。
`needs_human_decision` は質問票へそのまま転記せず、人間向けの問いへ再編集する。
内部 gate の項目名は人間向け質問本文に単独で出さない。
実装用語または内部設計語を出す必要がある場合は、`固定名（人間が判断できる説明）` の形で説明を添える。
説明を添えられない用語は、質問票から消さず `説明不足` として残す。
fixed decision で解ける内容は質問にしない。

`scenario-design.requirement-coverage.json` の `needs_human_decision` は、未回答一覧を作れる `question_id` を持つ。
質問票本文を同じ JSON へ持たせる場合は次を使ってよいが、判定条件 は必須項目として検査しない。

- `question_id`: `Q-001` 形式の連番
- `question_title`: 短い質問名
- `unresolved_decision`: 「決める仕様」に出す判断を 1 文で書く
- `premise`: 「決定済み」に出す、既に確定した仕様
- `undecided_reason`: 「未確定」に出す、まだ仕様として決まっていない点
- `options`: 3 件の選択肢と影響。選択肢は利用者または運用者から見える仕様差分だけを書く。`その他` は 判定条件 が 4 番として末尾に追加する
- `recommended_option`: AI 推奨の選択肢番号
- `recommendation_reason`: AI 推奨の理由を 2 文以内で書く
- `after_answer_generates`: 回答後に固定できる要求タイプまたは シナリオ

質問票では次を禁止する。

- `provider capability`、`phase resume boundary`、`job scoped phase run` のような内部設計語を説明なしで質問本文に出す
- gate の `detail_requirement_type` を質問分類として人間に見せる
- 1 つの質問に、削除、監査、履歴、復元、UI 表示を混ぜる
- ローカルアプリに監査要件がない場合、監査保持を選択肢へ入れる
- fixed decision で解ける内容を人間質問にする

`designer` が人間向け質問本文を書く場合、質問票の出力形式は次を固定形にする。

```markdown

### [Q-001] <短い質問名>

決める仕様:
<人間に決めてほしい判断>

決定済み:
<既に確定した仕様>

未確定:
<まだ仕様として決まっていない点>

選択肢:
1. <選択肢A>
2. <選択肢B>
3. <選択肢C>
4. その他

AI 推奨:
<選択肢番号と理由。理由は 2 文以内。>
```

## 判断規約

- 必ず通す要件を先に固定する
- シナリオ 候補母集団を `implement_lane` 由来の 6 種 候補成果物 から先に確認する
- 抽象要件を シナリオ へ進める前に、詳細要求タイプごとの明示状態を確認する
- 候補の採用、統合、不採用、競合、要人間判断を `scenario-design.candidate-coverage.json` に分ける
- 人間判断が必要な暗黙要求は `needs_human_decision` とし、質問票へ集約する
- 人間向け質問は内部 gate の項目名をそのまま使わず、仕様境界の判断に再編集する
- 仕様網羅 JSON は `scenario-design.md` に埋め込まず、`scenario-design.requirement-coverage.json` に分ける
- 質問票は `scenario-design.md` に埋め込まず、`scenario-design.questions.md` に分ける
- 未解決競合は シナリオ 完了にせず、`scenario-design.questions.md` へ集約する
- 実装方針の迷いは要件にせず リスク として管理する
- 有料の実AI API を システムテスト 前提にしない
- 正常系だけにしない
- 観測点がない シナリオ を書かない
- implementation-scope の承認済み実装範囲 を混ぜない
- 用語体系は `受け入れテスト > システムテスト > UI人間操作E2E / APIテスト` を正本にする
- `E2E` は UI 人間操作起点だけを指す
- `APIテスト` は 公開接点 起点の システムレベルテスト として扱う
- 受け入れテストは全 シナリオ case で先に固定する
- 各 シナリオ case に `実行テスト種別` と `実行段階` を必ず書く
- `実行テスト種別` は `APIテスト`、`UI人間操作E2E`、`lower-level only` だけを使う
- `実行段階` は `実装後`、`最終検証` だけを使う
- `APIテスト` では、受け入れ条件、公開接点 / API 境界、入力開始点、主要 結果、主要観測点、公開接点確認 の有無を固定する
- `UI人間操作E2E` では、開始操作、入力方法、主要操作列、主要観測点、UI-visible 結果、fake / stub 方針を固定する
- UI が入口の機能では、裏側の直接呼び出しや 検証データ 直接投入だけで成立するものを UI人間操作E2E と呼ばない

- 必ず通す要件と リスク を分ける
- `implement_lane` 由来の 候補成果物 を統合してから シナリオ表 を作る
- 詳細要求タイプの明示状態を シナリオ 前に確認する
- 候補網羅 と 競合 を JSON 別ファイルに分ける
- `needs_human_decision` は別ファイルの質問票に集約する
- 仕様網羅 JSON は別ファイルにし、Markdown 本文へ埋め込まない
- 再現可能な 検証データ と fake provider を優先する
- 受け入れ条件 と 検証 を結びつける
- 正本化 対象 を記録する
- `APIテスト` と `UI人間操作E2E` の必須情報を混同しない
- UI が入口の場合は、画面操作から得られる入力値を `UI人間操作E2E` の検証対象にする

## 非対象規約

- 人間判断が必要な暗黙要求や未解決競合を AI 判断で固定しない。
- `designer` から候補生成器を再起動しない。
- 実装方針、implementation-scope の承認済み実装範囲、プロダクトテスト実装詳細は扱わない。
- 有料の実API 前提や観測不能な期待結果は扱わない。
- 裏側の直接呼び出しだけの検証を UI 入口の `UI人間操作E2E` として扱わない。

## 出力規約

- 判断結果: シナリオ設計の完了、未完了、停止の判定を返す。
- 根拠参照: シナリオ設計の根拠にした要件、候補成果物、観測事実を返す。
- 不足情報: シナリオ設計を固定できない不足項目を返す。
- 次判断材料: `designer` または `implement_lane` が次を判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- task 内成果物 が承認状態、根拠参照、未決事項を含んでいる。
- 人間レビュー が必要な判断を AI だけで完了扱いにしていない。
- 必ず通す要件と リスク を分けた。
- 抽象要件を詳細要求タイプへ展開した。
- 各詳細要求タイプを `explicit`、`derived`、`not_applicable`、`deferred`、`needs_human_decision` に分類した。
- 仕様網羅を `scenario-design.requirement-coverage.json` に分離した。
- `needs_human_decision` だけを `scenario-design.questions.md` へ集約した。
- 利用者操作の流れ と シナリオ表 を分けた。
- 受け入れテストを全 シナリオ case で先に固定した。
- 各 シナリオ case に `実行テスト種別` と `実行段階` を書いた。
- 開始条件、操作、期待結果、観測点を明示した。
- `APIテスト` では受け入れ条件、公開接点、入力開始点、主要 結果、主要観測点、公開接点確認 を固定した。
- `UI人間操作E2E` では開始操作、入力方法、主要操作列、主要観測点、UI-visible 結果、fake / stub 方針を固定した。
- fake / 検証データ / 検証コマンド を確認した。

## 停止規約

- 人間判断が必要な暗黙要求を AI 判断で固定する必要がある場合は停止する。
- 未解決競合を AI 判断で解消する必要がある場合は停止する。
- `designer` から候補生成器を再起動する必要がある場合は停止する。
- 観測不能な期待結果を書く必要がある場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
