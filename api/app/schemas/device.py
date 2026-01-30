"""Device schemas."""

from datetime import datetime
from typing import List, Optional
from uuid import UUID

from pydantic import BaseModel, Field


class DeviceIPResponse(BaseModel):
    """Device IP address response."""
    ip_address: str
    vlan_id: Optional[int] = None
    first_seen: datetime
    last_seen: datetime

    class Config:
        from_attributes = True


class DeviceResponse(BaseModel):
    """Device response schema."""
    id: UUID
    mac_address: str
    oui_vendor: Optional[str] = None
    device_type: str
    device_name: Optional[str] = None
    device_notes: Optional[str] = None
    first_seen: datetime
    last_seen: datetime
    total_packets_sent: int
    total_packets_received: int
    total_bytes_sent: int
    total_bytes_received: int
    is_gateway: bool
    is_active: bool
    is_flagged: bool
    ip_addresses: List[str] = Field(default_factory=list)
    vlans: List[int] = Field(default_factory=list)

    class Config:
        from_attributes = True


class DeviceListResponse(BaseModel):
    """Paginated device list response."""
    items: List[DeviceResponse]
    total: int
    page: int
    page_size: int
    pages: int


class DeviceUpdate(BaseModel):
    """Device update schema."""
    device_type: Optional[str] = None
    device_name: Optional[str] = None
    device_notes: Optional[str] = None
    is_gateway: Optional[bool] = None
    is_flagged: Optional[bool] = None
