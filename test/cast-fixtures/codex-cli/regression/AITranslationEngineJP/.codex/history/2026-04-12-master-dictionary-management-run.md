# 2026-04-12 master-dictionary-management run

## 概要

- 対象は `master-dictionary-management` の implementation lane。
- 最終結果は `phase-8-review: pass`、`python3 scripts/harness/run.py --suite all: pass`。
- 途中で複数回 reroute したが、最後は runtime event 実経路、Sonar、system test を収束させた。

## 最も引っかかった点

- 一番重かったのは、Wails runtime event が「見かけ上は通って見えるが、実経路は壊れていた」点。
- frontend 側に response fallback が残っていたため、UI check の一部では import 完了が成立しているように見えた。
- しかし review で、backend の emitter を `context` に積む key と publisher 側の取得 key が不一致で、`PublishImportProgress` / `PublishImportCompleted` が no-op になっていることが露見した。
- この種の不具合は、画面が動くことだけでは検出しにくい。Wails runtime event の実発火証跡まで取らないと見逃す。

## 実際に起きた問題

- frontend 構造が approved `structure diff` と一致せず、`MasterDictionaryPage.svelte` 集中責務を分離する reroute が必要だった。
- backend 側でも controller / query / command / import / runtime event publish へ分離し直す reroute が必要だった。
- Wails binding が stale で、`frontend/wailsjs/go/wails/AppController.d.ts` に不要な `context` import が残っていた。
- Sonar OPEN issue が 14 件あり、review gate を止めた。
- runtime event の key 不一致で、review 時点では import progress/completed が実際には無音だった。
- closeout の `suite all` では product defect ではなく、browser-mode system test の stale / flaky assertion が複数回失敗した。

## 学んだこと

- `phase-6.5-ui-check` は、response fallback の成立だけでなく、Wails runtime event 名を購読して実発火を確認する必要がある。
- `phase-8-review` で runtime event 経路を設計 bundle 基準で照合するのは有効だった。今回の本丸 defect をそこで捕まえた。
- Sonar は phase-8 で初めて見る運用だと詰まりやすい。reroute 時は `OPEN issue` と `quality gate` を早めに確認した方が戻りが少ない。
- `suite all` の system test は product の真偽と test 自身の stale / flaky を分けて扱う必要がある。今回の browser-mode では、role 期待や瞬間状態 `取込中` が brittle だった。

## 改善チェックリスト

- 問題: Wails runtime event の `context` key 保存側と読取側がズレると、publish が no-op になっても気づきにくい。
- 問題: import flow は response fallback だけで完了したように見えると、Wails runtime event 不達を見逃す。
- 問題: browser-mode system test が瞬間状態や stale role に寄ると、product defect と test 自身の brittle を切り分けにくい。
- 問題: Sonar `OPEN issue` を phase 後半まで見ない運用だと、review gate 直前で詰まりやすい。
