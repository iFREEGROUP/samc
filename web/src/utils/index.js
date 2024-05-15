export const adjustImageToCanvas = (imgWidth, imgHeight, canvasWidth, canvasHeight) => {
    let scaledWidth = imgWidth;
    let scaledHeight = imgHeight;
    let scale = 1;

    // 确保图片能适应画布
    if (imgWidth > canvasWidth || imgHeight > canvasHeight) {
        // 先检查宽度，如果图片宽度大于画布宽度，则基于宽度调整
        if (imgWidth > canvasWidth) {
            scale = canvasWidth / imgWidth;
            scaledWidth = canvasWidth;
            scaledHeight = imgHeight * scale;
        }

        // 再检查调整后的高度是否也适合画布，避免只按宽度调整后高度仍超出
        if (scaledHeight > canvasHeight) {
            scale = canvasHeight / imgHeight;
            scaledHeight = canvasHeight;
            scaledWidth = imgWidth * scale;
        }
    }

    // 返回结果
    return {
        width: scaledWidth,
        height: scaledHeight,
        scale
    };
}

export const debounce = (fn, delay = 500) => {
    let timer = null
    return function (...args) {
        clearTimeout(timer)
        timer = setTimeout(() => {
            fn.call(this, ...args)
        }, delay);
    }
}
