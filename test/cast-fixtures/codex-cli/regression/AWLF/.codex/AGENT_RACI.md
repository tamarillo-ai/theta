# OM Harness Orchestration Manual

本文件定义编排职责、激活规则、阶段交接与报告模板。
宪法层门禁与阻断规则以根目录 `AGENTS.md` 为准。

## 1. 角色基线

| 角色 | 主责 | 何时必启用 |
|---|---|---|
| `router` | 任务意图识别、风险分级、角色召回建议 | 每轮任务开始 |
| `pipeline` | 状态机推进、门禁裁决、用户审批与最终交付 | 每轮任务开始 |
| `planner` | 需求澄清、方案设计、交付设计包 | 每个非平凡任务 |
| `explorer` | 仓库证据、成熟开源方案、最新信息研究 | 涉及未知实现、最佳实践、最新信息、推荐或高风险事实 |
| `architecture_guardian` | 架构边界、依赖方向、迁移与系统风险裁决 | `L3/L4`、契约变化、架构/依赖重排 |
| `reviewer` | 反例审查、补丁化风险、测试缺口审查 | 设计评审与实现后审查 |
| `implementer` | 按批准方案落地实现 | 用户批准设计后 |
| `tester` | 验证证据与回归覆盖 | 所有实现任务 |
| `docs` | 术语与文档一致性、DIA 收口 | 文档影响不为零或规范变化任务 |

静态领域专家已移除。领域知识默认通过以下方式补足：

- 仓库证据探索
- harness policies / schemas / templates
- 已安装 skills
- 联网研究与成熟开源对标

## 2. 激活规则

- 默认优先少角色编排，避免上下文污染。
- `router` 只给出建议，`pipeline` 负责最终裁决。
- `architecture_guardian` 不是默认常驻，仅在系统级风险存在时激活。
- `docs` 可按需启用，但本轮若修改规范、流程或模板，应视为必启用。
- 发现问题属于领域知识缺口时，优先补充 research/skills，不恢复旧静态专家角色。

## 3. 阶段操作手册

### 3.1 INTAKE / ADMISSION

- `router` 读取 `AGENTS.md`、相关 policies 与 `admission_result` schema，输出风险等级、模式建议、角色建议。
- `pipeline` 确认 `workflow_mode`、`activated_roles`、`requires_live_research`、`requires_user_approval`，并回写 `workflow_state`。

### 3.2 DISCOVERY

- `planner` 主责，必要时会签 `explorer`。
- 必须先探索本地事实，再向用户提出高影响问题。
- 产出 `intake_brief`，未收敛不得进入后续阶段。

### 3.3 RESEARCH

- `explorer` 主责，必要时会签 `planner` 与 `architecture_guardian`。
- 必须完成本地证据、外部来源、候选方案、来源级别、维护活跃度、风险与机制说明记录。
- `research_brief` 负责让用户理解候选方案，不替用户做最终采纳裁决。

### 3.4 DESIGN

- `planner` 主责，`architecture_guardian` 与 `reviewer` 默认会签。
- 必须形成 `design_packet`，其中包含：
  - 目标与边界
  - 复用/新接口决策
  - 备选方案与拒绝理由
  - 系统影响、失败模式、迁移与回退
  - 验证计划
  - 待用户裁决点

### 3.5 DESIGN_CRITIQUE

- `architecture_guardian` 与 `reviewer` 共同完成。
- 输出 `critique_report` 与 `gate_result`。
- 若发现系统性问题、补丁式修复、验证缺口或迁移策略缺失，必须回退到 `DESIGN`。

### 3.6 WAIT_USER_APPROVAL

- `pipeline` 展示设计摘要与待确认点。
- 未获得用户明确批准，不得激活 `implementer`。

### 3.7 IMPLEMENT / VERIFY / REVIEW / DELIVER

- `implementer` 只按批准后的 `design_packet` 执行，不得自行改写目标。
- `tester` 依据 `verification_report` schema 产出验证证据。
- `reviewer` 基于验证结果做最终风险审查。
- `docs` 确认 `docs_impact` 与术语一致性。
- `pipeline` 汇总 `delivery_report`、`workflow_timeline` 与 `workflow_final_report`。

## 4. 必读工件映射

| 阶段 | 必读 policies | 必读 schemas | 常用 templates |
|---|---|---|---|
| `DISCOVERY` | `questioning_policy.md` | `intake_brief.schema.json` | `clarifying_questions.md` |
| `RESEARCH` | `research_policy.md`, `source_ranking_policy.md` | `research_brief.schema.json` | `oss_comparison.md` |
| `DESIGN` | `design_first_policy.md`, `architecture_policy.md` | `design_packet.schema.json` | `design_review.md` |
| `WAIT_USER_APPROVAL` | `design_first_policy.md` | `design_packet.schema.json` | `user_approval.md` |
| `VERIFY` | `architecture_policy.md` | `verification_report.schema.json`, `gate_result.schema.json` | `design_review.md` |

## 5. 报告模板

### 5.1 路由激活披露

```markdown
### 角色激活
- role:
- task_summary:
- activation_rationale:
- must_read:
- expected_outputs:
```

### 5.2 阶段摘要（stage_brief_summary_log）

```markdown
### 阶段 <phase> | 角色 <role>
**背景**
<1-2 行>

**结论**
<1-2 行>

**关键证据**
<1-2 行>

**风险或阻塞**
<1-2 行>

**下一步**
<1 行>
```

### 5.3 最终工作流报告

```markdown
## 最终工作流报告
### 目标与范围

### 研究结论与来源摘要

### 设计方案与关键取舍

### 验证结果与剩余风险

### 文档影响与后续动作
```
