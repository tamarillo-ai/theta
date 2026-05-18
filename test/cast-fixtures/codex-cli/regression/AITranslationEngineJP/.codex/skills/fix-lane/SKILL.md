---
name: fix-lane
description: 人間が確認した不具合、レビュー非通過、検証失敗の恒久修正レーンを固定する作業プロトコル。
---
# Fix Lane

## 目的

`fix-lane` は、人間が確認した不具合、レビュー非通過、検証失敗を恒久修正へ渡す進行判断を task 内成果物DAG と起動入力へ固定する作業プロトコルである。
`fix_lane` が人間観測記録、修正前調査、修正実行入力、実装証跡、回帰確認、レビュー通過根拠を管理する時に使う。
`fix_lane` は担当 agent を起動し、各 agent の完了結果を集約する。

## 対応ロール

- `fix_lane` が使う。
- 呼び出し元は人間とする。
- 返却先は人間とする。
- 担当成果物は `人間観測記録`、`原因箇所シーケンス図`、`修正実行入力`、`実装後ブラウザ確認`、`レビュー通過根拠`、`作業レポート入力`、`作業計画完了移動` とする。
- 起動担当 agent は `investigator`、`diagrammer`、`backend_implementer`、`frontend_implementer`、`integration_implementer`、`implementation_scenario_tester`、`implementation_unit_tester`、`browser_confirmation`、観点別レビュー agent、`work_reporter` とする。

## 入力規約

- 呼び出し元: この skill を呼び出した人間または戻し元。
- 依頼要約: 修正対象として扱う観測内容。
- 作業計画フォルダ: task 内成果物を置く `docs/exec-plans/active/<task-id>/`。
- 既存成果物: 作業計画フォルダに既にある task 内成果物。
- 人間観測: 人間が見た画面、操作、ログ、失敗、期待との差分。
- 既存レビューYAML: 非必須入力として受け取る修正対象に関係する既存のレビュー結果。
- 検証ログ: 非必須入力として受け取る修正対象に関係する既存の検証出力。
- 探索証跡: 非必須入力として受け取る修正対象に関係する既存の探索テスト観測結果。
- 影響ファイル一覧: 非必須入力として受け取る修正対象に関係する既存の影響ファイル候補。

## 外部参照規約

- エージェント実行定義と実行境界は [fix_lane.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/fix_lane.toml) に従う。
- 修正前調査は [investigate](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/investigate/SKILL.md) に従う。
- 原因箇所シーケンス図は [diagramming](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/diagramming/SKILL.md) に従う。
- プロダクトコード実装は `implement-backend`、`implement-frontend`、`implement-integration` のいずれかに従う。
- 回帰テスト証跡は `tests-scenario` または `tests-unit` に従う。
- 実装後ブラウザ確認は [browser-confirmation](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/browser-confirmation/SKILL.md) に従う。
- 観点別レビューは `codex-review-behavior`、`codex-review-contract`、`codex-review-trust-boundary`、`codex-review-state-invariant`、`codex-review-responsibility-boundary` に従う。
- 外部成果物が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

修正レーンの成果物DAGは次を必ず持つ。
各成果物は、`依存対象` の成果物が揃った時だけ着手できる。

| 成果物ID | 担当者 | 依存対象 | 次 agent |
| --- | --- | --- | --- |
| `人間観測記録` | `fix_lane` | `task 枠` | なし |
| `修正前調査` | `investigator` | `人間観測記録` | `investigator` |
| `原因箇所シーケンス図` | `diagrammer` | `人間観測記録`, `修正前調査` | `diagrammer` |
| `修正実行入力` | `fix_lane` | `人間観測記録`, `修正前調査`, `原因箇所シーケンス図` | なし |
| `実装証跡` | 実装種別別 agent / `implement-backend` または `implement-frontend` または `implement-integration` | `修正実行入力` | `backend_implementer` または `frontend_implementer` または `integration_implementer` |
| `回帰テスト証跡` | `implementation_scenario_tester` または `implementation_unit_tester` | `実装証跡` | `implementation_scenario_tester` または `implementation_unit_tester` |
| `実装後ブラウザ確認` | `browser_confirmation` | `実装証跡`, `回帰テスト証跡?` | `browser_confirmation` |
| `レビュー通過根拠` | `fix_lane` | `人間観測記録`, `修正前調査`, `原因箇所シーケンス図`, `修正実行入力`, `実装証跡`, `回帰テスト証跡?`, `実装後ブラウザ確認` | `review_behavior`, `review_contract`, `review_trust_boundary`, `review_state_invariant`, `review_responsibility_boundary` |
| `作業レポート入力` | `fix_lane` / `work_reporter` | 全完了または停止済み成果物, `レビュー通過根拠?` | `work_reporter` |
| `作業計画完了移動` | `fix_lane` | `作業レポート入力` | なし |

