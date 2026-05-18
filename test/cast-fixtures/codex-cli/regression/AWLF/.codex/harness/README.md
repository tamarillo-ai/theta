# OM Harness

本目录是 OM 代理体系的运行时事实源。
目标不是继续扩写角色提示词，而是把关键行为沉淀为可版本化、可检索、可评测、可复用的 harness 工件。

## 目录结构

| 目录 | 作用 |
|---|---|
| `policies/` | 规定需求澄清、研究、架构思考、设计审批、来源排序等硬规则 |
| `schemas/` | 规定阶段输入输出的结构化契约 |
| `templates/` | 规定澄清、研究、设计评审、用户审批的推荐格式 |
| `evals/` | 评测代理是否真的会追问、研究、系统设计和拒绝补丁化方案 |
| `memory/` | 进度、决策、来源等留痕模板 |
| `scripts/` | 运行 schema 校验、session acceptance 与后续自动化检查 |

## 使用顺序

1. 读取根 `AGENTS.md`，明确宪法层规则。
2. 读取 `.codex/AGENT_RACI.md`，确定阶段职责与交接。
3. 根据当前阶段读取对应 `policies/` 与 `schemas/`。
4. 需要对用户展示时，使用 `templates/`。
5. 结束后用 `memory/` 留痕，并用 `evals/` 检查体系是否漂移。

## 迁移原则

- 保留控制平面：`router`、`pipeline`、`planner`、`explorer`、`architecture_guardian`、`reviewer`、`implementer`、`tester`、`docs`
- 移除静态领域专家：领域知识优先通过仓库探索、skills 和联网研究补足
- 设计先行：所有非只读任务必须先形成 `design_packet` 并获得用户批准
