import base64
import io
import cv2
import numpy as np

def base64_to_image(base64_string: str,output_path:str):
    image_binary = base64_to_data(base64_string)
    # 根据你的图片类型（JPEG, PNG等）使用相应的库打开并保存图片
    data = np.asarray(bytearray(image_binary), dtype="uint8")
    image = cv2.imdecode(data,cv2.IMREAD_COLOR)
    cv2.imwrite(output_path,image)

def base64_to_data(base64_string:str):

    """
    将 Base64 编码的图片数据转换为图片文件并保存。

    :param base64_string: Base64 编码的图片数据字符串
    :param output_path: 图片保存的路径，包括文件名和扩展名
    """
    # 去除 Base64 数据前缀 "data:image/jpeg;base64," 或 "data:image/png;base64," 等，如果你有的话
    base64_data = base64_string.split(',', 1)[-1]

    # 解码 Base64 数据
    image_data = base64.b64decode(base64_data)

    return image_data
    
# 示例 Base64 字符串（请替换为你的实际