## 判断規約

- 人間観測は探索テストの探索範囲拡張ではなく、修正入口の根拠として扱う。
- `修正前調査` は `investigator` を起動して渡す。
- `原因箇所シーケンス図` は `diagrammer` を起動して作る。
- `原因箇所シーケンス図` は修正前調査で確認した原因箇所の呼び出し順序だけを示す。
- `原因箇所シーケンス図` は何が問題か、どう直すかを説明する。
- `原因箇所シーケンス図` は全体シーケンス、推測原因、未確認の修正案を含めない。
- `修正実行入力` は人間観測記録、修正前調査、原因箇所シーケンス図を根拠にする。
- `修正実行入力` は影響ファイル候補、禁止変更範囲、実装 skill、回帰確認観点を分ける。
- `fix_lane` は新規実装レーン用の `implementation-scope` を作らない。
- 原因が未確認の場合は、恒久修正へ進めず、不足項目と戻し先を返す。
- 大規模修正は、既存仕様へ戻す修正として始まったが、期待状態、回帰確認、修正方針を既存根拠だけで固定できない状態とする。
- 仕様変更、機能追加、受け入れ条件の新規判断が必要な場合は、修正レーン対象外として扱う。
- 大規模修正または修正レーン対象外の場合は、恒久修正へ進めず、固定できない判断、戻し先、`implement-lane` 用タスクプロンプト案を返す。
- 実装 agent を起動する時は、`backend_implementer`、`frontend_implementer`、`integration_implementer` のいずれか 1 つに固定する。
- 回帰テスト証跡は変更範囲と検証目的から `implementation_scenario_tester` または `implementation_unit_tester` を起動して渡す。
- `実装後ブラウザ確認` の確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先は、人間観測、修正前調査、原因箇所シーケンス図、修正実行入力から `fix_lane` が定義する。
- `browser_confirmation` は `実装後ブラウザ確認` の実行だけを担当し、期待値の妥当性を判断しない。
- レビュー通過根拠は人間観測記録、修正前調査、原因箇所シーケンス図、修正実行入力、実装証跡、回帰テスト証跡、実装後ブラウザ確認を入力にして観点別レビュー agent を起動する。
- 作業レポート入力は `work_reporter` を起動して渡す。
- プロダクトコードとプロダクトテストは変更しない。

## 非対象規約

- 新規実装と機能拡張は扱わない。
- 探索テストの計画と観測は扱わない。
- 修正前調査の実施は扱わない。
- 直接のプロダクトコード実装は扱わない。
- 直接のプロダクトテスト実装は扱わない。
- 観点別レビューの実施は扱わない。
- 作業レポート本文の作成は扱わない。
- docs 正本化本文の更新は扱わない。
- task folder の状態更新以外の docs 更新は扱わない。

## 出力規約

