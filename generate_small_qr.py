#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import qrcode
from PIL import Image

def generate_small_qr():
    # 创建二维码实例
    qr = qrcode.QRCode(
        version=1,  # 控制二维码的大小
        error_correction=qrcode.constants.ERROR_CORRECT_L,  # 低错误纠正
        box_size=2,  # 每个小方块的像素数（很小）
        border=1,   # 边框大小
    )
    
    # 添加数据
    text = "Small QR Test - WeChat Model Performance"
    qr.add_data(text)
    qr.make(fit=True)
    
    # 创建图像
    img = qr.make_image(fill_color="black", back_color="white")
    
    # 保存为小尺寸图像
    img.save("small_qr.png")
    print(f"小尺寸二维码已生成并保存为 small_qr.png")
    print(f"图像尺寸: {img.size}")
    print(f"包含的文本内容: {text}")
    
    # 创建一个更小的版本用于测试超分辨率
    tiny_img = img.resize((50, 50), Image.NEAREST)  # 非常小的尺寸
    tiny_img.save("tiny_qr.png")
    print(f"\n超小尺寸二维码已生成并保存为 tiny_qr.png")
    print(f"图像尺寸: {tiny_img.size}")

if __name__ == "__main__":
    generate_small_qr()