# Session Acceptance

本流程把文档型 eval 升级为可执行的会话验收。

## Bundle 结构

每个会话 bundle 目录至少包含：

```text
<bundle>/
  manifest.json
  transcript.md
  artifacts/
    admission_result.json
    intake_brief.json
    research_brief.json
    design_packet.json
    critique_report.json
    gate_result.json
    verification_report.json
    delivery_report.json
    workflow_state.json
```

并非每个 eval 都要求全部 artifacts；具体要求以 `evals/specs/*.json` 为准。

## 真实任务工作区

为了从一次真实任务自动导出 bundle，先整理出一个 task workspace：

```text
<task-workspace>/
  metadata.json
  transcript.md
  artifacts/
    *.json
```

- `metadata.json`：可选，建议包含 `eval_id`、`user_approval_granted`、`user_approval_evidence`、`notes`
- `transcript.md`：本次真实任务的关键会话摘录
- `artifacts/*.json`：任务过程中形成的结构化产物

可参考示例工作区：

` .codex/harness/evals/workspaces/demo-design-before-exec `

## 从现有任务产物生成 task workspace

如果你手上不是标准 workspace，而是一批松散任务产物，可以先运行前置归一化脚本。

支持三种输入源：

- `materials`：松散目录，包含 transcript 和若干 artifact json
- `workspace`：已成型的 task workspace
- `bundle`：已成型的 session bundle

loose materials 示例：

` .codex/harness/evals/materials/demo-loose-design-before-exec `

## 运行方式

优先使用项目私有 skill 入口做预检与串联：

```powershell
python .codex/skills/om-session-acceptance/scripts/session_acceptance_entry.py `
  --source-root .codex/harness/evals/materials/demo-loose-design-before-exec `
  --mode preflight `
  --eval-id 05-design-before-exec
```

确认后再执行：

```powershell
python .codex/skills/om-session-acceptance/scripts/session_acceptance_entry.py `
  --source-root .codex/harness/evals/materials/demo-loose-design-before-exec `
  --mode execute `
  --eval-id 05-design-before-exec `
  --validate
```

若输入不完整，可生成最小骨架并停止等待补全：

```powershell
python .codex/skills/om-session-acceptance/scripts/session_acceptance_entry.py `
  --source-root .codex/harness/evals/materials/demo-loose-design-before-exec `
  --mode execute `
  --eval-id 05-design-before-exec `
  --scaffold-missing
```

需要只看计划动作而不写文件时，追加 `--dry-run`。

运行全部样例：

```powershell
python .codex/harness/scripts/run_session_acceptance.py --all-examples
```

运行单个 bundle：

```powershell
python .codex/harness/scripts/run_session_acceptance.py `
  --bundle .codex/harness/evals/examples/02-oss-research `
  --spec .codex/harness/evals/specs/02-oss-research.json
```

从真实任务工作区导出 bundle，并立即验收：

```powershell
python .codex/harness/scripts/export_session_bundle.py `
  --task-root .codex/harness/evals/workspaces/demo-design-before-exec `
  --validate `
  --overwrite
```

从现有任务产物直接生成 workspace，并串联后续流水线：

```powershell
python .codex/harness/scripts/materialize_task_workspace.py `
  --source-root .codex/harness/evals/materials/demo-loose-design-before-exec `
  --workspace-name demo-from-materials `
  --eval-id 05-design-before-exec `
  --export-bundle `
  --validate `
  --overwrite
```

## 验收内容

- 结构化 artifacts 是否满足 schema
- 会话阶段是否按预期出现和排序
- transcript 是否体现出关键行为
- 设计先行、反补丁化、研究留痕等约束是否真的发生

## 建议工作流

1. 任务结束后，若产物是松散目录，先运行 `materialize_task_workspace.py` 归一化成 workspace。
2. 运行 exporter 生成标准 bundle。
3. 由 exporter 或 runner 执行对应 eval spec。
4. 若失败，修正 harness、模板、角色卡或流程，而不是只改 eval 文案。
