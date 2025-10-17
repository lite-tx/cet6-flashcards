# CET6 智能单词学习工具 🎓

一个基于 Rust + Yew 的现代化 CET6 单词学习应用，集成 Live2D 看板娘和 RVC 离线语音朗读。

**✨ 完全离线运行，无需 Python 环境！**

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Audio](https://img.shields.io/badge/audio-3991_words-green.svg)

## ✨ 主要特性

### 📚 智能学习系统
- **间隔重复算法**：基于 SuperMemo 的智能复习系统
- **动态复习池**：根据掌握程度智能调整复习内容
- **自适应算法**：正确率越高，复习量自动增加
- **单词流动机制**：难词连续答对 3 次自动流入已掌握库
- **记忆衰减追踪**：长时间未复习的单词会优先复习

### 🎤 离线 RVC 语音朗读
- **完全离线**：3991 个单词的 RVC 语音已预生成，即点即播
- **点击朗读**：点击 Live2D 看板娘即可朗读当前单词
- **智能缓存**：单词切换时自动预加载，播放无延迟（<100ms）
- **内存优化**：已播放的音频保留在内存中，再次播放零延迟
- **零依赖**：无需 Python、RVC 服务器或任何后端

### 🎨 Live2D 互动
- **实时提示**：学习进度、答题结果实时显示
- **眼动追踪**：模型会跟随鼠标移动
- **随机提问**：每 5-10 分钟随机提问单词
- **多种表情**：根据学习状态显示不同表情

### 📊 学习统计
- 已掌握单词数量
- 难词本统计
- 复习池状态
- 答题正确率
- 周期结算统计

## 🚀 快速开始

### 前置要求

只需要 Rust 工具链：

- Rust 1.70+
- Trunk: `cargo install trunk`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

**不需要 Python！** 所有单词的 RVC 语音已预生成，可以直接使用。

### 安装步骤

```bash
# 1. 克隆仓库
git clone https://github.com/lite-tx/cet6-flashcards.git
cd cet6-flashcards

# 2. 解压离线语音包（如果是压缩文件）
tar -xzf cet6-audio-cache-20251016.tar.gz  # 如果需要

# 3. 启动应用
trunk serve --open
```

前端将在 `http://localhost:8080` 启动，点击右下角 Live2D 看板娘即可听到单词发音！

### 生产构建

```bash
trunk build --release
```

构建产物在 `dist/` 目录，可以部署到任何静态服务器。

## 📖 使用说明

### 基本操作

- **← →** : 切换生词
- **↑ ↓** : 切换复习单词
- **点击卡片** : 翻转查看释义
- **✓ 已掌握** : 标记为已掌握
- **★ 难词** : 加入难词本
- **点击 Live2D** : 朗读当前单词（使用离线语音）

### 学习流程

1. **学习生词**：浏览单词，标记已掌握或难词
2. **进入复习**：当词库达到 20 个单词后自动生成复习池
3. **答题测试**：根据中文释义拼写单词
4. **智能调整**：系统根据正确率调整下轮复习量

### 复习策略（间隔重复算法）

- **首次复习**：1 天后
- **第二次复习**：4 天后
- **后续复习**：间隔 = 上次间隔 × 难度系数（1.3-3.0）
- **答对效果**：增加难度系数，延长下次复习时间
- **答错效果**：降低难度系数，重置为 1 天后复习

### 复习池调整

- **正确率 100%**：下轮复习量 ×2
- **正确率 ≥50%**：下轮复习量 +1
- **正确率 <50%**：下轮复习量 ÷2（最少 1 个）

## 📦 离线语音包

### 音频包信息

本项目已预生成所有单词的 RVC 离线语音：

- **音频包**：`cet6-audio-cache-20251016.tar.gz` (293 MB)
- **单词数量**：3991 个（去重后）
- **音频格式**：WAV（高质量）
- **未压缩大小**：497 MB
- **索引文件**：`audio_cache/cache_index.json`

### 分享音频包

```bash
# 音频包已经压缩好，可以直接分享
# 接收方只需解压并运行 trunk serve 即可使用

tar -xzf cet6-audio-cache-20251016.tar.gz
trunk serve --open
```

### 离线模式工作原理

1. **零延迟播放**：音频已预生成，点击即播（<100ms）
2. **自动预加载**：切换单词时自动预加载下一个单词的音频
3. **内存缓存**：已播放的音频保留在内存中，再次播放零延迟
4. **完全离线**：无需网络连接，无需后端服务器

## 🔧 音频生成工具（高级）

**⚠️ 日常使用不需要这些工具！**

如果需要重新生成音频（使用不同的 RVC 模型或自定义语音），相关工具已打包：

- **工具包**：`audio-generation-tools.tar.gz` (214 KB)
- **说明文档**：解压后查看 `AUDIO_GENERATION_README.md`

### 何时需要音频生成工具？

1. **自定义语音**：想用不同的 RVC 模型生成专属声音
2. **添加新单词**：词库更新后生成新单词的音频
3. **修复损坏**：某些音频文件损坏需要重新生成

### 使用音频生成工具

```bash
# 1. 解压工具包
tar -xzf audio-generation-tools.tar.gz

# 2. 查看详细说明
cat AUDIO_GENERATION_README.md

# 3. 需要 Python 环境和 RVC 模型
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# 4. 生成音频（需要 2-3 小时）
./generate_audio.sh
```

**工具包包含的文件**：
- `rvc_server.py` - RVC 语音合成服务器
- `generate_audio_cache.py` - 批量生成音频缓存脚本
- `generate_audio.sh` - 一键启动脚本
- `compress_audio.sh` - 压缩音频包脚本
- `quick_guide.sh` - 快速使用指南
- `start_rvc_server.sh` - 启动服务器脚本
- `AUDIO_GENERATION_README.md` - 详细文档

## ⚙️ 配置说明

### 前端配置

在 `index.html` 中的配置（默认已优化）：

```javascript
// 使用离线缓存（已启用）
const USE_OFFLINE_CACHE = true;

// 使用 RVC（离线音频）
const USE_RVC = true;
```

**默认配置即可满足需求，无需修改。**

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

**可用模型**：
- `models/UG/ugofficial.model3.json` - UG 官方模型
- `models/shizuku/shizuku.model3.json` - Shizuku 模型

## 🔧 故障排除

### 离线语音相关

#### 音频无法播放

**症状**：点击 Live2D 没有声音

**排查步骤**：
1. 检查 `audio_cache/` 目录是否存在
2. 检查 `audio_cache/cache_index.json` 是否存在
3. 打开浏览器控制台（F12）查看是否有错误
4. 确认构建时 `audio_cache` 目录被复制到 `dist/`

**解决方案**：
```bash
# 检查音频文件
ls audio_cache/*.wav | wc -l  # 应输出 3991

# 重新构建
trunk build --release
```

#### 缓存索引加载失败

**症状**：控制台显示 "无法加载音频缓存索引"

**解决方案**：
```bash
# 重新解压音频包
tar -xzf cet6-audio-cache-20251016.tar.gz
```

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
- 检查 `models/` 目录是否被复制到 `dist/`

## 📁 项目结构

```
cet6-flashcards/
├── src/
│   ├── main.rs                      # 主应用逻辑（间隔重复算法）
│   └── models.rs                    # 数据模型定义
├── audio_cache/                      # 离线语音缓存（必需）
│   ├── cache_index.json             # 音频索引文件
│   ├── ability.wav                  # 单词音频文件
│   └── ...                          # 3991 个音频文件
├── models/                           # Live2D 模型
│   ├── UG/                          # UG 官方模型
│   └── shizuku/                     # Shizuku 模型
├── index.html                        # 前端页面（包含音频缓存管理）
├── 4-CET6-顺序.json                  # 单词数据
├── Cargo.toml                        # Rust 依赖
├── README.md                         # 本文档
├── cet6-audio-cache-20251016.tar.gz  # 音频包（分发用）
└── audio-generation-tools.tar.gz     # 音频生成工具（可选）
```

**日常使用只需要**：
- `src/`、`models/`、`audio_cache/`、`index.html`、`Cargo.toml`、`4-CET6-顺序.json`

**可选文件**：
- `audio-generation-tools.tar.gz` - 仅重新生成音频时需要
- `11_RVC/` - 仅自定义 RVC 模型时需要

## 🛠️ 技术栈

### 前端（核心）
- **Rust** - 系统编程语言
- **Yew** - Rust 的 React-like 框架
- **WebAssembly** - 高性能 Web 应用
- **oh-my-live2d** - Live2D 渲染库

### 离线语音系统
- **AudioCacheManager** - 自定义音频缓存管理器
- **MutationObserver** - DOM 变化监听，自动预加载
- **Blob URL** - 内存缓存机制
- **预生成 RVC 音频** - 3991 个 WAV 文件

### 音频生成（可选，仅自定义语音需要）
- **Python 3.10+** - 脚本语言
- **FastAPI** - Web 框架
- **edge-tts** - 微软 Edge TTS
- **rvc-python** - RVC 推理库
- **PyTorch 2.3.0** - 深度学习框架

## 📈 性能数据

### 离线模式（默认）
- **首次播放**：<100ms（内存缓存）
- **文件加载**：<200ms（从磁盘）
- **自动预加载**：切换单词时后台加载
- **内存占用**：约 50MB（已播放音频）
- **网络请求**：0（完全离线）

### 构建大小
- **WASM 包**：约 500KB（gzip 后）
- **音频包**：497MB（未压缩），293MB（压缩）
- **总大小**：约 800MB（包含所有资源）

## 🎯 最佳实践

### 使用建议

1. **开箱即用**：克隆仓库后直接 `trunk serve`，无需任何配置
2. **备份音频**：保存 `cet6-audio-cache-20251016.tar.gz`，避免重复生成
3. **分享给朋友**：发送音频包 + 项目代码，对方即可立即使用
4. **自定义语音**：使用 `audio-generation-tools.tar.gz` 重新生成

### 部署到生产环境

```bash
# 构建生产版本
trunk build --release

# 部署 dist/ 目录到任何静态服务器
# 例如：Nginx、GitHub Pages、Vercel 等
```

**注意**：确保 `audio_cache/` 目录包含在部署包中。

## 🗺️ 开发路线图

- [x] 基于间隔重复的智能复习算法
- [x] RVC 语音朗读功能
- [x] Live2D 看板娘互动
- [x] 离线语音缓存系统
- [x] 自动预加载机制
- [ ] 支持更多 TTS 引擎（VITS、GPT-SoVITS）
- [ ] 添加单词发音音标显示
- [ ] 支持自定义学习计划
- [ ] 添加学习统计图表
- [ ] 支持单词本导入/导出
- [ ] 移动端适配优化
- [ ] PWA 支持（完全离线使用）

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
- [SuperMemo](https://www.supermemo.com/) - 间隔重复算法灵感

## 📧 联系方式

- GitHub: [@lite-tx](https://github.com/lite-tx)
- Issues: [提交问题](https://github.com/lite-tx/cet6-flashcards/issues)

---

⭐ 如果这个项目对你有帮助，请给个 Star！

## 💡 使用提示

### 控制台调试命令

打开浏览器控制台（F12），可以使用以下命令：

```javascript
// 测试朗读当前单词
speakWord()

// 预加载指定单词
preloadWordAudio('ability')

// 查看缓存状态
console.log(audioCache.cache)

// 查看缓存索引
console.log(audioCache.cacheIndex)

// 随机提问
showRandomQuiz()
```

### 常见日志说明

- `📂 使用缓存音频` - 找到了离线缓存
- `💾 从内存缓存加载` - 从内存中读取
- `💿 从文件缓存加载` - 从文件中读取
- `🌐 调用 RVC 服务器在线生成` - 使用在线生成
- `🎵 预加载音频` - 成功预加载
- `🔄 检测到单词切换` - 自动预加载触发
