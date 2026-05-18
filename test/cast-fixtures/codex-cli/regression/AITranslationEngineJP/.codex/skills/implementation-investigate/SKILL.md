---
name: implementation-investigate
description: Codex implementation レーン 側の実装時調査の共通作業プロトコル。単一引き継ぎ入力 1 件内で 根拠優先に調査する判断基準を提供する。
---
# Implementation Investigate

## 目的

`implementation-investigate` は作業プロトコルである。
`implementation_investigator` agent が、`単一引き継ぎ入力` 1 件と 承認済み実装範囲 内で実装時の証拠を集める時の共通判断を提供する。

実行境界は [implementation_investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_investigator.toml) が持ち、引き継ぎ は skill に従う。

## 対応ロール

- `implementation_investigator` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implementation-investigate` の出力規約で固定する。

## 入力規約

- 単一引き継ぎ入力: `implementation-scope` から切り出された実装時調査用 引き継ぎ 1 件。
- 実行中タスク成果物場所: 調査結果、観測結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- 調査対象: 再現、trace、観測、再観測の対象にするファイル、symbol、公開接点、コマンド。
- 対象調査範囲: 調査してよい実装範囲。
- 検証コマンド: 調査結果を確認する実行許可済み command。

## 外部参照規約

- エージェント実行定義と実行境界は [implementation_investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_investigator.toml) に従う。
- エージェント実行定義: [implementation_investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_investigator.toml)
- `agent-browser` CLI の利用規約は [agent-browser.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/references/agent-browser.md) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。
- 関連 skill: /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-investigate/SKILL.md, /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-investigate-reproduce/SKILL.md, /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-investigate-trace/SKILL.md, /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-investigate-observe/SKILL.md, /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-investigate-reobserve/SKILL.md

## 内部参照規約

### 拘束観点

- 根拠優先の観測
- 観測済み事実 と 仮説 の分離
- 一時観測点の除去
- `agent-browser` CLI による UI / console / screenshot 根拠
- 重点 skill の選び方

- 参照 型 は [investigation-patterns.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-investigate/references/patterns/investigation-patterns.md) とする。

## 判断規約

- `単一引き継ぎ入力` 1 件と 承認済み実装範囲 を超えない
- 根拠 のない結論を固定しない
- Codex implementation レーン のブラウザ操作は `agent-browser` CLI で行う
- 一時観測点は返却前に除去する
- 恒久修正と プロダクトテスト 追加を混ぜない

- 観測条件、コマンド、結果を残す
- 一時変更 と 除去状態 を返す
- 推奨次対応を根拠付きで返す
- active 規約 は agent 1:1。調査種別の差分は 重点 skill で扱い、出力 obligation はこの 規約 に固定する。

## 非対象規約

- 恒久修正、プロダクトテスト追加、design-time investigation は扱わない。
- 承認済み実装範囲外の調査は扱わない。
- 根拠のない結論は固定しない。
- 調査種別ごとの個別 JSON 規約は使わない。

## 出力規約

- 判断結果: 実装時調査の完了、未完了、停止の判定を返す。
- 根拠参照: 調査の根拠にした入力、コマンド、観測結果を返す。
- 不足情報: 調査を完了できない不足項目を返す。
- 次判断材料: `implement_lane` が次を判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。
- 返却先: implement_lane
- 調査焦点: 承認済み実装範囲 内で何を調べたかを返す。
- 再現状態: 再現できたか、未再現か、再現不要かを返す。
- 観測事実: 観測済み事実だけを書き、仮説と混ぜない。
- 仮説: 原因候補と根拠を返す。
- 観測点: 確認した入口、経路、対象を返す。
- 一時変更: 一時観測点を使った場合だけパスと目的を返す。
- 除去状態: 一時観測点の除去状態を返す。
- 確認結果: 実行した 検証 と未実行理由を返す。
- 残り 不足: 未確認事項と理由を返す。
- 残留リスク: 実装判断に残る リスク を返す。
- 推奨次対応: 実装、テスト、戻しのどれが妥当かを根拠付きで返す。

## 完了規約

- 承認済み実装範囲 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- 観測済み事実 と 仮説 を分けた。
- 承認済み実装範囲 内の 根拠 だけを扱った。
- 一時変更 と 除去状態 を確認した。
- 必須 根拠: 対象調査範囲、実行したコマンドまたは観測結果。
- 完了判断材料: implement_lane が implement、tests、または implement-lane への戻しを判断できる。
- 残留リスク: 未確認事項と理由が返っている。

## 停止規約

- 恒久修正を行う時
- プロダクトテスト を追加する時
- 設計時調査を行う時
- 停止時は不足項目、衝突箇所、戻し先を返す。
- 単一引き継ぎ入力が不足する場合は停止する。
- 調査対象が不足する場合は停止する。
- 対象調査範囲が不足する場合は停止する。
- 一時観測点を安全に除去できない場合は停止する。
- 設計判断が不足している場合は停止する。
- 承認済み実装範囲 外の調査が必要な場合は停止する。
