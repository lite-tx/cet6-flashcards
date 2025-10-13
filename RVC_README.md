# RVC 语音朗读功能说明

## 功能概述

点击 Live2D 小人即可使用 RVC 模型朗读当前单词，提供更自然、更个性化的语音体验。

## 快速启动

### 1. 启动 RVC 后端服务

在项目根目录执行：

```bash
./start_rvc_server.sh
```

或者手动启动：

```bash
# 创建虚拟环境（首次）
python3 -m venv venv

# 激活虚拟环境
source venv/bin/activate

# 安装依赖
pip install -r requirements.txt

# 启动服务
python rvc_server.py
```

服务将在 `http://localhost:8765` 启动。

### 2. 启动前端应用

在另一个终端窗口：

```bash
trunk serve
```

### 3. 使用

- 打开浏览器访问 `http://localhost:8080`
- 点击右下角的 Live2D 小人
- 听到用 RVC 模型生成的语音朗读当前单词

## 配置选项

### 前端配置（index.html）

```javascript
const RVC_SERVER_URL = 'http://localhost:8765';  // RVC 服务器地址
const USE_RVC = true;  // true=使用RVC, false=使用浏览器TTS
```

### 后端配置（rvc_server.py）

```python
RVC_MODEL_PATH = "11_RVC/manbo_2_e100.pth"  # RVC 模型路径
RVC_INDEX_PATH = "11_RVC/added_IVF724_Flat_nprobe_1_v2.index"  # 索引文件
EDGE_VOICE = "en-US-JennyNeural"  # Edge TTS 语音（基础音源）
```

## 工作原理

1. **文本输入**：前端发送单词到 RVC 服务器
2. **基础 TTS**：使用 edge-tts 生成英文语音
3. **RVC 转换**：使用 RVC 模型转换为目标声音
4. **音频播放**：返回音频给前端播放

## 故障排除

### RVC 服务器无法启动

检查依赖是否正确安装：

```bash
source venv/bin/activate
pip install -r requirements.txt
```

### 点击无声音

1. 打开浏览器控制台（F12）查看错误信息
2. 确认 RVC 服务器是否正在运行
3. 检查是否有 CORS 错误

### 自动回退到浏览器 TTS

如果 RVC 服务不可用，系统会自动回退到浏览器内置 TTS，确保功能始终可用。

## 高级功能

### 更换 RVC 模型

1. 将新的 `.pth` 模型文件和 `.index` 文件放到 `11_RVC/` 目录
2. 修改 `rvc_server.py` 中的路径
3. 重启 RVC 服务器

### 调整语音参数

在 `rvc_server.py` 中修改：

- `rate`: 语速调整（如 "+10%"）
- `pitch`: 音调调整（如 "+50Hz"）
- RVC pitch: 音高转换（默认 0）

## API 文档

### POST /tts

生成 TTS 语音

**请求体：**
```json
{
  "text": "hello",
  "rate": "+0%",
  "pitch": "+0Hz"
}
```

**响应：**
- 成功：音频文件（audio/wav 或 audio/mpeg）
- 失败：HTTP 错误码和错误信息

### GET /

健康检查和服务状态

### GET /health

服务健康状态

## 依赖说明

- **fastapi**: Web 框架
- **edge-tts**: 微软 Edge TTS（免费）
- **rvc-python**: RVC 推理引擎
- **uvicorn**: ASGI 服务器

## 性能优化

- 首次生成可能需要 3-5 秒（模型加载）
- 后续生成约 1-2 秒
- 可以通过缓存常用单词来加速

## 注意事项

- RVC 服务器需要持续运行才能使用语音功能
- 如果 RVC 不可用，会自动使用浏览器 TTS
- 建议使用 Chrome/Edge 浏览器以获得最佳体验
