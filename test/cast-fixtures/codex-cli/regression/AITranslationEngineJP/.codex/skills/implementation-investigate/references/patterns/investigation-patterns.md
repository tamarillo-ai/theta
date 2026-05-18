# Investigation Patterns

## 目的

`implementation_investigator` が実装前後の 根拠 を集めるための判断パターンをまとめる。
agent TOML の ツール権限 と skill の出力規約は上書きしない。

## 採用する考え方

- 入口 から 完了 まで call chain を追う。
- 観測済み事実、仮説、残り 不足 を分ける。
- silent 失敗、空 catch、dangerous 代替処理、lost stack trace を重点的に探す。
- temporary observation は目的、path、除去確認を記録する。
- build / 実行定義 エラー は最小再現と最小観測点から切り分ける。

## 適用ルール

- Wails 紐づけ、frontend gateway、backend service、infra adapter のどこで失敗したかを分ける。
- console、backend log、test 出力、UI 状態 を 根拠 として区別する。
- UI 状態、console、screenshot は `agent-browser` CLI の コマンド 出力 として残す。
- paid real AI API を調査で呼ばない。fake / DI seam / test mode を使う。
- 一時観測点は返却前に除去し、cleanup_status を必ず返す。

## 赤旗

- `catch {}`、`.catch(() => [])`、原因を隠す 既定 value がある。
- 再現条件を変えたまま 通過 と判断している。
- 仮説を fact として実装種別別 agent へ渡している。
- temporary 変更 が残ったまま 完了 している。
