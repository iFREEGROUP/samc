from functools import lru_cache
import logging
import os
import sys
from logging.handlers import RotatingFileHandler
from config import settings
from fastapi.staticfiles import StaticFiles


sys.path.append(os.path.dirname(os.path.abspath(__file__)) + '/../')
from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.routers.file import file_router
from app.routers.sam import sam_router
from pathlib import Path



# logs_dir = Path("logs")
# if not logs_dir.exists():
#     logs_dir.mkdir()

# file_handler = RotatingFileHandler(filename="logs/app.log", maxBytes=1000000, backupCount=10)
# logger = logging.getLogger()
# logger.addHandler(file_handler)
# logger.setLevel(logging.INFO)



app = FastAPI()
app.include_router(file_router)
app.include_router(sam_router)
app.mount("/data", StaticFiles(directory=settings.base_dir), name="data")
app.mount("/", StaticFiles(directory="./build", html=True), name="static")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

load_dotenv(verbose=True, override=True)

if __name__ == '__main__':
    import uvicorn
    uvicorn.run(app="main:app", host="0.0.0.0", port=5050, reload=True,workers=1)