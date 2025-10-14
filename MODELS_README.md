# Live2D 模型配置说明

由于版权原因，Live2D 模型文件未包含在本仓库中。你需要自行准备 Live2D 模型。

## 📁 模型目录结构

将你的 Live2D 模型放置在 `models/` 目录下：

```
models/
├── your_model_name/
│   ├── model.model3.json    # 模型主配置文件
│   ├── *.moc3               # 模型网格文件
│   ├── *.physics3.json      # 物理配置
│   ├── textures/            # 贴图文件夹
│   └── motions/             # 动作文件夹
```

## 🎨 推荐的免费 Live2D 模型来源

### 1. Live2D 官方示例模型
- **地址**：https://www.live2d.com/download/sample-data/
- **许可**：可用于学习和非商业用途
- **模型**：Haru, Hiyori, Mark, Rice 等

### 2. oh-my-live2d 示例模型
- **地址**：https://github.com/oh-my-live2d/oh-my-live2d
- **说明**：查看项目文档中的模型链接

### 3. VRoid Hub
- **地址**：https://hub.vroid.com/
- **说明**：需要转换为 Live2D 格式

## ⚙️ 配置模型

在 `index.html` 中修改模型路径：

```javascript
models: [
    {
        path: 'models/your_model_name/model.model3.json',
        scale: 0.2,  // 调整缩放
        position: [240, 260]  // 调整位置 [x, y]
    }
]
```

## 📝 注意事项

1. **版权合规**：
   - 仅使用有明确许可的模型
   - 遵守模型作者的使用条款
   - 商业使用需获得授权

2. **模型格式**：
   - 支持 Live2D Cubism 3.0+ 格式
   - 文件扩展名：`.model3.json`

3. **性能优化**：
   - 推荐贴图大小：1024x1024 或 2048x2048
   - 避免使用过大的模型文件

## 🔗 相关资源

- [Live2D 官方文档](https://docs.live2d.com/)
- [oh-my-live2d 文档](https://oml2d.com/)
- [Live2D Cubism Editor](https://www.live2d.com/en/download/cubism/)
