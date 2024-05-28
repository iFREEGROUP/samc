from functools import lru_cache
import logging
import time
from dataclasses import dataclass
from typing import Annotated, List
from pathlib import Path
from fastapi import APIRouter, Body, Depends, Form
from fastapi.params import File
from pydantic import BaseModel, Field

from app.config import Settings, get_settings
from app.routers.render import BaseResponse
from app.routers.utils import base64_to_image

file_router = APIRouter(
    prefix="/api",
    tags=["file"],
    responses={404: {"description": "Not found"}},
)
  


@file_router.get("/files")
async def get_image_files(settings: Annotated[Settings, Depends(get_settings)]):
    base_dir = Path(settings.base_dir)
    if not base_dir.exists() or not base_dir.is_dir():
        return BaseResponse.error("base_dir:{} is not a directory".format(base_dir))
    files = []
    for file in base_dir.iterdir():
        if not file.is_file():
            continue
        if file.suffix.lower() not in [".jpg", ".jpeg", ".png"]:
            continue
        if file.name.startswith('.') or file.name.endswith("_mask.png"):
            continue
        
        mask_file = file.stem + "_mask.png"
        full_mask_file = base_dir / mask_file
        if not full_mask_file.exists():
            mask_file = None

        files.append({
            "image_path": str(file.name),
            "mask_path": mask_file,
            "rotate": find_rotate_from_file(base_dir,file.name),
        })
    
    return BaseResponse.ok(files)

def find_rotate_from_file(base_dir:Path,file_name:str):
    file:Path = base_dir / "rotate.txt"
    if not file.exists():
        return 0
    rotate = 0
    with open(str(file), "r") as f:
        lines = f.readlines()
        for line in lines:
            if len(line) == 0:continue
            sp = line.split(',')
            if sp[0] == file_name:
                rotate = float(sp[1])
                break
    return rotate


class MaskParam(BaseModel):
    ori_image_name: str
    mask: str
    rotate: float = 0

@file_router.post("/masks")
async def save_mask(settings: Annotated[Settings, Depends(get_settings)],item: MaskParam):
    base_dir = Path(settings.base_dir)
    ori_image_path:Path = base_dir / item.ori_image_name
    file_name = ori_image_path.stem + "_mask.png"
    mask_image_path = base_dir / file_name
    base64_to_image(item,mask_image_path)
    #保存旋转角度
    rotate_file = base_dir / "rotate.txt"
    if not rotate_file.exists():
        rotate_file.touch()
    content = f"{item.ori_image_name},{item.rotate}\n"
    with open(rotate_file, "a") as f:
        f.write(content)
        
    return BaseResponse.ok(
        dict(
            full_path=str(mask_image_path),
            file_name=file_name,
        )
    )