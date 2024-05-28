
from mobile_sam import sam_model_registry
from mobile_sam.modeling.sam import Sam
import functools
from .config import settings
import torch


@functools.lru_cache()
def init_model(model_path: str = None)->Sam :
    if model_path is None:
        model_path = settings.model_path
    print("model_path",model_path)
    model_type = "vit_t"
    device = "cuda" if torch.cuda.is_available() else "cpu"
    mobile_sam = sam_model_registry[model_type](checkpoint=model_path)
    mobile_sam.to(device=device)
    mobile_sam.eval()
    return mobile_sam

