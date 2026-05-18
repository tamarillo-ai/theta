# Repository Guidelines

本文件用于定义 AI 与维护者在 OM 根仓内的行为规范与工作流边界。内容仅包含行为与流程，不重复技术细节。

其上级规范为C:\Users\Administrator\.codex\AGENTS.md，同样需要你参阅

## 1. 身份与边界
- 目标：优先保证技术正确性与可验证性。
- 不确定时必须明确说明，并请求进一步信息或验证。
- 不推测未证实内容，避免臆断。
- 任何重大变更（结构/工具链/构建入口）需先说明影响并获得确认。

## 2. 工作流（固定顺序）
1) 理解：读取必要上下文与文档索引。
2) 计划：复杂任务必须列 TODO 并逐步更新状态。
3) 执行：小步修改，保持一致性。
4) 验证：执行必要检查或测试。

## 3. 构建体系
- 和XMake有关

## 4. 文件与路径约束
- build 目录统一为 `build/<profile>`。
- 所有路径使用仓库相对路径表述；对仓库外部 worktree 统一使用相对 `robot/` 的上级相对路径表述，例如 `../worktrees/oh-my-robot/<task>/`。
- 不删除与任务无关的文件。

## 5. 文档一致性要求
- 构建体系相关文档以以下为准：
  - `oh-my-robot/docs/quick_start.md`
  - `oh-my-robot/docs/build/maintenance_manual.md`
- 构建流程变更必须同步更新上述两份文档。
- Git / 分支 / worktree / 协作流程相关文档以 `oh-my-robot/docs/process/git_collaboration_spec.md` 为准；此类流程变更必须同步更新该文档。
- 新增构建文档时需在上述两份手册中建立索引或链接。

## 6. 变更记录与维护
- 重要流程变更需追加 `oh-my-robot/docs/build/maintenance_manual.md` 的“变更记录”小节。
- Git / worktree 协作规则正文统一维护在 `oh-my-robot/docs/process/git_collaboration_spec.md`；维护手册仅保留变更摘要，不重复维护正文。
- 历史/过程类文档仅作为归档参考，不作为现行标准。

## 7. 编码与格式
- 文档统一使用 UTF-8 编码。请使用UTF-8格式打开，UTF-8格式保存。
- 中文为主，术语保持一致（host/build profile）。

## 8. 并行开发（KISS）
- `robot/main` 是 root 仓库唯一共享集成基线；只承担子模块指针更新、联调验证、root 构建入口与 root 文档调整。
- `oh-my-robot` 是唯一日常开发仓库；其 `feature/*`、`fix/*`、`hotfix/*` 分支及并行 worktree 统一在 `oh-my-robot` 自己的仓库中管理。
- 开发 `oh-my-robot` 时，worktree 必须放在 `robot/` 外部；推荐目录为 `../worktrees/oh-my-robot/<task>/`；禁止继续使用 `robot/.worktrees/` 作为 `oh-my-robot` 的默认开发池。
- `robot/oh-my-robot` 仅视为集成快照；可用于子模块指针收敛、联调验证与发布前检查，不作为 `oh-my-robot` 的日常功能开发 checkout。
- `oh-my-robot` 的远端拓扑、分支命名、rebase、PR 与强推规则统一以 `oh-my-robot/docs/process/git_collaboration_spec.md` 为准；root 仓库不重复维护子模块开发细则。
- 若 root 仓库本身需要临时隔离工作区，仅可作为单次集成任务例外，并需先获用户确认；不得反向把该例外扩展成 `oh-my-robot` 的常态开发模式。
- `robot/main` 只允许记录官方 `upstream/integration` 已可达的子模块提交，不得指向仅存在于个人 Fork 的临时提交。
