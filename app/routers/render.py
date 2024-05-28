from typing import Any

from pydantic import BaseModel


class BaseResponse(BaseModel):
    code: int = 200
    message: str = "Success"
    data: Any

    @staticmethod
    def ok(data: Any):
        return BaseResponse(data=data)

    @staticmethod
    def error(message: str):
        return BaseResponse(message=message, data={})