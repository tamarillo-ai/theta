# UI Design: <task-id>

- `skill`: ui-design
- `status`: draft
- `source_plan`: `./plan.md`
- `scenario_source`: `./scenario-design.md`
- `ux_standard_source`: `docs/UX-standard.md`

## UI Contract

- `display_items`:
- `primary_actions`:
- `button_enablement`:
- `state_variants`:
- `post_implementation_review`:

## Interface Frame

- `purpose`:
- `audience`:
- `primary_workflow`:
- `information_density`:
- `visual_direction`:
- `remembered_signal`:

## Structure Notes

- `page_sections`:
- `layout_constraints`:
- `responsive_constraints`:
- `accessibility_constraints`:

## UX Standard Review

- `source`: `docs/UX-standard.md`
- `screen_structure_high_priority_results`:
- `screen_structure_applicable_results`:
- `layout_responsive_high_priority_results`:
- `layout_responsive_applicable_results`:
- `deferred_items`:

## Interaction States

- `loading`:
- `empty`:
- `error`:
- `disabled`:
- `progress`:
- `retry`:
- `success`:

## Post Implementation Review

- `desktop_review_points`:
- `mobile_review_points`:
- `overflow_risks`:
- `visual_polish_open_questions`:

## Agent Browser Review

- `command_source`: `agent-browser`
- `checked_url`:
- `checked_viewports`:
- `ux_standard_review`:
  - `source`: `docs/UX-standard.md`
  - `high_priority_results`:
  - `applicable_results`:
  - `deferred_items`:
- `wording_review`:
  - `review_timing`: `after_agent_browser_review`
  - `fixed_names_preserved`:
  - `business_japanese_terms`:
  - `internal_state_names_hidden`:
  - `next_action_wording`:
  - `allowed_english_labels`:
  - `plain_language_next_action_judgement`:
- `console_errors`:
- `screenshot_or_snapshot_refs`:
- `layout_breaks`:
- `ambiguous_interactions`:
- `open_issues`:
- `not_checked_reason`:

## Rules

- UI は `ui-design.md` の UI 要件契約で固定する
- UI 確認は実画面で行う
- UX 標準の確認結果を `UX Standard Review` に記録する
- `agent-browser` 確認後に、専門知識がなくても次に何をするか分かる表現水準かを表示文言レビューで確認する
- 固定名以外の画面表示文言は、日本語の業務語へ置き換える
- 内部状態名は画面に出さず、利用者の次操作を示す文へ変換する
- 英語ラベルは、利用者が設定画面で見る既存語だけに限定する
- 既存画面変更では、既存画面または既存 UI 部品を土台にする
- 既存画面変更では、独自の page shell、card、grid、配色、余白体系を新規に作らない
- 既存画面変更では、変更対象区画だけを差し替え、変更しない区画は既存画面の構造と表示を維持する
- 新規画面では、`docs/screen-design` の画面設計に従う
- 細かな visual polish は実装後に人間が実物を確認して直す
- product component 名や owned scope は、implementation-scope で必要な時だけ扱う
- implementation-scope の `owned_scope` や product code 対象 file は書かない
