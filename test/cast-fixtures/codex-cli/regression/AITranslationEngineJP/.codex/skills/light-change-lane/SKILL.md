---
name: light-change-lane
description: 軽量変更レーンの成果物DAG、起動入力、軽い設計、実装、テスト追従、レビュー、終了条件を固定する作業プロトコル。
---
# Light Change Lane

## 目的

`light-change-lane` は、既存仕様の意味を大きく広げない軽量変更を進める作業プロトコルである。
`light_change_lane` が task 枠、軽量変更計画、実装証跡、確認結果、テスト修正証跡、レビュー通過根拠、作業レポート入力を管理する時に使う。

## 対応ロール

- `light_change_lane` が使う。
- 呼び出し元は人間とする。
- 返却先は人間とする。
- 担当成果物は `task 枠`、`軽量変更計画`、`設計差分図`、`実装証跡`、`人間確認`、`テスト修正証跡`、`実装後ブラウザ確認`、`レビュー通過根拠`、`正本化判断`、`作業レポート入力`、`作業計画完了移動` とする。
- 起動担当 agent は `light_change_planner`、`diagrammer`、`backend_implementer`、`frontend_implementer`、`integration_implementer`、`implementation_scenario_tester`、`implementation_unit_tester`、`browser_confirmation`、観点別レビュー agent、`docs_updater`、`work_reporter` とする。

## 入力規約

- 呼び出し元: この skill を呼び出した人間または戻し元。
- 依頼要約: 軽量変更として扱う依頼内容。
- 作業計画フォルダ: task 内成果物を置く `docs/exec-plans/active/<task-id>/`。
- 既存成果物: 作業計画フォルダに既にある task 内成果物。
- 人間介入状態: 人間確認、承認、差し戻し、追加質問の記録。
- 非必須検証ログ: 軽量変更に関係する既存の検証出力。

## 外部参照規約

- エージェント実行定義と実行境界は [light_change_lane.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/light_change_lane.toml) に従う。
- 軽量変更計画は [light-change-planning](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/light-change-planning/SKILL.md) に従う。
- 設計差分図は [diagramming](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/diagramming/SKILL.md) に従う。
- プロダクト実装は `implement-backend`、`implement-frontend`、`implement-integration` のいずれかに従う。
- シナリオテスト修正は [tests-scenario](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/tests-scenario/SKILL.md) に従う。
- 単体テスト修正は [tests-unit](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/tests-unit/SKILL.md) に従う。
- 実装後ブラウザ確認は [browser-confirmation](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/browser-confirmation/SKILL.md) に従う。
- 観点別レビューは `codex-review-behavior`、`codex-review-contract`、`codex-review-trust-boundary`、`codex-review-state-invariant`、`codex-review-responsibility-boundary` に従う。
- docs 正本化は [updating-docs](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/updating-docs/SKILL.md) に従う。
- 外部成果物が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

軽量変更レーンの成果物DAGは次を必ず持つ。
各成果物は、`依存対象` の成果物が揃った時だけ着手できる。
`次 agent` は、その成果物を揃えるために引き継ぎ入力を渡す相手を示す。

| 成果物ID | 担当者 | 依存対象 | 次 agent |
| --- | --- | --- | --- |
| `task 枠` | `light_change_lane` | `[]` | なし |
| `軽量変更計画` | `light_change_planner` | `task 枠` | `light_change_planner` |
| `設計差分図` | `diagrammer` | `軽量変更計画` | `diagrammer` |
| `実装証跡` | 実装種別別 agent / `implement-backend` または `implement-frontend` または `implement-integration` | `軽量変更計画`, `設計差分図` | `backend_implementer` または `frontend_implementer` または `integration_implementer` |
| `人間確認` | 人間 | `実装証跡` | 人間 |
| `テスト修正証跡` | `implementation_scenario_tester` または `implementation_unit_tester` | `実装証跡`, `人間確認?` | `implementation_scenario_tester` または `implementation_unit_tester` |
| `実装後ブラウザ確認` | `browser_confirmation` | `実装証跡`, `人間確認?`, `テスト修正証跡?` | `browser_confirmation` |
| `レビュー通過根拠` | `light_change_lane` | `軽量変更計画`, `実装証跡`, `人間確認?`, `テスト修正証跡?`, `実装後ブラウザ確認` | `review_behavior`, `review_contract`, `review_trust_boundary`, `review_state_invariant`, `review_responsibility_boundary` |
| `正本化判断` | 仕様変更または仕様追加あり | `レビュー通過根拠` | `docs_updater?` |
| `詳細仕様正本反映` | 仕様変更または仕様追加あり | `正本化判断` | `docs_updater?` |
| `作業レポート入力` | `light_change_lane` / `work_reporter` | 全完了または停止済み成果物, `レビュー通過根拠?` | `work_reporter` |
| `作業計画完了移動` | `light_change_lane` | `作業レポート入力` | なし |

