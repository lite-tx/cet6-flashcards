# CET6 背单词工具

一个使用 Rust + Yew + WebAssembly 构建的现代化网页背单词应用。

## 功能特性

- 📚 **单词卡片** - 点击卡片翻转查看释义
- 🎯 **学习进度** - 自动保存学习进度到浏览器本地存储
- ✅ **已掌握标记** - 标记已经掌握的单词
- ⭐ **难词本** - 标记难记的单词方便复习
- 🎲 **随机模式** - 随机选择单词进行复习
- 📊 **统计信息** - 实时显示学习统计
- 📱 **响应式设计** - 支持手机、平板和桌面设备
- 🎨 **精美UI** - 渐变背景和流畅动画

## 技术栈

- **Rust** - 系统编程语言
- **Yew** - Rust 的前端框架
- **WebAssembly** - 高性能 Web 运行时
- **Serde** - JSON 序列化/反序列化

## 前置要求

1. 安装 Rust：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. 安装 Trunk（Yew 构建工具）：
```bash
cargo install trunk
```

3. 添加 WebAssembly 目标：
```bash
rustup target add wasm32-unknown-unknown
```

## 快速开始

### 开发模式

在项目目录下运行：

```bash
trunk serve --open
```

应用将在 `http://127.0.0.1:8080` 启动并自动在浏览器中打开。

### 生产构建

构建优化版本：

```bash
trunk build --release
```

构建产物将生成在 `dist/` 目录中，可以直接部署到任何静态网站托管服务。

### 部署到 GitHub Pages

1. 构建项目：
```bash
trunk build --release --public-url /CET6/
```

2. 将 `dist/` 目录内容推送到 `gh-pages` 分支

3. 在 GitHub 仓库设置中启用 Pages

## 使用说明

### 基本操作

- **查看释义**：点击单词卡片翻转查看释义和短语
- **切换单词**：使用"上一个"/"下一个"按钮导航
- **标记掌握**：点击"✓ 已掌握"标记熟练掌握的单词
- **添加难词**：点击"★ 难词"将难记的单词加入难词本
- **随机复习**：点击"🎲 随机"随机跳转到一个单词

### 进度保存

学习进度会自动保存在浏览器的 LocalStorage 中，包括：
- 当前学习位置
- 已掌握的单词列表
- 难词本列表

关闭浏览器后再次打开会自动恢复进度。

### 清除进度

如果需要重新开始，可以在浏览器开发者工具中清除 LocalStorage：
1. 按 F12 打开开发者工具
2. 进入 Application/应用 标签
3. 找到 Local Storage
4. 删除 `cet6_progress` 键

## 项目结构

```
CET6/
├── Cargo.toml              # Rust 项目配置
├── index.html              # HTML 模板（包含 CSS）
├── 4-CET6-顺序.json        # 单词数据
├── src/
│   ├── main.rs            # 主应用逻辑
│   └── models.rs          # 数据模型定义
└── README.md              # 项目说明
```

## 数据格式

单词数据采用 JSON 格式：

```json
{
  "word": "abandon",
  "translations": [
    {
      "translation": "放弃",
      "type": "v"
    }
  ],
  "phrases": [
    {
      "phrase": "abandon hope",
      "translation": "放弃希望"
    }
  ]
}
```

## 浏览器兼容性

支持所有现代浏览器：
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## 性能优化

- 使用 WebAssembly 提供接近原生的性能
- 懒加载单词数据
- CSS 动画硬件加速
- 生产构建启用代码压缩和优化

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
