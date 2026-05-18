---
name: browser-confirmation
description: 実装後ブラウザ確認で、呼び出し元が定義した確認 URL、操作経路、期待値、安全条件に従い、ブラウザ証跡を残す作業プロトコル。
---
# Browser Confirmation

## 目的

`browser-confirmation` は、実装後ブラウザ確認を軽量に実行する作業プロトコルである。
`browser_confirmation` agent が確認 URL、操作経路、期待値、安全条件に従い、ブラウザ証跡、異常、未確認理由を返す時に使う。

## 対応ロール

- `browser_confirmation` が使う。
- 呼び出し元は `implement_lane`、`fix_lane`、`light_change_lane` とする。
- 返却先は呼び出し元とする。
- 担当成果物は `実装後ブラウザ確認` とする。

## 入力規約

- 呼び出し元: 確認を依頼した lane agent。
- 確認 URL: `agent-browser` で開く対象 URL。
- 起動状態: アプリ、サーバー、テストデータ、認証状態の準備状況。
- 操作経路: 実行する画面操作の列。
- 操作期待値: 各操作後に満たすべき画面状態、表示、状態変化。
- 禁止操作: 実行してはいけない操作。
- 証跡出力先: `snapshot`、`errors`、必要な `screenshot` とログを置く path。

## 外部参照規約

- エージェント実行定義と実行境界は [browser_confirmation.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/browser_confirmation.toml) に従う。
- `agent-browser` CLI の利用規約は [agent-browser.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/references/agent-browser.md) に従う。
- 確認経路と期待値の定義は呼び出し元 lane の成果物に従う。
- 外部成果物が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

実装後ブラウザ確認の成果物は次を必ず持つ。

| 成果物 | 拘束する内容 |
| --- | --- |
| 操作確認結果 | 操作経路ごとの実行結果と期待値との差分 |
| 証跡参照 | `snapshot`、`errors`、必要な `screenshot` とログの path |
| 異常記録 | console または network の異常 |
| 未確認理由 | 確認できなかった操作、期待値、証跡の理由 |
| 戻し先 | 呼び出し元 lane agent |

## 判断規約

- 確認 URL、操作経路、操作期待値、安全条件に従って実行する。
- `snapshot` と `errors` は必ず取得する。
- `screenshot` は画面状態、表示差分、未確認理由の説明に必要な場合に取得する。
- 期待値の妥当性は判断しない。
- 仕様判断、原因推定、修正方針作成は行わない。

## 非対象規約

- 確認経路の追加は扱わない。
- 期待値の追加は扱わない。
- 仕様判断は扱わない。
- 原因推定は扱わない。
- 修正方針作成は扱わない。
- プロダクトコード、プロダクトテスト、docs 正本本文は変更しない。

## 出力規約

- 操作確認結果: 操作経路ごとの実行結果と期待値との差分を返す。
- 証跡参照: `snapshot`、`errors`、必要な `screenshot` とログの path を返す。
- 異常記録: console または network の異常を返す。
- 未確認理由: 確認できなかった操作、期待値、証跡の理由を返す。
- 戻し先: 呼び出し元 lane agent を返す。
- 禁止事項: 出力に仕様判断、原因推定、修正方針、プロダクトコード変更義務を含めない。

## 完了規約

- 操作経路ごとの実行結果が根拠参照付きで返っている。
- `snapshot` と `errors` の証跡参照が返っている。
- 必要な `screenshot` とログの証跡参照または未取得理由が返っている。
- console または network の異常がある場合は異常記録が返っている。
- 未確認がある場合は未確認理由と戻し先が返っている。

## 停止規約

- 確認 URL が不足する場合は停止する。
- 起動状態が不足する場合は停止する。
- 操作経路が不足する場合は停止する。
- 操作期待値が不足する場合は停止する。
- 安全条件が不足する場合は停止する。
- 証跡出力先が不足する場合は停止する。
- 有料 API 到達リスクがある場合は停止する。
- 外部送信リスクがある場合は停止する。
- 破壊的操作リスクがある場合は停止する。
- ブラウザ操作不能の場合は停止する。