- 人間向け返却: 成果物DAGの現在成果物、着手可能成果物、停止中成果物、停止理由を返す。
- 起動先向け返却: 起動先 agent 向けに対象成果物、満たされた `依存対象`、読むファイル、禁止事項、期待する成果物を返す。
- 人間観測記録: 人間が見た画面、操作、ログ、失敗、期待との差分を返す。
- 修正前調査起動入力: `investigator` 向けに人間観測記録、既存レビューYAML、検証ログ、禁止事項、期待する成果物を返す。
- 原因箇所シーケンス図起動入力: `diagrammer` 向けに人間観測記録、修正前調査、原因箇所、問題点、修正方針、禁止範囲、対象作業計画フォルダを返す。
- 原因箇所シーケンス図: 修正前判断材料として、原因箇所のシーケンス図、問題点、修正方針、根拠参照、検証結果、未決事項を返す。
- 修正実行入力: 実装種別別 agent 向けに人間観測記録、修正前調査、原因箇所シーケンス図、影響ファイル候補、禁止変更範囲、実装 skill、回帰確認観点を返す。
- 実装後ブラウザ確認起動入力: `browser_confirmation` 向けに確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先を返す。
- 実装後ブラウザ確認: 操作確認結果、証跡参照、console または network 異常、未確認理由、戻し先を返す。
- レーン戻し入力: 大規模修正または修正レーン対象外の場合に、固定できない判断、戻し先、`implement-lane` 用タスクプロンプト案を返す。
- レビュー起動入力: レビュー agent 向けに人間観測記録、修正前調査、原因箇所シーケンス図、修正実行入力、実装証跡、回帰テスト証跡、レビューYAMLパスを返す。
- 作業レポート入力: 完了または停止した成果物、検証、残留リスク、次に見るべき場所を返す。
- 作業計画完了移動: 作業計画フォルダを `docs/exec-plans/completed/<task-id>/` へ移動した根拠を返す。
- 禁止事項: 出力にプロダクトコード、プロダクトテスト、docs 正本本文の変更を含めない。

## 完了規約

- 修正レーンの次成果物、起動、停止、戻しを再解釈なしで判断できる。
- 人間観測記録、修正前調査、原因箇所シーケンス図、修正実行入力、実装証跡が根拠参照付きで確認されている。
- `原因箇所シーケンス図` が修正着手前に揃っている。
- `原因箇所シーケンス図` が原因箇所の呼び出し順序、問題点、修正方針を含んでいる。
- 実装 agent と実装 skill が `backend_implementer` / `implement-backend`、`frontend_implementer` / `implement-frontend`、`integration_implementer` / `implement-integration` のいずれか 1 組に固定されている。
- 回帰テスト証跡が必要な場合は、test agent の完了結果が確認されている。
- `実装後ブラウザ確認` が確認 URL、操作経路、操作期待値、証跡参照、未確認理由を含んでいる。
- 5 観点の `reviewback.<観点>.yaml` が確認されている。
- 終了処理、停止、戻しのいずれでも `作業レポート入力` と作業観測根拠が作成されている。
- close 時は作業計画フォルダが `docs/exec-plans/completed/<task-id>/` へ移動済みである。

## 停止規約

- 依頼が修正レーン対象か判断できない場合は停止する。
- 人間観測、レビュー非通過、検証失敗の根拠がない場合は停止する。
- 原因が未確認なのに恒久修正へ進みそうな場合は停止する。
- 修正前調査なしで修正実行入力へ進みそうな場合は停止する。
- `原因箇所シーケンス図` なしで修正実行入力へ進みそうな場合は停止する。
- `原因箇所シーケンス図` に原因未確認の推測または未確認の修正案が含まれる場合は停止する。
- 修正実行入力を固定できない場合は停止する。
- 大規模修正なのに恒久修正へ進みそうな場合は停止する。
- 仕様変更、機能追加、受け入れ条件の新規判断が必要な場合は停止する。
- 修正レーンで `implementation-scope` を作りそうな場合は停止する。
- 実装 skill を 1 つに固定できない場合は停止する。
- `実装後ブラウザ確認` の確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先が不足する場合は停止する。
- `実装後ブラウザ確認` なしで `レビュー通過根拠` へ進みそうな場合は停止する。
- プロダクトコードまたはプロダクトテストを直接変更しそうな場合は停止する。
- レビュー agent 起動入力に人間観測記録、修正前調査、原因箇所シーケンス図、修正実行入力、実装証跡、回帰テスト証跡の必要分が不足する場合は停止する。
- 停止時は不足項目、衝突箇所、固定できない判断、戻し先を返す。
