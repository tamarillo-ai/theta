---
name: design-bundle
description: Codex 側の 設計成果物 進行 skill。シナリオ、独立 UI 設計、implementation-scope を task 内成果物 として固定するための 正本、判断、引き継ぎ を提供する。
---
# Design Bundle

## 目的

`design-bundle` は作業プロトコルである。
`designer` agent と Codex 本体が、シナリオ、独立 UI 設計、implementation-scope を task 内成果物 として固定する時の、人間可読な実行説明の正本として使う。

作業流れ の次 実行判断、作業計画フォルダ 進行管理、人間向け Codex implementation レーン 引き継ぎ の返却は `implement_lane` が担当する。
プロダクトコードとプロダクトテスト は変更しない。

## 対応ロール

- `designer` が使う。
- 呼び出し元は `implement_lane` または人間とする。
- 返却先は 人間レビュー または `implement_lane` とする。
- 担当成果物は `design-bundle` の出力規約で固定する。

## 入力規約

- 呼び出し元: `implement_lane` または人間を受け取る。
- 引き継ぎ入力: 呼び出し元が渡す設計対象と根拠参照。
- 依頼要約: 新規実装または機能拡張として扱う依頼内容。
- 作業計画フォルダ: task 内成果物を置く `docs/exec-plans/active/<task-id>/`。
- 設計範囲: 設計成果物として固定する対象範囲。
- 非必須入力: 人間レビュー記録、対象 skill、既存設計成果物、シナリオ候補成果物、既知不足を受け取る。
- 必須成果物: `/Users/iorishibata/Repositories/AITranslationEngineJP/docs/index.md` と 作業計画フォルダ を受け取る。
- 文脈独立条件: 引き継ぎ入力だけで作業でき、引き継いでいない会話文脈に依存しない。

## 外部参照規約

- エージェント実行定義と実行境界は [designer.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/designer.toml) に従う。
- 要件正本: [spec.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/spec.md) とする。
- architecture 正本: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) とする。
- ER 正本: [er.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/er.md) と [diagrams/er](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/diagrams/er/) とする。
- 画面正本: [screen-design](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/screen-design/README.md) とする。
- 上位シナリオ詳細仕様正本: [detail-specs](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/detail-specs/README.md) とする。
- scenario 正本: [scenario-tests](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/scenario-tests/README.md) とする。
- 補助参照: 入力に明示された関連 docs、関連 skill、人間の現在指示
- エージェント実行定義: [designer.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/designer.toml)
- 実行境界: エージェント実行定義に従う
- ui: [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/ui-design/SKILL.md)
- 候補観点: [actor-goal](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-actor-goal-generation/SKILL.md)、[lifecycle](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-lifecycle-generation/SKILL.md)、[state-transition](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-state-transition-generation/SKILL.md)、[失敗](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-failure-generation/SKILL.md)、[external-integration](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-external-integration-generation/SKILL.md)、[operation-audit](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-operation-audit-generation/SKILL.md)
- シナリオ: [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-design/SKILL.md)
- implementation 対象範囲: [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-scope/SKILL.md)
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

### 実装対象範囲判定条件

implementation-scope を扱う時は、Codex 実装レーンの agent 起動 のトークン量を事前計算しない。
代わりに [implementation-scope](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-scope/SKILL.md) の 引き継ぎ分割規約 と 規模判定条件 に従い、論理境界と規模の目安で分割する。

各 引き継ぎ は原則として `1 受け入れユースケース × 1 検証意図` に収める。
Codex implementation レーンから 対象範囲 過大で 戻し された場合は、既存 承認 を維持せず `pending-human-review` に戻す。

### シナリオ完備判定条件

scenario-design は、抽象要件から直接 シナリオ を作って完了にしない。
`designer` は `implement_lane` が揃えた 6 種の `scenario-candidates.<viewpoint>.md` を読み、候補の重複、採用、統合、不採用、競合を固定してから シナリオ表 を作る。
`designer` は候補生成器を再 起動 しない。
[scenario-design](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-design/SKILL.md) の詳細要求タイプを使い、明示的ではない判断を先に検出する。
詳細要求タイプの仕様網羅は `scenario-design.requirement-coverage.json` に分ける。
シナリオ 候補の採否と競合は `scenario-design.candidate-coverage.json` に分ける。
人間向け質問票は `scenario-design.questions.md` に分ける。
`scenario-design.md` に長い JSON や質問票本文を埋め込まない。

### 人間向け質問票の作成条件

`designer` は `needs_human_decision` をそのまま質問票へ転記しない。
質問票は、人間が決める必要のある仕様境界を扱う。
質問は「どの仕様を決めればよいか」を 1 文で先に書く。
選択肢は利用者または運用者から見える仕様差分にし、実装方式を混ぜない。
監査、履歴、外部連携など、未要求の観点を質問に混ぜない。
既に fixed decision で決められる内容は質問にしない。
内部 gate の項目名は人間向け質問本文に単独で出さない。
実装用語または内部設計語を出す必要がある場合は、`固定名（人間が判断できる説明）` の形で説明を添える。
説明を添えられない用語は、質問票から消さず `説明不足` として残す。
repo-local gate が出す質問票は未回答 ID 一覧にとどめる。
人間向け質問本文は `designer` が同じ正本へ再編集する。

