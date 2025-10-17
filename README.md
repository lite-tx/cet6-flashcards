# CET6 智能单词学习工具

一个基于 Rust + Yew 的现代化 CET6 单词学习应用，集成智能复习算法、Live2D 互动和语音朗读功能。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

## 主要特性

### 智能学习系统
- **间隔重复算法**：基于 SuperMemo 算法的智能复习系统
- **动态复习池**：自动根据记忆曲线调整复习内容
- **单词流动机制**：难词连续答对 3 次自动流入已掌握库
- **自适应学习**：根据正确率自动调整复习量

### 互动功能
- **Live2D 看板娘**：实时显示学习进度和反馈
- **语音朗读**：支持点击朗读单词发音
- **随机提问**：定时从复习池随机抽取单词测试

### 学习统计
- 已掌握单词数量
- 难词本统计
- 复习池和缓存池状态
- 答题正确率跟踪

## 快速开始

### 前置要求

- Rust 1.70+
- Trunk: `cargo install trunk`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

### 安装步骤

```bash
# 1. 克隆仓库
git clone https://github.com/lite-tx/cet6-flashcards.git
cd cet6-flashcards

# 2. 准备必需资源（见下方"资源准备"部分）

# 3. 启动应用
trunk serve --open
```

前端将在 `http://localhost:8080` 启动。

### 生产构建

```bash
trunk build --release
```

构建产物在 `dist/` 目录，可以部署到任何静态服务器。

## 资源准备

**重要提示**：由于版权原因，以下资源文件未包含在本仓库中，需要自行准备：

### 1. Live2D 模型（必需）

在 `models/` 目录下放置 Live2D 模型文件。推荐的模型结构：

```
models/
└── [模型名称]/
    ├── model3.json
    ├── *.moc3
    ├── *.physics3.json
    └── textures/
```

**获取途径**：
- [Live2D 官方网站](https://www.live2d.com/)
- [Live2D Cubism SDK](https://www.live2d.com/download/cubism-sdk/)
- 社区提供的免费模型（请遵守相应许可）

**配置方法**：
编辑 `index.html` 第 448 行，修改模型路径：
```javascript
models: [
    {
        path: 'models/你的模型/model.json',
        scale: 0.2,
        position: [240, 260]
    }
]
```

### 2. 音频文件（可选）

如需离线语音朗读功能，在 `audio_cache/` 目录下放置音频文件：

```
audio_cache/
├── cache_index.json      # 音频索引文件
├── word1.wav
├── word2.wav
└── ...
```

**生成方法**：
- 使用 TTS 工具（如 edge-tts）生成单词发音
- 可选：使用 RVC 进行语音转换
- 参考原 README 中的音频生成工具说明

**cache_index.json 格式**：
```json
{
  "total_count": 3991,
  "words": {
    "ability": "ability.wav",
    "abandon": "abandon.wav"
  }
}
```

如不准备音频文件，应用会回退使用浏览器内置 TTS。

### 3. oh-my-live2d 库（已包含）

`lib/oh-my-live2d.min.js` 已包含在仓库中（开源库）。

## 使用说明

### 基本操作

- **← →** : 切换生词
- **↑ ↓** : 切换复习单词
- **点击卡片** : 翻转查看释义
- **✓ 已掌握** : 标记为已掌握
- **★ 难词** : 加入难词本
- **点击 Live2D** : 朗读当前单词（需配置模型和音频）

### 学习流程

1. **浏览生词**：使用左右箭头浏览单词，标记已掌握或难词
2. **智能复习**：当词库达到一定数量后，系统自动生成复习池
3. **答题测试**：使用上下箭头进入复习模式，根据中文释义拼写单词
4. **动态调整**：系统根据正确率自动调整下轮复习量

### 复习策略

- **首次复习**：1 天后
- **第二次复习**：4 天后
- **后续复习**：间隔 = 上次间隔 × 难度系数（1.3-3.0）
- **答对**：增加难度系数，延长下次复习时间
- **答错**：降低难度系数，重置为 1 天后复习

## 项目结构

```
cet6-flashcards/
├── src/
│   ├── main.rs                 # 主应用逻辑
│   └── models.rs               # 数据模型定义
├── models/                      # Live2D 模型（需自行准备）
├── audio_cache/                 # 音频文件（可选，需自行准备）
├── lib/
│   └── oh-my-live2d.min.js     # Live2D 库
├── index.html                   # 前端页面
├── 4-CET6-顺序.json            # 单词数据
├── Cargo.toml                   # Rust 依赖
└── README.md                    # 本文档
```

## 技术栈

- **前端框架**：Rust + Yew + WebAssembly
- **UI 渲染**：HTML5 + CSS3
- **Live2D**：oh-my-live2d
- **存储**：LocalStorage
- **TTS**：浏览器 Web Speech API（可选 RVC）

## 开发路线图

- [x] 基于间隔重复的智能复习算法
- [x] Live2D 看板娘互动
- [x] 语音朗读功能
- [x] 自动预加载机制
- [ ] 支持自定义单词本
- [ ] 添加学习统计图表
- [ ] 移动端适配优化
- [ ] PWA 支持

## 常见问题

### Q: 如何获取 Live2D 模型？
A: 可以从 Live2D 官方网站下载，或使用社区提供的免费模型（请遵守相应许可）。

### Q: 没有音频文件能用吗？
A: 可以！应用会自动回退使用浏览器内置 TTS 进行朗读。

### Q: 如何自定义单词库？
A: 编辑 `4-CET6-顺序.json` 文件，按照现有格式添加单词数据。

### Q: 为什么 Live2D 不显示？
A: 检查 `models/` 目录是否存在且包含正确的模型文件，并在 `index.html` 中配置正确的路径。

## 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

**注意**：本项目不包含以下资源：
- Live2D 模型文件（需自行准备，遵守相应版权）
- 音频文件（需自行生成，遵守相应版权）

使用第三方资源时，请遵守其原始许可证和版权声明。

## 致谢

- [Yew](https://yew.rs/) - Rust WebAssembly 框架
- [oh-my-live2d](https://oml2d.com/) - Live2D 集成方案
- [SuperMemo](https://www.supermemo.com/) - 间隔重复算法灵感

## 联系方式

- GitHub: [@lite-tx](https://github.com/lite-tx)
- Issues: [提交问题](https://github.com/lite-tx/cet6-flashcards/issues)

---

如果这个项目对你有帮助，请给个 Star！