### 軽量変更分類

| 分類 | 意味 | 扱い |
| --- | --- | --- |
| `範囲内修正` | 仕様製本と既存実装の範囲内で期待状態へ寄せる変更 | 軽量変更として進める |
| `軽量仕様変更` | 既存シナリオの意味を大きく広げず、表示、条件、文言、保存値、境界接続を少し変える変更 | 軽量変更として進める |
| `設計戻し` | 新しいシナリオ、状態遷移、永続仕様、公開契約、外部連携判断が必要な変更 | `design-bundle` へ戻す |
| `修正レーン戻し` | 人間観測、レビュー非通過、検証失敗を既存仕様へ戻す恒久修正 | `fix-lane` へ戻す |

## 判断規約

- 次の実行判断は成果物DAGの未完了成果物、満たされた `依存対象`、既存成果物、対象 skill の完了規約で決める。
- `task 枠` は人間依頼、変更禁止範囲、確認したい結果を含める。
- `軽量変更計画` は `light_change_planner` を起動して渡す。
- `軽量変更計画` が `設計戻し` または `修正レーン戻し` を返す場合は、実装へ進めない。
- `設計差分図` は `diagrammer` を起動して作る。
- `設計差分図` は、予定変更箇所だけの追加・削除差分を示す コンポーネント図 と シーケンス図 に限定する。
- `設計差分図` は、全体構成図、正本図、変更しない箇所の網羅図として作らない。
- `設計差分図` の起動入力には、軽量変更計画、予定変更箇所、追加予定箇所、削除予定箇所、禁止範囲、出力先を含める。
- 実装 agent の起動入力には、`backend_implementer`、`frontend_implementer`、`integration_implementer` のどれを起動するかを必ず明示する。
- backend、frontend、統合境界のどれか 1 つの実装 skill に固定できない変更は、軽量変更として扱わない。
- backend と frontend を同時に触る必要がある場合は、統合境界変更として扱える時だけ進める。
- `人間確認` は実物確認、操作、表示、出力、状態変化、検証結果の確認を扱う。
- `テスト修正証跡` は軽量変更で落ちるシナリオテストまたは単体テストだけを扱う。
- `実装後ブラウザ確認` の確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先は、task 枠、軽量変更計画、実装証跡から `light_change_lane` が定義する。
- `browser_confirmation` は `実装後ブラウザ確認` の実行だけを担当し、期待値の妥当性を判断しない。
- `レビュー通過根拠` は軽量変更計画、実装証跡、人間確認、テスト修正証跡、実装後ブラウザ確認を入力にして観点別レビュー agent を起動する。
- 仕様変更または仕様追加が少しでもある場合は `正本化判断` を必須成果物にする。
- human 承認済みの恒久仕様がある場合は `詳細仕様正本反映` を必須成果物にする。
- 起動先 agent には文脈を引き継がず、必要情報を引き継ぎ入力に明示する。
- 起動先 agent は下位 agent を起動せず、渡された成果物だけを作る。
- 人間介入が必要な成果物は AI だけで完了にしない。
- プロダクトコードとプロダクトテストは変更しない。

## 非対象規約

- 新規実装と機能拡張の初期設計は扱わない。
- 既存仕様へ戻す恒久修正は扱わない。
- 探索テストの計画と観測は扱わない。
- 重い構造整理は扱わない。
- シナリオ候補生成、シナリオ設計、UI契約作成は扱わない。
- 起動先 agent の下位 agent 起動は扱わない。
- 直接のプロダクトコード実装は扱わない。
- 直接のプロダクトテスト実装は扱わない。
- docs 正本化本文の更新は扱わない。

## 出力規約

