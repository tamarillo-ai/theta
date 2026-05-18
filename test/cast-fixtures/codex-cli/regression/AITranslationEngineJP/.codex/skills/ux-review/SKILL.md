---
name: ux-review
description: Codex frontend 人間レビュー前 UX 事前確認作業プロトコル。UX 標準適合、fakeAPI 状態、視認性、既存画面との統一性を確認する。
---
# UX Review

## 目的

`ux-review` は frontend 実装後、人間レビュー前に UX 事前確認を行う作業プロトコルである。
実画面、UI 根拠、fakeAPI 状態を照合し、人間レビューへ進めるかを `ux-review.yaml` に固定する。

## 対応ロール

- `ux_review` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` の frontend 人間レビュー前判断 とする。
- 担当成果物は `ux-review` の出力規約で固定する。

## 入力規約

- UX確認対象差分: frontend 人間レビュー前に確認する差分を受け取る。
- 実装目的: frontend 実装が満たすべき目的を受け取る。
- UI根拠: 承認済み `ui-design.md` と関連する task 内 UI 成果物を受け取る。
- 実装結果: frontend 実装 agent が返した実装結果を受け取る。
- 実画面確認入力: review URL、起動状態、確認済み `fakeScenario`、未確認状態、未確認理由を受け取る。
- 変更ファイル: frontend 実装差分に含まれる変更ファイル一覧を受け取る。
- 作業計画フォルダ: `docs/exec-plans/active/<task-id>/` を受け取る。
- UX確認YAMLパス: `docs/exec-plans/active/<task-id>/ux-review.yaml` を受け取る。

## 外部参照規約

- エージェント実行定義と実行境界は [ux_review.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/ux_review.toml) に従う。
- UX 標準正本は [UX-standard.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/UX-standard.md) とする。
- fakeAPI 運用仕様は [frontend-fake-api.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/frontend-fake-api.md) とする。
- UI 設計規約は [ui-design](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/ui-design/SKILL.md) とする。
- `agent-browser` 利用規約は [agent-browser.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/references/agent-browser.md) とする。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

UX 事前確認観点表は次を拘束する。

| 観点 | 確認内容 |
| --- | --- |
| UX標準適合 | `docs/UX-standard.md` の高優先度項目に反していないか |
| ユーザー判断可能性 | ユーザーが目的、状態、操作、結果を判断できるか |
| fakeAPI状態十分性 | 人間レビューに必要な `fakeScenario` が確認されているか |
| 視認性 | 要素サイズ、密度、余白、一覧幅が読みにくくないか |
| 画面統一性 | 既存画面、既存部品、文言、配置規則との統一感が壊れていないか |

重大度指標は次を拘束する。

| レベル | 意味 |
| --- | --- |
| `blocker` | 画面確認不能、review URL 不備、状態確認不能により人間レビューへ進めない問題 |
| `major` | UX標準違反、fakeAPI状態不足、視認性破綻、既存画面との不統一により修正が必要な問題 |
| `minor` | 人間レビュー時の注意として残せる改善問題 |
| `nit` | 修正してもよいが、人間レビュー前の判断を止めない微細な問題 |

## 判断規約

- UX 事前確認問題の重大度は内部参照規約の重大度指標から選ぶ。
- `blocker` と `major` は `fix_required: true` にする。
- `minor` と `nit` は `fix_required: false` にする。
- 未解決の `fix_required: true` がある場合は `must_fix_open: true` にする。
- `max_level` は未解決指摘の最大重大度にする。
- `ux-review.yaml` の `must_fix_open: true` は frontend 人間レビュー前の停止判断材料にする。
- UI 根拠にない改善案は、修正指示ではなく未承認改善候補として扱う。
- fakeAPI は backend 実装、統合境界実装、永続化仕様の代替として扱わない。
- 呼び出し元から渡された実画面確認入力を UX 事前確認の根拠として扱ってよい。

## 非対象規約

- プロダクトコード、プロダクトテスト、検証データ、スナップショット、test helper は変更しない。
- 実装後 5 観点レビューは扱わない。
- backend 実装、統合境界実装、永続化仕様の妥当性は主判定にしない。
- ハーネスを実行しない。
- 改善ログを作成または追記しない。

## 出力規約

- UX確認YAML: `docs/exec-plans/active/<task-id>/ux-review.yaml` を作成、追記、解決更新、削除する。
- `review_status`: `no_issue`、`issues_open`、`stopped` のいずれかを返す。
- `must_fix_open`: `major` 以上の未解決問題があるかを返す。
- `max_level`: 未解決問題の最大重大度を返す。
- `checked_states`: 確認した `fakeScenario` と根拠を返す。
- `unchecked_states`: 未確認の `fakeScenario`、理由、残るリスクを返す。
- `issues`: 問題、重大度、根拠、修正確認方法を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示、修正範囲の命令を含めない。

## 完了規約

- `ux-review.yaml` に UX 事前確認の状態、根拠、未解決指摘、未確認範囲が記録されている。
- UX 事前確認観点表を確認した。
- `docs/UX-standard.md` の高優先度項目と対象画面に関係する項目を確認した。
- fakeAPI 運用仕様の標準 `fakeScenario` と task 固有の確認状態を確認した。
- 実画面で、目的、状態、操作、結果をユーザーが判断できるか確認した。
- 実画面で、要素サイズ、密度、余白、一覧幅、既存画面との統一性を確認した。
- `review_status`、`must_fix_open`、`max_level`、`checked_states`、`unchecked_states`、`issues` が記録されている。
- 残留リスクとして、未確認状態と理由が記録されている。

## 停止規約

- `UX確認対象差分` が不足する場合は停止する。
- `実装目的` が不足する場合は停止する。
- `UI根拠` が不足する場合は停止する。
- `実画面確認入力` が不足する場合は停止する。
- `UX確認YAMLパス` が不足する場合は停止する。
- 実画面確認に必要な review URL、起動状態、確認状態、未確認理由が判定できない場合は停止する。
- UX 事前確認以外の観点を主判定にしそうな場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
