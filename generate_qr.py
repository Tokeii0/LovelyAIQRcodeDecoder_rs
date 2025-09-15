#!/usr/bin/env python3
import qrcode
from PIL import Image

# 创建二维码实例
qr = qrcode.QRCode(
    version=1,
    error_correction=qrcode.constants.ERROR_CORRECT_L,
    box_size=10,
    border=4,
)

# 添加数据
data = "Hello, QR Code Decoder!"
qr.add_data(data)
qr.make(fit=True)

# 创建图像
img = qr.make_image(fill_color="black", back_color="white")

# 保存图像
img.save("test_qr.png")
print(f"二维码已生成并保存为 test_qr.png")
print(f"包含的文本内容: {data}")