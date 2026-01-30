// Device types
export interface Device {
  id: string
  mac_address: string
  ip_addresses: string[]
  hostname: string | null
  vendor: string | null
  device_type: DeviceType
  os_fingerprint: string | null
  first_seen: string
  last_seen: string
  is_active: boolean
  open_ports: number[]
  protocols: string[]
  tags: string[]
  notes: string | null
  risk_score: number
  network_zone: string | null
}

export type DeviceType =
  | 'unknown'
  | 'workstation'
  | 'server'
  | 'router'
  | 'switch'
  | 'firewall'
  | 'printer'
  | 'camera'
  | 'iot'
  | 'plc'
  | 'hmi'
  | 'scada'
  | 'mobile'
  | 'virtual'

// Flow types
export interface Flow {
  id: string
  src_mac: string
  src_ip: string
  src_port: number
  dst_mac: string
  dst_ip: string
  dst_port: number
  protocol: string
  packets: number
  bytes: number
  first_seen: string
  last_seen: string
  application: string | null
  flags: string[]
}

// Protocol statistics
export interface ProtocolStats {
  protocol: string
  packets: number
  bytes: number
  flows: number
  percentage: number
}

// Dashboard statistics
export interface DashboardStats {
  total_devices: number
  active_devices: number
  new_devices_24h: number
  total_flows: number
  total_bytes: number
  protocols: ProtocolStats[]
  device_types: { type: DeviceType; count: number }[]
  alerts: Alert[]
  top_talkers: TopTalker[]
}

export interface TopTalker {
  device_id: string
  mac_address: string
  ip_address: string
  hostname: string | null
  bytes_sent: number
  bytes_received: number
  total_bytes: number
}

// Alerts
export interface Alert {
  id: string
  type: AlertType
  severity: AlertSeverity
  title: string
  description: string
  device_id: string | null
  flow_id: string | null
  created_at: string
  acknowledged: boolean
  acknowledged_by: string | null
  acknowledged_at: string | null
}

export type AlertType =
  | 'new_device'
  | 'port_scan'
  | 'unusual_traffic'
  | 'protocol_anomaly'
  | 'high_bandwidth'
  | 'connection_failure'

export type AlertSeverity = 'low' | 'medium' | 'high' | 'critical'

// Network topology
export interface TopologyNode {
  id: string
  label: string
  group: DeviceType
  title: string
  shape: string
  color?: string
}

export interface TopologyEdge {
  id: string
  from: string
  to: string
  value: number
  title: string
}

// API responses
export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

export interface ApiError {
  detail: string
  code?: string
}

// Auth
export interface User {
  id: string
  username: string
  email: string
  role: 'admin' | 'operator' | 'viewer'
}

export interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
}

// Filters
export interface DeviceFilters {
  search?: string
  device_type?: DeviceType
  is_active?: boolean
  network_zone?: string
  min_risk_score?: number
}

export interface FlowFilters {
  src_ip?: string
  dst_ip?: string
  protocol?: string
  min_bytes?: number
  start_time?: string
  end_time?: string
}

// Time ranges for charts
export type TimeRange = '1h' | '6h' | '24h' | '7d' | '30d'

// WebSocket messages
export interface WSMessage {
  type: 'device_update' | 'flow_update' | 'alert' | 'stats_update'
  payload: unknown
}
