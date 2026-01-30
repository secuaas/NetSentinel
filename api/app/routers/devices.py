"""Device API endpoints."""

from math import ceil
from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from ..database import get_db
from ..models.device import Device, DeviceIP
from ..schemas.device import DeviceListResponse, DeviceResponse, DeviceUpdate

router = APIRouter(prefix="/devices", tags=["devices"])


def device_to_response(device: Device) -> DeviceResponse:
    """Convert Device model to response schema."""
    ip_addresses = [str(ip.ip_address) for ip in device.ips]
    vlans = list(set(ip.vlan_id for ip in device.ips if ip.vlan_id is not None))

    return DeviceResponse(
        id=device.id,
        mac_address=str(device.mac_address),
        oui_vendor=device.oui_vendor,
        device_type=device.device_type,
        device_name=device.device_name,
        device_notes=device.device_notes,
        first_seen=device.first_seen,
        last_seen=device.last_seen,
        total_packets_sent=device.total_packets_sent,
        total_packets_received=device.total_packets_received,
        total_bytes_sent=device.total_bytes_sent,
        total_bytes_received=device.total_bytes_received,
        is_gateway=device.is_gateway,
        is_active=device.is_active,
        is_flagged=device.is_flagged,
        ip_addresses=ip_addresses,
        vlans=vlans,
    )


@router.get("", response_model=DeviceListResponse)
async def list_devices(
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=100),
    search: Optional[str] = None,
    device_type: Optional[str] = None,
    is_active: Optional[bool] = None,
    is_flagged: Optional[bool] = None,
    vlan_id: Optional[int] = None,
    sort_by: str = Query("last_seen", regex="^(mac_address|device_name|first_seen|last_seen|total_bytes_sent|total_bytes_received)$"),
    sort_order: str = Query("desc", regex="^(asc|desc)$"),
    db: AsyncSession = Depends(get_db),
):
    """List devices with pagination and filters."""
    query = select(Device).options(selectinload(Device.ips))

    # Apply filters
    if search:
        query = query.where(
            Device.mac_address.ilike(f"%{search}%")
            | Device.device_name.ilike(f"%{search}%")
            | Device.oui_vendor.ilike(f"%{search}%")
        )
    if device_type:
        query = query.where(Device.device_type == device_type)
    if is_active is not None:
        query = query.where(Device.is_active == is_active)
    if is_flagged is not None:
        query = query.where(Device.is_flagged == is_flagged)
    if vlan_id is not None:
        query = query.join(DeviceIP).where(DeviceIP.vlan_id == vlan_id)

    # Count total
    count_query = select(func.count()).select_from(query.subquery())
    total = await db.scalar(count_query) or 0

    # Apply sorting
    sort_column = getattr(Device, sort_by)
    if sort_order == "desc":
        sort_column = sort_column.desc()
    query = query.order_by(sort_column)

    # Apply pagination
    offset = (page - 1) * page_size
    query = query.offset(offset).limit(page_size)

    result = await db.execute(query)
    devices = result.scalars().unique().all()

    return DeviceListResponse(
        items=[device_to_response(d) for d in devices],
        total=total,
        page=page,
        page_size=page_size,
        pages=ceil(total / page_size) if total > 0 else 1,
    )


@router.get("/{device_id}", response_model=DeviceResponse)
async def get_device(
    device_id: UUID,
    db: AsyncSession = Depends(get_db),
):
    """Get a device by ID."""
    query = select(Device).options(selectinload(Device.ips)).where(Device.id == device_id)
    result = await db.execute(query)
    device = result.scalar_one_or_none()

    if not device:
        raise HTTPException(status_code=404, detail="Device not found")

    return device_to_response(device)


@router.patch("/{device_id}", response_model=DeviceResponse)
async def update_device(
    device_id: UUID,
    update: DeviceUpdate,
    db: AsyncSession = Depends(get_db),
):
    """Update device metadata."""
    query = select(Device).options(selectinload(Device.ips)).where(Device.id == device_id)
    result = await db.execute(query)
    device = result.scalar_one_or_none()

    if not device:
        raise HTTPException(status_code=404, detail="Device not found")

    # Update fields
    update_data = update.model_dump(exclude_unset=True)
    for field, value in update_data.items():
        setattr(device, field, value)

    await db.commit()
    await db.refresh(device)

    return device_to_response(device)


@router.get("/{device_id}/flows")
async def get_device_flows(
    device_id: UUID,
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=100),
    db: AsyncSession = Depends(get_db),
):
    """Get flows for a device."""
    from ..models.flow import TrafficFlow
    from ..schemas.flow import FlowListResponse, FlowResponse

    # Verify device exists
    device_query = select(Device).where(Device.id == device_id)
    device = await db.scalar(device_query)
    if not device:
        raise HTTPException(status_code=404, detail="Device not found")

    # Get flows
    query = select(TrafficFlow).where(
        (TrafficFlow.src_device_id == device_id)
        | (TrafficFlow.dst_device_id == device_id)
    ).order_by(TrafficFlow.last_seen.desc())

    # Count
    count_query = select(func.count()).select_from(query.subquery())
    total = await db.scalar(count_query) or 0

    # Paginate
    offset = (page - 1) * page_size
    query = query.offset(offset).limit(page_size)

    result = await db.execute(query)
    flows = result.scalars().all()

    return FlowListResponse(
        items=[
            FlowResponse(
                id=f.id,
                src_mac=str(f.src_mac),
                src_ip=str(f.src_ip) if f.src_ip else None,
                src_port=f.src_port,
                dst_mac=str(f.dst_mac),
                dst_ip=str(f.dst_ip) if f.dst_ip else None,
                dst_port=f.dst_port,
                vlan_id=f.vlan_id,
                ip_protocol=f.ip_protocol,
                first_seen=f.first_seen,
                last_seen=f.last_seen,
                packet_count=f.packet_count,
                byte_count=f.byte_count,
            )
            for f in flows
        ],
        total=total,
        page=page,
        page_size=page_size,
        pages=ceil(total / page_size) if total > 0 else 1,
    )
