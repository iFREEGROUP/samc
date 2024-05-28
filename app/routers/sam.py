import base64
import json
from typing import Annotated, List, Optional, Tuple
from fastapi import APIRouter, Body, Depends
import numpy as np
from pydantic import BaseModel
from mobile_sam import SamPredictor
import cv2
from app.config import Settings, get_settings
from app.model import init_model
from app.routers.render import BaseResponse
from app.routers.utils import base64_to_data

sam_router = APIRouter(
    prefix="/api",
    tags=["sam"],
    responses={404: {"description": "Not found"}},
)

class SegmentParam(BaseModel):
    image: str
    inputs: List[Tuple[float,float,int]]
    masks: Optional[str] = None #暂时不使用

@sam_router.post("/sam_base64")
async def sam_anything_base64(settings: Annotated[Settings, Depends(get_settings)],item: SegmentParam,mobile_sam = Depends(init_model)):
    input_point = []
    input_label=[]
    for input in item.inputs:
        input_point.append(input[:2])
        input_label.append(input[2])

    predictor = SamPredictor(mobile_sam)

    data = base64_to_data(item.image)
    data = np.asarray(bytearray(data), dtype="uint8")

    image = cv2.imdecode(data,cv2.IMREAD_COLOR)
    image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
    predictor.set_image(image)

    masks_input = None
    # if item.masks is not None:
        # received_json = json.loads(item.masks)
        # masks_input = np.expand_dims(np.array(received_json), 0)



    masks, scores, logits = predictor.predict(
        point_coords=np.array(input_point),
        point_labels=np.array(input_label),
        multimask_output=False,
        mask_input=masks_input
    )

    best_mask_low_mask = logits[np.argmax(scores), :, :].astype(np.uint8)
    best_mask = masks[np.argmax(scores), :, :].astype(np.uint8)

    best_mask *= 255
    frame = cv2.cvtColor(best_mask, cv2.COLOR_RGB2BGR)

    gray_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY) 

    num_labels, labels, stats, centroids = cv2.connectedComponentsWithStats(gray_frame)
    min_area = 100  # 根据实际情况调整
    mask_optimized = np.zeros_like(gray_frame)
    for i in range(1, num_labels):
        if stats[i, cv2.CC_STAT_AREA] > min_area:
            mask_optimized[labels == i] = 255
    contours, _ = cv2.findContours(mask_optimized.astype(np.uint8), cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    mask_contours = np.zeros_like(frame)
    frame = cv2.drawContours(mask_contours, contours, -1, (0,255,0), thickness=cv2.FILLED)
    
    # cv2.imwrite("test.png",frame)

    image = cv2.imencode('.jpg',frame)[1]
    image_code = str(base64.b64encode(image))[2:-1]

    return BaseResponse.ok(dict(
        mask = image_code,
        low_mask = json.dumps(best_mask_low_mask.tolist()),
    ))
# export https_proxy=http://172.20.0.100:55508 http_proxy=http://172.20.0.100:55508 all_proxy=socks5://172.20.0.100:55508