## Project

**特价机票发现平台**

一个面向中文用户的网页应用，用来集中发现 OTA 平台和航空公司的特价机票，并把分散、复杂、容易踩坑的票价信息整理成可快速判断的决策界面。它不是另一个订票站，而是一个"特价发现 + 规则翻译 + 购买决策辅助"平台，优先服务价格敏感、时间相对灵活、希望捡到高性价比机票的人。

**Core Value:** 用户能在几分钟内判断一张"看起来便宜"的机票到底值不值得买，并能立即采取下一步。

### Constraints

- **Data freshness**: 票价时效很短，所有 deal 都需要展示采集时间、最后更新时间、失效时间
- **Trust**: 任何"值得买"结论都要附解释，不能只给分数不给理由
- **Execution**: 这是 greenfield MVP，优先验证产品价值，避免一开始投入重型搜价/出票能力
- **Experience**: 必须 mobile-first，同时保证桌面端筛选和比较体验足够高效
- **Content operations**: v1 很可能先采用人工或半自动录入 deal 的方式，保证上线速度和内容质量

## Technology Stack

Key: Next.js 15 + Payload CMS 3.82 + SQLite + Amadeus API + TypeScript + Vitest.

## Workflow

This project uses **Superpowers (obra/superpowers)** workflow.
Load skills through the native `skill` tool.
Key skills: brainstorming, writing-plans, using-git-worktrees, test-driven-development.
