import os
from PIL import Image, ImageFilter, ImageOps

def convert_to_pixel_art(input_path="test.png", output_path="pixel.png", 
                         downscale_factor=4, color_count=32, blur_radius=0.8):
    """
    将图片转换为像素画，集成抗锯齿与颜色量化。
    
    参数:
        input_path: 输入图片路径
        output_path: 输出图片路径
        downscale_factor: 降采样倍数（数值越大像素块越大）
        color_count: 最终调色板颜色数量（建议 16-64）
        blur_radius: 高斯模糊半径（用于抑制降采样噪点）
    """
    if not os.path.exists(input_path):
        print(f"错误: 未找到文件 '{input_path}'，请确保图片位于脚本同级目录。")
        return

    # 1. 读取图片并转换为 RGBA 模式（保留透明通道）
    original = Image.open(input_path).convert("RGBA")
    width, height = original.size
    print(f"原始尺寸: {width}x{height}")

    # 2. 计算目标低分辨率尺寸
    new_width = max(1, width // downscale_factor)
    new_height = max(1, height // downscale_factor)
    print(f"像素化尺寸: {new_width}x{new_height}")

    # 3. 抗锯齿降采样：先用 Lanczos 缩放，再高斯模糊平滑噪点
    #    Lanczos 提供高质量的重采样，高斯模糊进一步消除局部高频杂色
    small = original.resize((new_width, new_height), Image.LANCZOS)
    small = small.filter(ImageFilter.GaussianBlur(radius=blur_radius))

    # 4. 颜色量化：使用中位切分算法将颜色精简到指定数量
    #    先转为 RGB 模式（中位切分不支持 RGBA），量化后再转回 RGBA
    rgb_small = small.convert("RGB")
    quantized = rgb_small.quantize(colors=color_count, method=Image.MEDIANCUT)
    quantized = quantized.convert("RGBA")

    # 5. 最近邻放大：将低分辨率图像放大回原尺寸，形成清晰像素块
    result = quantized.resize((width, height), Image.NEAREST)

    # 6. 保存并展示
    result.save(output_path)
    print(f"已保存至: {output_path}")
    result.show()

if __name__ == "__main__":
    # 固定参数，可直接调整
    convert_to_pixel_art(
        input_path="test.png",
        output_path="pixel.png",
        downscale_factor=10,      # 降采样倍数：越大像素越粗
        color_count=32,          # 调色板颜色数：越少风格越复古
        blur_radius=0.5          # 模糊半径：越大抗锯齿越强，但边缘越软
    )