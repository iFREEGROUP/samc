from functools import lru_cache
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    base_dir: str = "."
    model_path: str = "mobile_sam.pt"
    class Config:
        env_file = ".env"


settings = Settings()

@lru_cache
def get_settings():
    return Settings()