"""Statistics API endpoints."""

from fastapi import APIRouter, Depends
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from ..database import get_db
from ..models.device import Device, DeviceIP
from ..models.flow import TrafficFlow
from ..schemas.stats import DashboardStats, ProtocolStats, TopTalker, VlanStats

router = APIRouter(prefix="/stats", tags=["statistics"])

# Protocol name mapping
PROTOCOL_NAMES = {
    1: "ICMP",
    6: "TCP",
    17: "UDP",
    47: "GRE",
    50: "ESP",
    89: "OSPF",
}

ETHERTYPE_NAMES = {
    0x0800: "IPv4",
    0x0806: "ARP",
    0x86DD: "IPv6",
}


@router.get("/dashboard", response_model=DashboardStats)
async def get_dashboard_stats(
    db: AsyncSession = Depends(get_db),
):
    """Get dashboard statistics."""
    # Device counts
    total_devices = await db.scalar(select(func.count(Device.id))) or 0
    active_devices = await db.scalar(
        select(func.count(Device.id)).where(Device.is_active == True)
    ) or 0

    # Flow counts
    total_flows = await db.scalar(select(func.count(TrafficFlow.id))) or 0

    # Total packets and bytes
    flow_stats = await db.execute(
        select(
            func.sum(TrafficFlow.packet_count),
            func.sum(TrafficFlow.byte_count),
        )
    )
    row = flow_stats.one()
    total_packets = row[0] or 0
    total_bytes = row[1] or 0

    # Protocol distribution
    protocol_query = await db.execute(
        select(
            TrafficFlow.ip_protocol,
            func.sum(TrafficFlow.packet_count).label("packets"),
            func.sum(TrafficFlow.byte_count).label("bytes"),
        )
        .group_by(TrafficFlow.ip_protocol)
        .order_by(func.sum(TrafficFlow.packet_count).desc())
        .limit(10)
    )
    protocols = []
    for row in protocol_query:
        proto_num = row[0]
        proto_name = PROTOCOL_NAMES.get(proto_num, f"Proto {proto_num}") if proto_num else "Unknown"
        protocols.append(
            ProtocolStats(
                protocol_name=proto_name,
                packet_count=row[1] or 0,
                byte_count=row[2] or 0,
                percentage=round((row[1] or 0) / total_packets * 100, 2) if total_packets > 0 else 0,
            )
        )

    # Top talkers (devices by total bytes)
    top_talkers_query = await db.execute(
        select(
            Device.mac_address,
            Device.device_name,
            Device.device_type,
            (Device.total_bytes_sent + Device.total_bytes_received).label("bytes_total"),
            (Device.total_packets_sent + Device.total_packets_received).label("packets_total"),
        )
        .order_by((Device.total_bytes_sent + Device.total_bytes_received).desc())
        .limit(10)
    )
    top_talkers = [
        TopTalker(
            mac_address=str(row[0]),
            device_name=row[1],
            device_type=row[2],
            bytes_total=row[3] or 0,
            packets_total=row[4] or 0,
        )
        for row in top_talkers_query
    ]

    # VLAN statistics
    vlan_query = await db.execute(
        select(
            DeviceIP.vlan_id,
            func.count(func.distinct(DeviceIP.device_id)).label("device_count"),
            func.sum(DeviceIP.packets_sent + DeviceIP.packets_received).label("packets"),
            func.sum(DeviceIP.bytes_sent + DeviceIP.bytes_received).label("bytes"),
        )
        .where(DeviceIP.vlan_id.isnot(None))
        .group_by(DeviceIP.vlan_id)
        .order_by(func.count(func.distinct(DeviceIP.device_id)).desc())
    )
    vlans = [
        VlanStats(
            vlan_id=row[0],
            device_count=row[1] or 0,
            packet_count=row[2] or 0,
            byte_count=row[3] or 0,
        )
        for row in vlan_query
    ]

    return DashboardStats(
        total_devices=total_devices,
        active_devices=active_devices,
        total_flows=total_flows,
        total_packets=total_packets,
        total_bytes=total_bytes,
        protocols=protocols,
        top_talkers=top_talkers,
        vlans=vlans,
        uptime_seconds=0,  # Would need to track separately
    )
