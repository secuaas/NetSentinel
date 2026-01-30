"""Flow schemas."""

from datetime import datetime
from typing import List, Optional
from uuid import UUID

from pydantic import BaseModel


class FlowResponse(BaseModel):
    """Traffic flow response schema."""
    id: UUID
    src_mac: str
    src_ip: Optional[str] = None
    src_port: Optional[int] = None
    dst_mac: str
    dst_ip: Optional[str] = None
    dst_port: Optional[int] = None
    vlan_id: Optional[int] = None
    ip_protocol: Optional[int] = None
    protocol_name: Optional[str] = None
    first_seen: datetime
    last_seen: datetime
    packet_count: int
    byte_count: int

    class Config:
        from_attributes = True


class FlowListResponse(BaseModel):
    """Paginated flow list response."""
    items: List[FlowResponse]
    total: int
    page: int
    page_size: int
    pages: int
