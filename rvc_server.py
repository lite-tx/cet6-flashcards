#!/usr/bin/env python3
"""
RVC TTS Server - 将文本转换为带有 RVC 声音的语音
"""
import os
import asyncio
import tempfile
import logging
from pathlib import Path
from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse, Response
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import edge_tts
import subprocess

# 配置日志
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="RVC TTS Server")

# 允许跨域请求
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# RVC 配置
RVC_MODEL_PATH = "11_RVC/manbo_2_e100.pth"
RVC_INDEX_PATH = "11_RVC/added_IVF724_Flat_nprobe_1_v2.index"

# Edge-TTS 配置 (英文女声)
EDGE_VOICE = "en-US-JennyNeural"  # 可选: en-GB-SoniaNeural, en-US-AriaNeural

# 尝试导入 RVC Python API
try:
    from rvc_python.infer import RVCInference
    RVC_AVAILABLE = True
    RVC_INFERENCE = None  # 延迟初始化
    logger.info("✅ RVC Python API 导入成功")
except ImportError as e:
    RVC_AVAILABLE = False
    RVC_INFERENCE = None
    logger.warning(f"⚠️ RVC Python API 导入失败: {e}")


class TTSRequest(BaseModel):
    text: str
    rate: str = "+0%"  # 语速调整
    pitch: str = "+0Hz"  # 音调调整


def rvc_convert(input_audio: str, output_audio: str) -> bool:
    """
    使用 RVC 转换音频
    返回: True 表示成功，False 表示失败
    """
    global RVC_INFERENCE

    try:
        logger.info(f"🎵 开始 RVC 转换: {input_audio} -> {output_audio}")

        # 初始化 RVC 推理器（只初始化一次）
        if RVC_INFERENCE is None:
            logger.info("初始化 RVC 推理器...")
            device = "cuda:0" if os.path.exists("/dev/nvidia0") else "cpu:0"
            RVC_INFERENCE = RVCInference(
                model_path=RVC_MODEL_PATH,
                index_path=RVC_INDEX_PATH if os.path.exists(RVC_INDEX_PATH) else "",
                device=device,
                version="v2"
            )
            logger.info(f"✅ RVC 推理器初始化完成 (device: {device})")

        # 使用 RVC 进行推理（只需要输入和输出路径）
        RVC_INFERENCE.infer_file(
            input_path=input_audio,
            output_path=output_audio
        )

        logger.info("✅ RVC 转换完成")
        return True

    except Exception as e:
        logger.error(f"❌ RVC 转换失败: {str(e)}")
        import traceback
        traceback.print_exc()
        return False


@app.on_event("startup")
async def startup_event():
    """启动时检查依赖"""
    logger.info("🚀 RVC TTS Server 启动中...")

    # 检查模型文件
    if not os.path.exists(RVC_MODEL_PATH):
        logger.error(f"❌ RVC 模型文件不存在: {RVC_MODEL_PATH}")
    else:
        logger.info(f"✅ RVC 模型文件: {RVC_MODEL_PATH}")

    if not os.path.exists(RVC_INDEX_PATH):
        logger.warning(f"⚠️ RVC 索引文件不存在: {RVC_INDEX_PATH}")
    else:
        logger.info(f"✅ RVC 索引文件: {RVC_INDEX_PATH}")

    # 检查 RVC API
    if RVC_AVAILABLE:
        logger.info("✅ RVC Python API 可用")
    else:
        logger.warning("⚠️ RVC Python API 不可用")


@app.get("/")
async def root():
    """健康检查"""
    return {
        "status": "running",
        "service": "RVC TTS Server",
        "model": RVC_MODEL_PATH,
        "rvc_available": RVC_AVAILABLE
    }


@app.post("/tts")
async def text_to_speech(request: TTSRequest):
    """
    将文本转换为语音
    1. 使用 edge-tts 生成基础 TTS
    2. 使用 RVC 转换为目标声音
    """
    if not request.text.strip():
        raise HTTPException(status_code=400, detail="文本不能为空")

    logger.info(f"🎤 收到 TTS 请求: {request.text}")

    # 创建临时文件
    temp_dir = Path(tempfile.mkdtemp())
    base_audio = temp_dir / "base.mp3"
    rvc_audio = temp_dir / "rvc.wav"

    try:
        # 步骤 1: 使用 edge-tts 生成基础语音
        logger.info("📝 步骤 1: 生成基础 TTS...")
        communicate = edge_tts.Communicate(
            text=request.text,
            voice=EDGE_VOICE,
            rate=request.rate,
            pitch=request.pitch
        )
        await communicate.save(str(base_audio))
        logger.info(f"✅ 基础 TTS 生成完成: {base_audio}")

        # 步骤 2: 使用 RVC 转换语音
        if RVC_AVAILABLE:
            logger.info("🎵 步骤 2: 使用 RVC 转换语音...")

            success = rvc_convert(str(base_audio), str(rvc_audio))

            if not success or not rvc_audio.exists():
                logger.warning("⚠️ RVC 转换失败，返回原始 TTS")
                with open(base_audio, "rb") as f:
                    audio_data = f.read()
                import shutil
                shutil.rmtree(temp_dir)
                return Response(
                    content=audio_data,
                    media_type="audio/mpeg",
                    headers={"Access-Control-Allow-Origin": "*"}
                )

            logger.info(f"✅ RVC 转换完成: {rvc_audio}")

            # 返回 RVC 转换后的音频
            with open(rvc_audio, "rb") as f:
                audio_data = f.read()
            import shutil
            shutil.rmtree(temp_dir)
            return Response(
                content=audio_data,
                media_type="audio/wav",
                headers={"Access-Control-Allow-Origin": "*"}
            )
        else:
            # 如果没有 rvc-cli，直接返回基础 TTS
            logger.warning("⚠️ rvc-cli 不可用，返回原始 TTS")
            with open(base_audio, "rb") as f:
                audio_data = f.read()
            # 清理临时文件
            import shutil
            shutil.rmtree(temp_dir)
            return Response(
                content=audio_data,
                media_type="audio/mpeg",
                headers={"Access-Control-Allow-Origin": "*"}
            )

    except asyncio.TimeoutError:
        logger.error("❌ TTS 生成超时")
        # 清理临时文件
        import shutil
        if temp_dir.exists():
            shutil.rmtree(temp_dir)
        raise HTTPException(status_code=500, detail="TTS 生成超时")
    except Exception as e:
        logger.error(f"❌ 错误: {str(e)}")
        # 清理临时文件
        import shutil
        if temp_dir.exists():
            shutil.rmtree(temp_dir)
        raise HTTPException(status_code=500, detail=f"生成失败: {str(e)}")


@app.get("/health")
async def health():
    """健康检查端点"""
    return {"status": "healthy"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8765, log_level="info")
