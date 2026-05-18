---
applyTo: "**/*.py,**/*.pyi"
---

# Python 向け追加指示

このファイルは、Python コードに対する GitHub Copilot の追加指示である。

この repository では、Python は主に無線通信シミュレーション、数値実験、CSV 処理、可視化、論文用データ生成に使う前提で扱う。

## 基本ルール

- Python では `PEP 8` を守り、関数、メソッド、公開 API には型ヒントを付ける。
- Python スクリプトや Python 製 CLI の実行は、基本 `uv` を使い、`python foo.py` より `uv run python foo.py`、`pytest` より `uv run pytest` を優先する。
- 可能なら `from __future__ import annotations` を使う前提で考える。
- 文字列整形は基本的に f-string を使う。
- ファイル操作やパス結合は `os.path` より `pathlib.Path` を優先する。
- デバッグ用途の `print()` を増やさず、必要なら `logging` を使う。
- `NumPy` / tensor / dataframe を扱うコードでは、shape、dtype、軸、列名の前提をコードから読めるようにする。
- 実験コードでは、seed、設定値、出力先、保存ファイル名を追跡しやすく保つ。

## 構造

- 責務ごとに関数やモジュールを分割し、巨大なファイルや長すぎる関数を避ける。
- 実装は既存のレイヤ分割に合わせ、ビジネスロジックを I/O 層へ混ぜ込みすぎない。
- DTO や設定オブジェクトには `dataclass` を検討する。
- 抽象化が必要な箇所では `Protocol` や明確な interface を活用する。
- シミュレーション本体、設定ロード、CSV 入出力、可視化は可能な範囲で分離する。
- 1 つの関数に「計算」「保存」「plot」「ログ出力」を詰め込みすぎない。

## テスト

- テストは `pytest` を前提に考える。
- 新しい振る舞いを追加する場合は、できるだけ先に失敗するテストを書く。
- テスト名は振る舞いが伝わるようにし、正常系だけでなく異常系も含める。
- I/O や外部依存は分離し、unit test しやすい形を優先する。
- カバレッジ確認が必要な場合は `pytest --cov=src --cov-report=term-missing` を基準にする。
- 数値系テストでは、shape、列名、しきい値、許容誤差、期待ファイル出力を明示する。

## 品質ゲート

- format は `black`、import 整理は `isort`、lint は `ruff` を想定する。
- 検証コマンドも、基本 `uv run black --check .`、`uv run isort --check-only .`、`uv run ruff check .` のように `uv run` 形式で扱う。
- 自動生成するコードも、この前提に沿った import 順と style で出力する。
- 例外処理は明示的に行い、失敗時の文脈が分かる実装にする。
- 可視化コードでは、軸ラベルや凡例の欠落を軽視しない。

## セキュリティ

- API key や認証情報は環境変数または secret 管理経由で扱う。
- SQL を使う場合は必ず parameterize し、文字列連結で query を作らない。
- 外部入力は検証し、file path、コマンド実行、HTML 出力では特に慎重に扱う。
- セキュリティ確認が必要な場合は `bandit -r src/` を候補にする。

## レビュー優先順位

- まずバグ、例外漏れ、型の不整合、境界条件の欠落を探す。
- 次にテスト不足、可読性、保守性、性能上の明確な問題を確認する。
- 修正提案では、最小変更で安全性を上げる案を優先する。
- シミュレーション用途では、shape mismatch、dtype drift、列名ズレ、seed 漏れ、誤った集計軸を優先して確認する。
