"""Flow API endpoints."""

from math import ceil
from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from ..database import get_db
from ..models.flow import TrafficFlow
from ..schemas.flow import FlowListResponse, FlowResponse

router = APIRouter(prefix="/flows", tags=["flows"])

# Protocol name mapping
PROTOCOL_NAMES = {
    1: "ICMP",
    6: "TCP",
    17: "UDP",
    47: "GRE",
    50: "ESP",
    89: "OSPF",
}


def flow_to_response(flow: TrafficFlow) -> FlowResponse:
    """Convert Flow model to response schema."""
    protocol_name = PROTOCOL_NAMES.get(flow.ip_protocol) if flow.ip_protocol else None

    return FlowResponse(
        id=flow.id,
        src_mac=str(flow.src_mac),
        src_ip=str(flow.src_ip) if flow.src_ip else None,
        src_port=flow.src_port,
        dst_mac=str(flow.dst_mac),
        dst_ip=str(flow.dst_ip) if flow.dst_ip else None,
        dst_port=flow.dst_port,
        vlan_id=flow.vlan_id,
        ip_protocol=flow.ip_protocol,
        protocol_name=protocol_name,
        first_seen=flow.first_seen,
        last_seen=flow.last_seen,
        packet_count=flow.packet_count,
        byte_count=flow.byte_count,
    )


@router.get("", response_model=FlowListResponse)
async def list_flows(
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=100),
    src_mac: Optional[str] = None,
    dst_mac: Optional[str] = None,
    src_ip: Optional[str] = None,
    dst_ip: Optional[str] = None,
    vlan_id: Optional[int] = None,
    protocol: Optional[int] = None,
    port: Optional[int] = None,
    sort_by: str = Query("last_seen", regex="^(first_seen|last_seen|packet_count|byte_count)$"),
    sort_order: str = Query("desc", regex="^(asc|desc)$"),
    db: AsyncSession = Depends(get_db),
):
    """List flows with pagination and filters."""
    query = select(TrafficFlow)

    # Apply filters
    if src_mac:
        query = query.where(TrafficFlow.src_mac == src_mac)
    if dst_mac:
        query = query.where(TrafficFlow.dst_mac == dst_mac)
    if src_ip:
        query = query.where(TrafficFlow.src_ip == src_ip)
    if dst_ip:
        query = query.where(TrafficFlow.dst_ip == dst_ip)
    if vlan_id is not None:
        query = query.where(TrafficFlow.vlan_id == vlan_id)
    if protocol is not None:
        query = query.where(TrafficFlow.ip_protocol == protocol)
    if port is not None:
        query = query.where(
            (TrafficFlow.src_port == port) | (TrafficFlow.dst_port == port)
        )

    # Count total
    count_query = select(func.count()).select_from(query.subquery())
    total = await db.scalar(count_query) or 0

    # Apply sorting
    sort_column = getattr(TrafficFlow, sort_by)
    if sort_order == "desc":
        sort_column = sort_column.desc()
    query = query.order_by(sort_column)

    # Apply pagination
    offset = (page - 1) * page_size
    query = query.offset(offset).limit(page_size)

    result = await db.execute(query)
    flows = result.scalars().all()

    return FlowListResponse(
        items=[flow_to_response(f) for f in flows],
        total=total,
        page=page,
        page_size=page_size,
        pages=ceil(total / page_size) if total > 0 else 1,
    )


@router.get("/{flow_id}", response_model=FlowResponse)
async def get_flow(
    flow_id: UUID,
    db: AsyncSession = Depends(get_db),
):
    """Get a flow by ID."""
    query = select(TrafficFlow).where(TrafficFlow.id == flow_id)
    result = await db.execute(query)
    flow = result.scalar_one_or_none()

    if not flow:
        raise HTTPException(status_code=404, detail="Flow not found")

    return flow_to_response(flow)
