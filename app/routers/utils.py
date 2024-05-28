import base64
import io
import cv2

def base64_to_image(base64_string: str,output_path):
    image_binary = base64_to_data(base64_string)
    # 根据你的图片类型（JPEG, PNG等）使用相应的库打开并保存图片
    try:
        import PIL.Image
        image = PIL.Image.open(image_binary)
        image.save(output_path)
        print(f"图片已成功保存至：{output_path}")

    except ImportError:
        print("请安装Pillow库来处理图片：pip install Pillow")


def base64_to_data(base64_string):
    """
    将 Base64 编码的图片数据转换为图片文件并保存。

    :param base64_string: Base64 编码的图片数据字符串
    :param output_path: 图片保存的路径，包括文件名和扩展名
    """
    # 去除 Base64 数据前缀 "data:image/jpeg;base64," 或 "data:image/png;base64," 等，如果你有的话
    base64_data = base64_string.split(',', 1)[-1]

    # 解码 Base64 数据
    image_data = base64.b64decode(base64_data)

    # 使用BytesIO来处理二进制数据
    # image_data = io.BytesIO(image_data)

    return image_data
    
# 示例 Base64 字符串（请替换为你的实际