質問票は次の項目だけを持つ。

- `決める仕様`: 人間が確定する仕様境界を 1 文で書く。
- `決定済み`: 既に確定した仕様だけを書く。
- `未確定`: まだ仕様として決まっていない点を書く。
- `選択肢`: 利用者または運用者から見える仕様差分だけを書く。
- `AI 推奨`: 1 つだけ書き、理由は 2 文以内にする。

質問票では次を禁止する。

- `provider capability`、`phase resume boundary`、`job scoped phase run` のような内部設計語を説明なしで質問本文に出す。
- gate の `detail_requirement_type` を質問分類として人間に見せる。
- 1 つの質問に、削除、監査、履歴、復元、UI 表示を混ぜる。
- ローカルアプリに監査要件がない場合、監査保持を選択肢へ入れる。
- fixed decision で解ける内容を人間質問にする。

シナリオ設計を 人間レビュー へ進める条件は次の通り。

- 必要な詳細要求タイプが `explicit`、`derived`、`not_applicable`、`deferred` のいずれかに分類されている
- 6 種の シナリオ 候補成果物 が作業計画フォルダに存在する
- `scenario-design.candidate-coverage.json` で全 候補 の採用、統合、不採用、競合、要人間判断が分類されている
- `not_applicable` と `deferred` には理由がある
- `needs_human_decision` が 0 件である
- 未解決競合が 0 件である
- 人間判断が必要な項目がある場合は、シナリオ 完了ではなく `scenario-design.questions.md` 出力で停止している

### UI 設計分離条件

UI 設計は design bundle 本体へ含めず、`ui-design` の独立成果物として扱う。
UI が関係する task では、`ui-design.md` を人間レビュー前に揃える。
frontend 実装がある task では、承認済み UI 設計成果物を implementation-scope と frontend 実装の根拠にする。

## 判断規約

- 判断は入力 成果物、外部参照規約、対象 agent の責務境界に従う。

## 非対象規約

- 作業流れ順序決定、作業計画フォルダ進行管理、作業前確認は扱わない。
- `ui-design` の実画面確認を除く 実画面 observation、docs 正本化、プロダクト実装は扱わない。
- ツール権限、agent 実行定義、プロダクト仕様正本は変更しない。

## 出力規約

- 判断結果: 設計成果物の完了、未完了、停止の判定を返す。
- 引き継ぎ先: `implement_lane` を返す。
- 渡す対象範囲: 設計成果物、人間レビュー 状態、未回答質問を返す。
- 返却先: `implement_lane` を返す。
- 対象成果物: 扱った シナリオ、シナリオ候補 統合、独立 UI 設計、implementation-scope の状態を返す。
- 変更成果物: 作成または更新した task 内成果物パスを返す。
- 人間レビュー状態: 人間レビュー が必要な判断、承認待ち、承認済みの状態を返す。
- 確認結果: 実行した確認と未実行理由を返す。
- 引き継ぎまたは停止理由: `implement_lane` へ戻す理由または停止理由を返す。
- 未決事項: 設計継続に必要な未決事項を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- task 内成果物 が承認状態、根拠参照、未決事項を含んでいる。
- 人間レビュー が必要な判断を AI だけで完了扱いにしていない。
- 必須根拠として、根拠成果物パス、必要な 人間承認 記録、実行した 検証結果 がある。
- 完了判断材料として、`implement_lane` が次の 作業流れ 実行判断、人間レビュー、人間向け Codex implementation レーン 引き継ぎ を判断できる情報が返っている。
- 残留リスクとして、設計継続に必要な未決事項が返っている。

## 停止規約

- scenario-design に `needs_human_decision` または未解決競合が残る場合は、質問票を返して人間回答待ちにする。
- シナリオ 候補成果物 が不足する場合は、`implement_lane` に戻し、候補生成器の不足を解消してから再開する。
- 作業流れ 順序決定 や 作業計画フォルダ進行管理 が主目的なら `implement_lane` へ戻す。
- 作業前の影響範囲、実行計画、検証方法の確認が不足する場合は `implement_lane` へ戻す。
- `ui-design` の範囲外で実画面 observation が必要なら `investigator` を使う前提で `implement_lane` へ戻す。
- docs 正本化が必要なら人間承認後に `docs_updater` を使う前提で `implement_lane` へ戻す。
- プロダクト 実装が必要なら `implement_lane` へ戻し、人間向け Codex implementation レーン 引き継ぎ の扱いを判断させる。
- 停止時は不足項目、衝突箇所、戻し先を返す。
- 作業流れの進行管理要求は停止する。
- 不足 引き継ぎ入力では停止する。
- プロダクト実装依頼では停止する。
- 未承認 docs 正本化では停止する。
- 実装レーン責務の実装時作業では停止する。
- 人間レビュー が必要な判断を AI だけで確定しそうな場合は停止する。
- scenario-design に必要な シナリオ候補成果物 が不足する場合は停止する。
- シナリオ候補網羅に未解決競合が残る場合は停止する。
- 作業計画フォルダが不足する場合は停止する。
- 設計範囲 が不明な場合は停止する。
- 引き継ぎ入力 だけでは作業できない場合は停止する。
- プロダクト 実装が必要な場合は停止する。
