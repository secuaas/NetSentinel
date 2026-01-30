"""Traffic flow model."""

from datetime import datetime
from typing import Optional
from uuid import UUID, uuid4

from sqlalchemy import BigInteger, DateTime, ForeignKey, Integer, SmallInteger
from sqlalchemy.dialects.postgresql import INET, MACADDR, UUID as PG_UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from ..database import Base


class TrafficFlow(Base):
    """Traffic flow between two devices."""

    __tablename__ = "traffic_flows"

    id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), primary_key=True, default=uuid4)
    src_device_id: Mapped[Optional[UUID]] = mapped_column(PG_UUID(as_uuid=True), ForeignKey("devices.id", ondelete="SET NULL"))
    src_mac: Mapped[str] = mapped_column(MACADDR, nullable=False)
    src_ip: Mapped[Optional[str]] = mapped_column(INET)
    src_port: Mapped[Optional[int]] = mapped_column(Integer)
    dst_device_id: Mapped[Optional[UUID]] = mapped_column(PG_UUID(as_uuid=True), ForeignKey("devices.id", ondelete="SET NULL"))
    dst_mac: Mapped[str] = mapped_column(MACADDR, nullable=False)
    dst_ip: Mapped[Optional[str]] = mapped_column(INET)
    dst_port: Mapped[Optional[int]] = mapped_column(Integer)
    vlan_id: Mapped[Optional[int]] = mapped_column(SmallInteger)
    outer_vlan_id: Mapped[Optional[int]] = mapped_column(SmallInteger)
    ethertype: Mapped[Optional[int]] = mapped_column(SmallInteger)
    ip_protocol: Mapped[Optional[int]] = mapped_column(SmallInteger)
    first_seen: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    last_seen: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    packet_count: Mapped[int] = mapped_column(BigInteger, default=0)
    byte_count: Mapped[int] = mapped_column(BigInteger, default=0)
    tcp_flags_seen: Mapped[Optional[int]] = mapped_column(SmallInteger, default=0)
