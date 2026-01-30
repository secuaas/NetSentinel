"""Statistics schemas."""

from typing import List, Optional

from pydantic import BaseModel


class ProtocolStats(BaseModel):
    """Protocol statistics."""
    protocol_name: str
    packet_count: int
    byte_count: int
    percentage: float


class TopTalker(BaseModel):
    """Top talker device."""
    mac_address: str
    device_name: Optional[str] = None
    device_type: str
    bytes_total: int
    packets_total: int


class VlanStats(BaseModel):
    """VLAN statistics."""
    vlan_id: int
    device_count: int
    packet_count: int
    byte_count: int


class DashboardStats(BaseModel):
    """Dashboard statistics."""
    total_devices: int
    active_devices: int
    total_flows: int
    total_packets: int
    total_bytes: int
    protocols: List[ProtocolStats]
    top_talkers: List[TopTalker]
    vlans: List[VlanStats]
    uptime_seconds: int
