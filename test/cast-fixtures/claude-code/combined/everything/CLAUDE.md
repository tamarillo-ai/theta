# CLAUDE.md

此文件为 Claude Code (claude.ai/code) 在本仓库中工作时提供指导。

## 项目概述

**wewins-fota-new** 是一个大规模物联网固件 OTA（Over-The-Air）管理平台，旨在管理数百万设备的固件更新。项目目前处于早期开发阶段，拥有完整的架构文档，但代码实现极少。

## 技术栈

- **后端**: Java（预期使用 Spring Boot - 尚未实现）
- **数据库**:
  - PostgreSQL - 主数据存储（设备、策略、产品）
  - Redis - 缓存、限流、设备活跃度跟踪（使用 Bitmap）
  - ClickHouse - 事件记录和分析
- **消息队列**: RabbitMQ 用于异步事件处理
- **文件存储**: RustFS（或 S3 兼容存储）用于固件包
- **CDN**: 用于固件分发
- **基础设施**: Docker 容器

## 架构原则

### 单一代码库，多种部署模式
系统采用**同构部署模型**，使用一个 Docker 镜像以不同模式部署：

- **主区域** (`MODE=main`)：中央控制、管理后台、跨区域数据接入
- **区域部署** (`MODE=region`)：本地数据处理、设备 API、配置同步

### 跨区域通信
- 所有跨区域通信均使用**公网 HTTPS**（无私有网络）
- 使用 HMAC + nonce 认证保证安全
- 控制面：30 秒轮询进行配置同步
- 数据面：本地 MQ 缓冲 + Forwarder 模式批量传输

### 数据本地化
- 设备 API (`/v1/upgrade/check`, `/v1/upgrade/report`) 由本地区域提供服务
- 本地 Redis 用于实时操作（策略快照、限流）
- 本地 ClickHouse 用于详细事件日志
- 主区域维护汇总数据和设备位置索引

## 核心 API 接口

### 设备 API
- `GET /v1/upgrade/check` - 设备检查固件更新
- `POST /v1/upgrade/check` - 备用 POST 方法，支持扩展载荷
- `POST /v1/upgrade/report` - 设备上报升级状态（DL_START、DL_OK、DL_FAIL、UP_OK）

### 内部 API（仅主区域）
- `GET /internal/config/version` - 配置版本轮询
- `GET /internal/config/snapshot/*` - 快照拉取（策略、产品、控制）
- `POST /internal/ingest/*` - 跨区域汇总数据接入

## 核心设计模式

### 灰度发布
- 基于哈希的设备选择：`Hash(imei) % 100 < gray_rate`
- 百分比控制（0-100%）
- 时间窗口限制
- 通过 Redis 计数器和分布式锁实现

### 设备活跃度跟踪（Redis Bitmaps）
- 设备自增 ID 作为 bitmap 偏移量
- 每日 bitmap 键记录活跃设备
- 极高存储效率：10 亿设备 ≈ 12MB 存储
- 通过 BITOP 操作进行离线分析

### 设备位置索引
- 5 分钟时间桶
- 支持跨区域设备日志查询
- 两阶段查询：检查主索引 → 查询区域 ClickHouse

### 配置同步
- 基于拉取（非推送）
- 版本 + 快照模式
- 使用版本指针的原子缓存写入
- 24 小时降级容错

## 性能要求

- **API 响应**: 99% 的检查请求 < 50ms
- **吞吐量**: 10,000+ QPS 容量
- **规模**: 1000万+ 设备
- **数据保留**: 设备索引 30 天

## 重要约束

### 安全
- 固件完整性验证（MD5/SHA256）
- 签名下载 URL（S3 Pre-signed URLs 模式）
- 每设备限流
- 跨区域调用使用 HMAC 认证

### 数据流
- 上报 API：发后即忘到 MQ，立即返回
- 跨区域无直接数据库连接
- 所有跨区域调用均为批量且幂等
- Forwarder 批量：≥1000 条或 ≥15 秒等待

### Forwarder 模式
- 跨区域传输前批量处理 MQ 消息
- 处理网络不稳定
- DLQ（死信队列）处理失败批次
- 防止主区域宕机期间的数据丢失

## 目录结构

```
wewins-fota-new/
├── docs/                          # 架构和 PRD 文档
│   ├── prd.md                     # 产品需求（中文）
│   └── FOTA 系统架构及技术说明书.md  # 技术架构（中文）
├── fota-ui/                       # 前端应用（空，未实现）
└── .agent/                        # Claude Code 代理配置
```

## 开发说明

### 当前状态
- 文档完整
- 尚无构建配置文件（无 pom.xml、package.json 或 Dockerfile）
- 无源码实现
- 项目处于规划/设置阶段

### 预期实现
- Maven 或 Gradle 用于 Java 后端
- Vue.js 或 React 用于管理前端
- Docker 多阶段构建用于主区域/区域部署
- PostgreSQL 数据库迁移脚本
- Redis 数据结构初始化脚本

### 核心业务逻辑
1. **检查更新流程**：
   - 从 Redis 缓存验证设备
   - 更新每日活跃度 bitmap
   - 匹配策略（版本、标签、灰度比例）
   - 检查时间窗口
   - 生成签名下载 URL
   - 返回响应及下次检查间隔

2. **上报流程**：
   - 接收请求 → 推送到 RabbitMQ
   - 立即返回 200 OK
   - 消费者批量写入 ClickHouse
   - UP_OK 时：异步更新 PostgreSQL 设备版本

3. **配置同步（Region 工作进程）**：
   - 每 30 秒轮询主区域版本变化
   - 版本更新时拉取快照
   - 原子切换 Redis 中的版本指针
   - 主区域不可达时继续使用已知快照

### 灰度算法
```java
// 判断设备是否命中灰度发布
int bucket = hash(imei) % 100;
boolean hits = bucket < policy.grayRate;
```

## 文档语言

项目文档主要使用中文。在贡献或参考架构决策时，请查阅 `docs/` 目录中的文件以获取详细规范。