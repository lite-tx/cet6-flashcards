#!/bin/bash
# RVC TTS Server 启动脚本

echo "🚀 启动 RVC TTS Server..."

# 检查 Python 虚拟环境
if [ ! -d "venv" ]; then
    echo "📦 创建 Python 虚拟环境..."
    python3 -m venv venv
fi

# 激活虚拟环境
source venv/bin/activate

# 安装依赖
echo "📥 安装依赖..."
pip install -q -r requirements.txt

# 启动服务器
echo "✅ 启动服务器..."
python rvc_server.py
