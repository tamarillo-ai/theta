---
name: light-change-planning
description: 軽量変更レーンで、人間要望、仕様製本、関連 docs、task-local 成果物、既存実装を突き合わせ、軽量変更として実装へ渡せるかを固定する。
---
# Light Change Planning

## 目的

`light-change-planning` は、軽量変更として実装へ進める前に、やりたいことと仕様製本、関連 docs、task-local 成果物、既存実装を突き合わせる作業プロトコルである。
`light_change_planner` が軽量変更計画を作る時に使う。

## 対応ロール

- `light_change_planner` が使う。
- 呼び出し元は `light_change_lane` とする。
- 返却先は `light_change_lane` とする。
- 担当成果物は `軽量変更計画` とする。

## 入力規約

- task 枠: 人間依頼、変更禁止範囲、確認したい結果。
- 作業計画フォルダ: task 内成果物を置く `docs/exec-plans/active/<task-id>/`。
- 既存成果物: 作業計画フォルダに既にある task 内成果物。
- 非必須検証ログ: 軽量変更に関係する既存の検証出力。

## 外部参照規約

- エージェント実行定義と実行境界は [light_change_planner.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/light_change_planner.toml) に従う。
- docs 入口は [docs/index.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/index.md) とする。
- 要件正本は [spec.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/spec.md) とする。
- architecture 正本は [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) とする。
- 変更対象に応じて [coding-guidelines-backend.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines-backend.md)、[coding-guidelines-frontend.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines-frontend.md)、[coding-guidelines-tests.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines-tests.md) を参照する。
- 関連する仕様製本は [detail-specs](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/detail-specs/README.md) から選ぶ。
- 関連する画面正本は [screen-design](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/screen-design/README.md) から選ぶ。
- 関連する scenario 正本は [scenario-tests](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/scenario-tests/README.md) から選ぶ。
- 軽量変更計画雛形は [light-change-planning.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/light-change-planning/assets/light-change-planning.md) とする。
- 外部成果物が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

軽量変更判定は次の表に従う。

| 判定 | 意味 | 返却先 |
| --- | --- | --- |
| `範囲内修正` | 仕様製本と既存実装の範囲内で期待状態へ寄せる変更 | `light_change_lane` |
| `軽量仕様変更` | 既存シナリオの意味を大きく広げず、表示、条件、文言、保存値、境界接続を少し変える変更 | `light_change_lane` |
| `設計戻し` | 新しいシナリオ、状態遷移、永続仕様、公開契約、外部連携判断が必要な変更 | `design-bundle` |
| `修正レーン戻し` | 人間観測、レビュー非通過、検証失敗を既存仕様へ戻す恒久修正 | `fix-lane` |

## 判断規約

- task 枠、仕様製本、関連 docs、task-local 成果物、既存実装を突き合わせる。
- 実装変更が backend、frontend、integration のどれに属するかを 1 つに固定する。
- backend と frontend を同時に触る必要がある場合は、統合境界変更として扱える時だけ軽量変更にする。
- 仕様製本と人間依頼が衝突する場合は、軽量変更として進めない。
- 承認済み task-local 成果物と人間依頼が衝突する場合は、軽量変更として進めない。
- 既存仕様へ戻す恒久修正として扱うべき場合は `修正レーン戻し` にする。
- 新しいシナリオ、状態遷移、永続仕様、公開契約、外部連携判断が必要な場合は `設計戻し` にする。
- 軽量変更として進める場合は、変更対象、禁止範囲、検証コマンド、読む docs を実装 agent が再解釈しない粒度で固定する。
- プロダクトコードとプロダクトテストは変更しない。

## 非対象規約

- プロダクトコード実装は扱わない。
- プロダクトテスト実装は扱わない。
- シナリオ候補生成、シナリオ設計、UI契約作成は扱わない。
- docs 正本化本文の更新は扱わない。
- 作業レポート本文の作成は扱わない。

## 出力規約

- 判断結果: `範囲内修正`、`軽量仕様変更`、`設計戻し`、`修正レーン戻し`、停止のいずれかを返す。
- 根拠参照: 判断に使った docs、task-local 成果物、既存実装を返す。
- 実装種別: `implement-backend`、`implement-frontend`、`implement-integration` のいずれかを返す。
- 変更対象: 変更してよいファイル、symbol、公開接点を返す。
- 禁止範囲: 変更してはいけない範囲を返す。
- 検証コマンド: 実装 agent が実行する局所検証を返す。
- 正本化判断材料: 仕様変更または仕様追加の有無を返す。
- 不足情報: 軽量変更計画を固定できない不足項目を返す。
- 戻し入力: `設計戻し` または `修正レーン戻し` の戻し先と理由を返す。
- 禁止事項: 出力にプロダクトコード、プロダクトテスト、docs 正本本文の変更を含めない。

## 完了規約

- 判断結果が根拠参照付きで固定されている。
- 仕様製本、関連 docs、task-local 成果物、既存実装を確認した。
- 軽量変更として進む場合は、実装種別、変更対象、禁止範囲、検証コマンドが固定されている。
- 仕様変更または仕様追加の有無が固定されている。
- `設計戻し` または `修正レーン戻し` の場合は、戻し先と理由が固定されている。

## 停止規約

- task 枠が不足する場合は停止する。
- 仕様製本、関連 docs、task-local 成果物、既存実装を確認できない場合は停止する。
- 判断結果を固定できない場合は停止する。
- 実装種別を 1 つに固定できない場合は停止する。
- 変更対象または禁止範囲を固定できない場合は停止する。
- 検証コマンドを固定できない場合は停止する。
- プロダクトコード、プロダクトテスト、docs 正本本文を変更しそうな場合は停止する。
- 停止時は不足項目、衝突箇所、固定できない判断、戻し先を返す。
