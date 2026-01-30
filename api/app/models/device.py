"""Device models."""

from datetime import datetime
from typing import List, Optional
from uuid import UUID, uuid4

from sqlalchemy import Boolean, BigInteger, DateTime, ForeignKey, SmallInteger, String, Text
from sqlalchemy.dialects.postgresql import INET, MACADDR, UUID as PG_UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from ..database import Base


class Device(Base):
    """Network device entity."""

    __tablename__ = "devices"

    id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), primary_key=True, default=uuid4)
    mac_address: Mapped[str] = mapped_column(MACADDR, unique=True, nullable=False)
    oui_vendor: Mapped[Optional[str]] = mapped_column(String(128))
    oui_prefix: Mapped[Optional[str]] = mapped_column(String(8))
    device_type: Mapped[str] = mapped_column(String(50), default="unknown")
    device_name: Mapped[Optional[str]] = mapped_column(String(255))
    device_notes: Mapped[Optional[str]] = mapped_column(Text)
    first_seen: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    last_seen: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    total_packets_sent: Mapped[int] = mapped_column(BigInteger, default=0)
    total_packets_received: Mapped[int] = mapped_column(BigInteger, default=0)
    total_bytes_sent: Mapped[int] = mapped_column(BigInteger, default=0)
    total_bytes_received: Mapped[int] = mapped_column(BigInteger, default=0)
    is_gateway: Mapped[bool] = mapped_column(Boolean, default=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    is_flagged: Mapped[bool] = mapped_column(Boolean, default=False)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow, onupdate=datetime.utcnow)

    # Relationships
    ips: Mapped[List["DeviceIP"]] = relationship("DeviceIP", back_populates="device", cascade="all, delete-orphan")


class DeviceIP(Base):
    """IP address associated with a device."""

    __tablename__ = "device_ips"

    id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), primary_key=True, default=uuid4)
    device_id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), ForeignKey("devices.id", ondelete="CASCADE"), nullable=False)
    ip_address: Mapped[str] = mapped_column(INET, nullable=False)
    ip_version: Mapped[int] = mapped_column(SmallInteger, default=4)
    vlan_id: Mapped[Optional[int]] = mapped_column(SmallInteger)
    subnet_mask: Mapped[Optional[str]] = mapped_column(INET)
    first_seen: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    last_seen: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    packets_sent: Mapped[int] = mapped_column(BigInteger, default=0)
    packets_received: Mapped[int] = mapped_column(BigInteger, default=0)
    bytes_sent: Mapped[int] = mapped_column(BigInteger, default=0)
    bytes_received: Mapped[int] = mapped_column(BigInteger, default=0)

    # Relationships
    device: Mapped["Device"] = relationship("Device", back_populates="ips")