- 人間向け返却: 成果物DAGの現在成果物、着手可能成果物、停止中成果物、停止理由を返す。
- 起動先向け返却: 起動先 agent 向けに対象成果物、満たされた `依存対象`、読むファイル、禁止事項、期待する成果物を返す。
- task 枠: 人間依頼、変更禁止範囲、確認したい結果を返す。
- 軽量変更計画起動入力: `light_change_planner` 向けに task 枠、既存成果物、非必須検証ログ、禁止事項、期待する成果物を返す。
- 設計差分図起動入力: `diagrammer` 向けに図化目的、軽量変更計画、予定変更箇所、追加予定箇所、削除予定箇所、禁止範囲、対象作業計画フォルダを返す。
- 設計差分図: 人間確認前の判断材料として、追加・削除差分のコンポーネント図、追加・削除差分のシーケンス図、根拠参照、検証結果、未決事項を返す。
- 実装起動入力: 実装種別別 agent 向けに軽量変更計画、実装 skill、変更対象、検証コマンド、停止条件を返す。
- 人間確認記録: 人間確認の承認、差し戻し、追加質問、確認根拠を返す。
- テスト修正起動入力: テスト修正担当 agent 向けに対象テスト範囲、検証目的、実装結果、人間確認結果、検証コマンド、停止条件を返す。
- 実装後ブラウザ確認起動入力: `browser_confirmation` 向けに確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先を返す。
- 実装後ブラウザ確認: 操作確認結果、証跡参照、console または network 異常、未確認理由、戻し先を返す。
- レビュー起動入力: レビュー agent 向けにレビュー対象差分、実装目的、軽量変更計画、実装結果、検証証跡、変更ファイル、レビューYAMLパスを返す。
- 作業レポート入力: 完了または停止した成果物、検証、残留リスク、次に見るべき場所を返す。
- 作業計画完了移動: 作業計画フォルダを `docs/exec-plans/completed/<task-id>/` へ移動した根拠を返す。
- 禁止事項: 出力にプロダクトコード、プロダクトテスト、docs 正本本文の変更を含めない。

## 完了規約

- 軽量変更レーンの次成果物、起動、人間確認、停止、戻しを再解釈なしで判断できる。
- `task 枠` が人間依頼、変更禁止範囲、確認したい結果を含んでいる。
- `軽量変更計画` が仕様製本、関連 docs、task-local 成果物、既存実装の突き合わせ結果を含んでいる。
- `軽量変更計画` が `範囲内修正`、`軽量仕様変更`、`設計戻し`、`修正レーン戻し` のいずれかを返している。
- `設計差分図` が実装着手前に揃っている。
- `設計差分図` が予定変更箇所だけの追加・削除差分を示す コンポーネント図 と シーケンス図 を含んでいる。
- `実装証跡` が軽量変更計画、禁止範囲、実装 skill、確認観点を根拠に起動されている。
- 起動先 agent が文脈継承なしで直接起動され、起動入力だけで成果物を返している。
- 人間確認が必要な場合は、承認、差し戻し、追加質問のいずれかが記録されている。
- テスト追従が必要な場合は、`テスト修正証跡` が記録されている。
- `実装後ブラウザ確認` が確認 URL、操作経路、操作期待値、証跡参照、未確認理由を含んでいる。
- 5 観点の `reviewback.<観点>.yaml` が確認されている。
- 仕様変更または仕様追加がある場合は、`正本化判断` の結果が根拠参照付きで記録されている。
- human 承認済みの恒久仕様がある場合は、`詳細仕様正本反映` の完了結果または停止理由が根拠参照付きで記録されている。
- 終了処理、停止、戻しのいずれでも `作業レポート入力` と作業観測根拠が作成されている。
- close 時は作業計画フォルダが `docs/exec-plans/completed/<task-id>/` へ移動済みである。

## 停止規約

- 依頼が軽量変更か判断できない場合は停止する。
- task 枠なしで軽量変更計画へ進みそうな場合は停止する。
- 軽量変更計画なしで実装へ進みそうな場合は停止する。
- `light_change_planner` の判定が `設計戻し` または `修正レーン戻し` の場合は停止する。
- `設計差分図` なしで `実装証跡` へ進みそうな場合は停止する。
- `設計差分図` が予定変更箇所以外を網羅図として含む場合は停止する。
- 実装 skill を 1 つに固定できない場合は停止する。
- 新しいシナリオ、状態遷移、永続仕様、公開契約、外部連携判断が必要な場合は停止する。
- テスト修正に必要な対象テスト範囲、検証目的、検証コマンドが不足する場合は停止する。
- テスト修正に軽量変更計画外のプロダクトコード変更または仕様変更が必要な場合は停止する。
- 起動先 agent に文脈継承または下位 agent 起動が必要な場合は停止する。
- `実装後ブラウザ確認` の確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先が不足する場合は停止する。
- `実装後ブラウザ確認` なしで `レビュー通過根拠` へ進みそうな場合は停止する。
- プロダクトコードまたはプロダクトテストを直接変更しそうな場合は停止する。
- レビュー agent 起動入力に実装結果、検証証跡、変更ファイル、レビューYAMLパスが不足する場合は停止する。
- 仕様変更または仕様追加があるのに `正本化判断` が不足する場合は終了不可とする。
- human 承認済みの恒久仕様があるのに `詳細仕様正本反映` が不足する場合は終了不可とする。
- 停止時は不足項目、衝突箇所、固定できない判断、戻し先を返す。
