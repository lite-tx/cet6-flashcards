# CET6 智能单词学习工具 🎓

一个基于 Rust + Yew 的现代化 CET6 单词学习应用，集成 Live2D 看板娘和 RVC 语音朗读功能。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Python](https://img.shields.io/badge/python-3.10+-blue.svg)

## ✨ 主要特性

### 📚 智能学习系统
- **动态复习池**：根据掌握程度智能调整复习内容
- **自适应算法**：正确率越高，复习量自动增加
- **单词流动机制**：难词连续答对 3 次自动流入已掌握库
- **记忆衰减**：长时间未复习的单词会降低熟练度

### 🎤 RVC 语音朗读
- **点击朗读**：点击 Live2D 看板娘即可朗读当前单词
- **RVC 模型支持**：使用自定义 RVC 模型进行语音转换
- **智能降级**：RVC 不可用时自动回退到 Edge-TTS
- **GPU 加速**：支持 NVIDIA GPU 加速推理

### 🎨 Live2D 互动
- **实时提示**：学习进度、答题结果实时显示
- **眼动追踪**：模型会跟随鼠标移动
- **多种表情**：根据学习状态显示不同表情

### 📊 学习统计
- 已掌握单词数量
- 难词本统计  
- 复习池状态
- 答题正确率

## 🚀 快速开始

### 前置要求

#### 前端（必需）
- Rust 1.70+
- Trunk: `cargo install trunk`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

#### 后端（RVC 功能，可选）
- Python 3.10+
- NVIDIA GPU（推荐，CPU 也可用但较慢）
- CUDA 12.1+（如使用 GPU）

### 安装步骤

#### 1. 克隆仓库

```bash
git clone https://github.com/lite-tx/cet6-flashcards.git
cd cet6-flashcards
```

#### 2. 启动前端

```bash
# 开发模式（热重载）
trunk serve

# 生产构建
trunk build --release
```

前端将在 `http://localhost:8080` 启动。

#### 3. 启动 RVC 后端（可选）

如果需要 RVC 语音朗读功能：

```bash
# 创建虚拟环境
python3 -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate

# 安装依赖
pip install -r requirements.txt

# 安装兼容版本的 PyTorch（重要！）
pip install torch==2.3.0+cu121 torchaudio==2.3.0+cu121 \
    --extra-index-url https://download.pytorch.org/whl/cu121

# 启动 RVC 服务器
python rvc_server.py
```

RVC 服务将在 `http://localhost:8765` 启动。

### 4. 准备 RVC 模型（可选）

将你的 RVC 模型文件放置到 `11_RVC/` 目录：

```
11_RVC/
├── your_model.pth              # RVC 模型文件
└── your_index.index            # 索引文件（可选）
```

修改 `rvc_server.py` 中的路径：

```python
RVC_MODEL_PATH = "11_RVC/your_model.pth"
RVC_INDEX_PATH = "11_RVC/your_index.index"
```

## 📖 使用说明

### 基本操作

- **← →** : 切换生词
- **↑ ↓** : 切换复习单词
- **点击卡片** : 翻转查看释义
- **✓ 已掌握** : 标记为已掌握
- **★ 难词** : 加入难词本
- **点击 Live2D** : 朗读当前单词

### 学习流程

1. **学习生词**：浏览单词，标记已掌握或难词
2. **进入复习**：当词库达到 20 个单词后自动生成复习池
3. **答题测试**：根据中文释义拼写单词
4. **智能调整**：系统根据正确率调整下轮复习量

### 复习策略

- **正确率 100%**：下轮复习量 ×2
- **正确率 ≥50%**：下轮复习量 +1  
- **正确率 <50%**：下轮复习量 ÷2（最少 1 个）

## ⚙️ 配置说明

### 前端配置

在 `index.html` 中可以配置：

```javascript
// RVC 服务器地址
const RVC_SERVER_URL = 'http://localhost:8765';

// 是否使用 RVC（false 使用浏览器 TTS）
const USE_RVC = true;
```

### Live2D 模型配置

在 `index.html` 中修改 Live2D 模型路径：

```javascript
models: [
    {
        path: 'models/你的模型/model.json',
        scale: 0.2,
        position: [240, 260]
    }
]
```

## 🔧 故障排除

### RVC 相关问题

#### PyTorch 版本不兼容

**症状**：`weights_only` 错误或 `'tuple' object has no attribute 'dtype'`

**解决**：使用 PyTorch 2.3.0
```bash
pip install torch==2.3.0+cu121 torchaudio==2.3.0+cu121 \
    --extra-index-url https://download.pytorch.org/whl/cu121
```

#### CORS 错误

**症状**：浏览器提示跨域错误

**解决**：
- 确保 RVC 服务器在运行
- 检查 `RVC_SERVER_URL` 配置
- 重启 RVC 服务器

#### 没有声音

**检查清单**：
1. RVC 服务器是否运行：`curl http://localhost:8765/`
2. 浏览器控制台是否有错误
3. 尝试设置 `USE_RVC = false` 使用浏览器 TTS

### 前端相关问题

#### Trunk 构建失败

```bash
# 重新安装 trunk
cargo install trunk --force

# 添加 wasm 目标
rustup target add wasm32-unknown-unknown
```

#### Live2D 不显示

- 检查模型路径是否正确
- 打开浏览器控制台查看错误
- 确保模型文件完整

## 📁 项目结构

```
cet6-flashcards/
├── src/
│   ├── main.rs              # 主应用逻辑
│   └── models.rs            # 数据模型定义
├── 11_RVC/                  # RVC 模型文件（需自行准备）
├── models/                  # Live2D 模型
│   ├── UG/                 # 示例模型
│   └── shizuku/            # 示例模型
├── index.html              # 前端页面
├── rvc_server.py           # RVC 后端服务
├── requirements.txt        # Python 依赖
├── Cargo.toml             # Rust 依赖
└── README.md              # 本文档
```

## 🛠️ 技术栈

### 前端
- **Rust** - 系统编程语言
- **Yew** - Rust 的 React-like 框架
- **WebAssembly** - 高性能 Web 应用
- **oh-my-live2d** - Live2D 渲染库

### 后端
- **Python 3.10+** - 脚本语言
- **FastAPI** - 现代 Web 框架
- **edge-tts** - 微软 Edge TTS
- **rvc-python** - RVC 推理库
- **PyTorch 2.3.0** - 深度学习框架

## 📈 性能优化

- 首次 RVC 转换：10-30秒（模型加载）
- 后续转换：1-3秒
- Edge-TTS 降级：1-2秒
- GPU 加速可显著提升速度

## 🗺️ 开发路线图

- [ ] 支持更多 TTS 引擎（VITS、GPT-SoVITS）
- [ ] 添加单词发音音标显示
- [ ] 支持自定义学习计划
- [ ] 添加学习统计图表
- [ ] 支持单词本导入/导出
- [ ] 移动端适配优化

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- [Yew](https://yew.rs/) - Rust WebAssembly 框架
- [oh-my-live2d](https://oml2d.com/) - Live2D 集成方案
- [edge-tts](https://github.com/rany2/edge-tts) - 免费 TTS 服务
- [RVC](https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI) - 语音转换技术

## 📧 联系方式

- GitHub: [@lite-tx](https://github.com/lite-tx)
- Issues: [提交问题](https://github.com/lite-tx/cet6-flashcards/issues)

---

⭐ 如果这个项目对你有帮助，请给个 Star！
