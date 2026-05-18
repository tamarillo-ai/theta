---
name: updating-docs
description: Codex 側の docs 正本化作業プロトコル。implementation 完了後に、人間承認済み docs-only 成果物 を正本へ反映する判断基準を提供する。
---
# Updating Docs

## 目的

`updating-docs` は作業プロトコルである。
`docs_updater` agent が implementation 完了後に人間承認済み 成果物 を docs 正本へ反映するための、正本、承認確認、検証 の見方を提供する。

人間可読な実行境界、引き継ぎ、停止 / 戻し はこの skill を正本にする。

## 対応ロール

- `docs_updater` が使う。
- 返却先は 呼び出し元 または次 agent とする。
- 担当成果物は `updating-docs` の出力規約で固定する。

## 入力規約

- 呼び出し元: docs 正本化を依頼した agent または人間。
- 実装完了レポート: implementation 完了後の根拠レポート。
- 承認記録: 人間が docs 正本化を承認した記録。
- 承認済み成果物: docs 正本へ反映してよい成果物。
- 正本化対象: 更新してよい docs 正本。
- 非必須入力: 検証コマンド、根拠 docs を受け取る。
- 必須成果物: Codex implementation 完了 レポート、承認済み docs-only 成果物、`/Users/iorishibata/Repositories/AITranslationEngineJP/docs/index.md` を受け取る。

## 外部参照規約

- エージェント実行定義と実行境界は [docs_updater.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/docs_updater.toml) に従う。
- 紐づけ: [docs_updater.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/docs_updater.toml)
- エージェント実行定義: [docs_updater.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/docs_updater.toml)
- 禁止対象: プロダクトコード、プロダクトテスト、作業流れ / skill / エージェント実行定義の変更
- 実行境界: エージェント実行定義に従う
- docs index: [index.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/index.md)
- 詳細仕様正本: [detail-specs](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/detail-specs/README.md)
- 紐づけ: [docs_updater.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/docs_updater.toml)
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。
- 関連 skill: /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/updating-docs/SKILL.md

## 内部参照規約

### 拘束観点

- Codex implementation 完了 レポート の確認
- docs 正本 の選び方
- 人間承認 記録 の確認
- 承認済み 成果物 と 正本 対象 の対応
- 上位シナリオ単位の詳細仕様 製本
- 検証 と 残り 不足 の記録

## 判断規約

- implementation 完了後にだけ正本化へ進む
- 人間承認済み 成果物 だけを反映する
- docs-only 対象範囲 を超えない
- implementation-scope を docs 正本へ自動昇格しない
- `detail-specs` は 上位シナリオ 単位で作り、画面単位または個別ユースケース単位へ独断で分割しない
- `detail-specs` へ反映する内容は、`scenario-design`、`ui-design`、実装結果、レビュー結果から恒久仕様だけを製本する
- 未確定仕様を独断で補完しない

- Codex implementation 完了 レポート を根拠として残す
- 承認 記録 を根拠として残す
- 正本 と task 内成果物 を分ける
- 検証 結果を残す

## 非対象規約

- 作業流れ、skill、エージェント実行定義、プロダクトコード、プロダクトテストは変更しない。
- implementation 完了前の正本化と未承認 draft の正本化は扱わない。
- implementation-scope を docs 正本へ自動昇格しない。
- task 内の実画面確認結果を docs 正本へそのまま昇格しない。
- スキーマ移行、DB 移行、基盤移行、cutover 手順は `detail-specs` へ昇格しない。
- `detail-specs` へ移す対象は、承認済み `scenario-design` にある上位シナリオの恒久仕様だけにする。
- プロダクト実装を同時に進めない。

## 出力規約

- 判断結果: docs 正本化の完了、未完了、停止の判定を返す。
- 根拠参照: docs 更新の根拠にした承認記録と成果物を返す。
- 不足情報: docs 正本化を完了できない不足項目を返す。
- 次判断材料: `implement_lane` が次を判断できる材料を返す。
- 引き継ぎ先: `implement_lane` を返す。
- 渡す対象範囲: docs 更新結果、検証、残り 不足を返す。
- 変更 docs: 更新した docs ファイルを返す。
- 更新した正本: 反映した 正本 を返す。
- 確認結果: 実行した 検証 と未実行理由を返す。
- 残留不足: 未反映、未確認、判断待ちを返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- 出力規約を満たし、次の 実行者 が再解釈なしで判断できる。
- 不足情報または停止理由がある場合は明示されている。
- Codex implementation 完了 レポート を確認した。
- 人間承認 記録 を確認した。
- 承認済み 成果物 と 正本 対象 を対応づけた。
- 検証 結果と 残り 不足 を記録した。
- 必須 根拠: Codex implementation 完了 レポート、承認 記録、根拠成果物パス、検証結果。
- 完了判断材料: implementation 完了 後の docs 正本が 承認済み 成果物 と同期している。
- 残留リスク: 未反映、未確認、判断待ちが返っている。

## 停止規約

- Codex implementation レーン の修正完了が未確認の時
- 作業流れ / skill / エージェント実行定義 や skill / agent を変更する時
- プロダクトコードやプロダクトテストの変更が必要な時
- 人間承認 が不足している時
- Codex implementation レーン の修正完了が分からない場合は停止する。
- 承認 がない場合は停止する。
- 作業流れ 変更なら `implement_lane` へ戻す。
- プロダクト 実装が必要なら `implement_lane` へ戻す。
- 停止時は不足項目、衝突箇所、戻し先を返す。
- Codex implementation 完了が不足する場合は停止する。
- 承認が不足する場合は停止する。
- プロダクト実装が必要な場合は停止する。
- 作業流れ / skill / エージェント実行定義 の変更が必要な場合は停止する。
- docs-only 対象範囲ではない場合は停止する。
