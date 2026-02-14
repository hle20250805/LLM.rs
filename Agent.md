# LLM.rs — Agent 任务说明

## 目标
把现有 YUAN_MODEL（Python + FastAPI + SentenceTransformer）迁移为 Rust 版服务，提供 OpenAI 兼容 /v1/embeddings API，并内置队列与并发控制，支持模型可替换，后续可切换到 vLLM 或其他后端。

## 范围与约束
- 语言：Rust
- API：OpenAI 兼容 /v1/embeddings；保留 /embed 兼容接口
- 队列：可配置并发与容量
- 模型：通过配置替换模型；初期可使用 Python 后端代理
- 目标交付：
  - 两天内完成“核心功能”（API + 队列 + 后端代理）
  - 七天内完善文档并发布到 GitHub

## 里程碑
1. **MVP (2 天内)**
   - Rust HTTP 服务可启动
   - /v1/embeddings 与 /embed 可用
   - 队列可配置（worker/容量）
   - 后端代理到现有 Python 服务
2. **增强 (7 天内)**
   - README 完整
   - 配置化模型加载与切换说明
   - 基础性能与并发说明

## 接口对齐
- 现有 Python: POST /embed
- 目标: POST /v1/embeddings (OpenAI 兼容)

## 可配置项（环境变量）
- HOST / PORT
- BACKEND_URL
- NORMALIZE_EMBEDDINGS
- BATCH_SIZE
- WORKERS
- QUEUE_CAPACITY
- MODEL_NAME

## 验收标准
- 本地启动 Rust 服务后，可以调用 /v1/embeddings 得到向量
- 队列生效（并发受 WORKERS 控制）
- README 能指导用户运行与切换模型
