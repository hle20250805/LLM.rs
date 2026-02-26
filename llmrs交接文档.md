# llm.rs 项目交接文档

## 一、项目代码结构

```
LLM.rs/
├── src/             # 源代码目录
│   ├── backend/      # 后端实现
│   │   └── candle.rs  # Candle 后端实现
│   ├── backend.rs    # 后端接口定义
│   ├── config.rs     # 配置文件处理
│   ├── lib.rs        # 库定义
│   ├── main.rs       # 主入口
│   ├── queue.rs      # 队列实现
│   └── types.rs      # 类型定义
├── models/           # 模型目录
│   └── Yuan-embedding-2.0-zh/  # 元模型
├── Dockerfile        # Docker 构建文件
├── Cargo.toml        # Rust 项目配置
└── config.toml       # 应用配置文件
```

## 二、项目部署（Docker 容器）

### 1. 本地构建 Docker 镜像

1. 进入项目目录：`cd /path/to/LLM.rs`
2. 构建镜像：`docker build -t h20260224/llmrs:latest .`
3. 导出镜像：`docker save -o llmrs.tar h20260224/llmrs:latest`

### 2. 服务器部署

1. **停止并禁用旧服务**：
   ```bash
   # 停止旧服务
   sudo systemctl stop llmrs.service
   # 禁用旧服务
   sudo systemctl disable llmrs.service
   # 验证服务状态
   sudo systemctl status llmrs.service
   ```

2. **将镜像上传到服务器**：
   - 使用 FinalShell 等工具将 `llmrs.tar` 文件上传到服务器的 `/data/model/LLM/` 目录

3. **加载镜像**：
   ```bash
   docker load -i /data/model/LLM/llmrs.tar
   ```

4. **启动容器**：
   ```bash
   docker run -d -p 3000:3000 --name llmrs-container -v /data/model/LLM/LLM.rs/models:/app/models h20260224/llmrs:latest
   ```

5. **查看容器状态**：
   ```bash
   docker ps | grep llmrs-container
   ```

6. **查看服务日志**：
   ```bash
   docker logs llmrs-container
   ```

## 三、服务访问方式

- **访问地址**：`http://服务器IP:3000`
- **服务端口**：3000
- **服务状态**：已成功启动并运行
- **模型**：已加载 Yuan-embedding-2.0-zh 模型

## 四、服务使用方法

### 1. API 接口

服务提供以下 API 接口：

- **GET /health**：健康检查
  - 响应：`{"status": "ok"}`

- **POST /embed**：生成文本嵌入向量
  - 请求体：`{"texts": ["待嵌入的文本"], "normalize_embeddings": true, "batch_size": 1}`
  - 响应：`{"embeddings": [[0.1, 0.2, ...]]}`

- **POST /v1/embeddings**：OpenAI 兼容接口
  - 请求体：`{"input": "待嵌入的文本", "model": "Yuan-embedding-2.0-zh"}`
  - 响应：符合 OpenAI 格式的嵌入响应

### 2. 测试示例

使用 curl 测试服务：

```bash
# 测试健康检查
curl -X GET http://localhost:3000/health

# 测试嵌入生成
curl -X POST http://localhost:3000/embed -H "Content-Type: application/json" -d '{"texts": ["测试文本"], "normalize_embeddings": true, "batch_size": 1}'

# 测试 OpenAI 兼容接口
curl -X POST http://localhost:3000/v1/embeddings -H "Content-Type: application/json" -d '{"input": "测试文本", "model": "Yuan-embedding-2.0-zh"}'
```

## 五、代码修改

### 1. 修改配置

配置文件位于 `config.toml`，可以修改以下配置：
- 模型路径
- 服务端口
- 日志级别

### 2. 添加新功能

如果需要添加新功能，主要修改以下文件：
- `src/main.rs`：添加新的 API 端点
- `src/backend/`：实现新的后端功能
- `src/types.rs`：添加新的数据类型

## 六、故障排查

### 1. 常见问题

- **模型加载失败**：确保 `models` 目录已正确挂载，且包含 `Yuan-embedding-2.0-zh` 模型
- **端口冲突**：如果 3000 端口被占用，先停止占用端口的进程
- **依赖问题**：确保 Docker 镜像使用了正确的基础镜像（Debian bookworm-slim）

### 2. 查看日志

使用以下命令查看容器日志：
```bash
docker logs llmrs-container
```

### 3. 重启服务

如果服务出现问题，可以重启容器：
```bash
docker restart llmrs-container
```

## 七、Docker 镜像管理

### 1. 查看镜像

```bash
docker images | grep llmrs
```

### 2. 删除镜像

```bash
docker rmi h20260224/llmrs:latest
```

### 3. 重新构建镜像

如果代码有修改，需要重新构建镜像：
```bash
docker build -t h20260224/llmrs:latest .
docker push h20260224/llmrs:latest
```

## 八、总结

llm.rs 项目是一个基于 Rust 和 Candle 库的文本嵌入服务，提供了简单的 API 接口用于生成文本的嵌入向量。项目已通过 Docker 容器成功部署在服务器上，使用 3000 端口提供服务。

服务的核心功能是将输入文本转换为高维向量表示，可用于文本相似度计算、聚类分析等场景。服务支持标准嵌入接口和 OpenAI 兼容接口，方便与各种应用集成。

所有 API 接口都已测试通过，服务运行正常。如果需要对服务进行扩展或修改，可以参考代码结构和部署文档进行操作。