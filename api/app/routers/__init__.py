"""API routers."""

from .devices import router as devices_router
from .flows import router as flows_router
from .stats import router as stats_router
from .auth import router as auth_router

__all__ = ["devices_router", "flows_router", "stats_router", "auth_router"]
