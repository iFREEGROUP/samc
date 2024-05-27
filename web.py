
from typing import Union
from mobile_sam import sam_model_registry, SamAutomaticMaskGenerator, SamPredictor
from fastapi import FastAPI
import torch

app = FastAPI()


@app.post("/sam")
def read_root():
    model_type = "vit_t"
    sam_checkpoint = "./mobile_sam-tiny-vitt.safetensors"

    device = "cuda" if torch.cuda.is_available() else "cpu"

    mobile_sam = sam_model_registry[model_type](checkpoint=sam_checkpoint)
    mobile_sam.to(device=device)
    mobile_sam.eval()
    predictor = SamPredictor(mobile_sam)
    predictor.set_image()
    predictor.predict
    return {"Hello": "World"}

# export https_proxy=http://172.20.0.100:55508 http_proxy=http://172.20.0.100:55508 all_proxy=socks5://172.20.0.100:55508

if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)