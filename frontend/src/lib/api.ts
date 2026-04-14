const API_BASE = '/api/v1'
const ADMIN_BASE = '/api/admin'

// ── Auth store (simple Zustand) ──────────────────────────────
interface AuthState {
  token: string | null
  user: AuthUser | null
  login: (token: string, user: AuthUser) => void
  logout: () => void
}

export interface AuthUser {
  id: string
  email: string
  name: string
  roles: string[]
}

import { create } from 'zustand'

export const useAuth = create<AuthState>((set) => ({
  token: localStorage.getItem('atlas_token'),
  user: JSON.parse(localStorage.getItem('atlas_user') || 'null'),
  login: (token, user) => {
    localStorage.setItem('atlas_token', token)
    localStorage.setItem('atlas_user', JSON.stringify(user))
    set({ token, user })
  },
  logout: () => {
    localStorage.removeItem('atlas_token')
    localStorage.removeItem('atlas_user')
    set({ token: null, user: null })
  },
}))

// ── Fetch helpers ────────────────────────────────────────────
async function request<T>(url: string, opts?: RequestInit): Promise<T> {
  const token = useAuth.getState().token
  const res = await fetch(url, {
    ...opts,
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...opts?.headers,
    },
  })
  if (!res.ok) {
    const body = await res.text().catch(() => res.statusText)
    throw new Error(`${res.status}: ${body}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

// ── Auth API ─────────────────────────────────────────────────
export interface LoginRequest {
  email: string
  password: string
}

export interface LoginResponse {
  token: string
  user: AuthUser
  expires_at: string
}

export function login(data: LoginRequest): Promise<LoginResponse> {
  return request('/api/v1/auth/login', {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

// ── Schema API ───────────────────────────────────────────────
export interface EntityDefinition {
  id?: string
  name: string
  label: string
  pluralLabel: string
  tableName?: string
  description?: string
  fields: FieldDefinition[]
  indexes: IndexDefinition[]
  workflow?: WorkflowDefinition
  isAuditEnabled: boolean
  isSoftDelete: boolean
  icon?: string
  color?: string
  metadata: unknown
}

export interface FieldDefinition {
  id?: string
  name: string
  label: string
  fieldType: FieldTypeUnion
  isRequired: boolean
  isUnique: boolean
  isReadOnly: boolean
  isSearchable: boolean
  defaultValue?: unknown
  helpText?: string
  displayOrder: number
  placeholder?: string
  validations: unknown[]
  visibility: { condition?: string; roles: string[]; hidden: boolean }
  formatting?: unknown
}

export type FieldTypeUnion =
  | { type: 'string'; maxLength?: number; pattern?: string }
  | { type: 'fixed_string'; length: number }
  | { type: 'integer'; min?: number; max?: number }
  | { type: 'decimal'; precision: number; scale: number }
  | { type: 'boolean' }
  | { type: 'date' }
  | { type: 'date_time' }
  | { type: 'reference'; entity: string; field?: string }
  | { type: 'one_to_many'; entity: string; foreignKey: string }
  | { type: 'one_to_one'; entity: string; foreignKey: string }
  | { type: 'enum'; values: string[] }
  | { type: 'computed'; formula: string; returnType: FieldTypeUnion }
  | { type: 'attachment' }
  | { type: 'currency'; code: string }
  | { type: 'rich_text' }
  | { type: 'json' }
  | { type: 'email' }
  | { type: 'url' }
  | { type: 'phone' }
  | { type: 'address' }

export interface IndexDefinition {
  name: string
  fields: string[]
  isUnique: boolean
}

export interface WorkflowDefinition {
  id?: string
  name: string
  initialState: string
  states: StateDefinition[]
  transitions: TransitionDefinition[]
  isActive: boolean
}

export interface StateDefinition {
  name: string
  label: string
  stateType: 'initial' | 'working' | 'final'
  entryActions: unknown[]
  exitActions: unknown[]
  metadata: unknown
}

export interface TransitionDefinition {
  name: string
  fromState: string
  toState: string
  action: string
  actionLabel?: string
  guards: unknown[]
  requiredRoles: string[]
  entryActions: unknown[]
  metadata: unknown
}

export function getEntitySchema(entity: string): Promise<EntityDefinition> {
  return request<EntityDefinition>(`${API_BASE}/schema/${entity}`)
}

export function getEntityForm(entity: string): Promise<FormConfig> {
  return request<FormConfig>(`${API_BASE}/schema/${entity}/form`)
}

export function getEntityListView(entity: string): Promise<ListViewConfig> {
  return request<ListViewConfig>(`${API_BASE}/schema/${entity}/list`)
}

export interface FormConfig {
  entity: string
  fields: FormFieldConfig[]
}

export interface FormFieldConfig {
  name: string
  label: string
  fieldType: string
  required: boolean
  visible: boolean
  editable: boolean
  placeholder?: string
  helpText?: string
  options?: string[]
}

export interface ListViewConfig {
  entity: string
  columns: { field: string; label: string; width?: number; sortable: boolean }[]
  defaultSort?: string
}

// ── Records API ──────────────────────────────────────────────
export interface PaginatedResponse<T> {
  data: T[]
  meta: { total: number; offset: number; limit: number; has_more?: boolean }
}

export interface ListParams {
  search?: string
  offset?: number
  limit?: number
  sort?: string
  order?: string
}

export function listRecords(entity: string, params?: ListParams): Promise<PaginatedResponse<Record<string, unknown>>> {
  const qs = new URLSearchParams()
  if (params?.search) qs.set('search', params.search)
  if (params?.offset != null) qs.set('offset', String(params.offset))
  if (params?.limit != null) qs.set('limit', String(params.limit))
  if (params?.sort) qs.set('sort', params.sort)
  if (params?.order) qs.set('order', params.order)
  const sep = qs.toString() ? '?' : ''
  return request(`${API_BASE}/${entity}${sep}${qs}`)
}

export function getRecord(entity: string, id: string): Promise<Record<string, unknown>> {
  return request(`${API_BASE}/${entity}/${id}`)
}

export function createRecord(entity: string, values: Record<string, unknown>): Promise<Record<string, unknown>> {
  return request(`${API_BASE}/${entity}`, {
    method: 'POST',
    body: JSON.stringify({ entity, values }),
  })
}

export function updateRecord(entity: string, id: string, values: Record<string, unknown>): Promise<Record<string, unknown>> {
  return request(`${API_BASE}/${entity}/${id}`, {
    method: 'PUT',
    body: JSON.stringify({ entity, id, values }),
  })
}

export function deleteRecord(entity: string, id: string): Promise<void> {
  return request(`${API_BASE}/${entity}/${id}`, { method: 'DELETE' })
}

// ── Workflow API ─────────────────────────────────────────────
export interface TransitionsResponse {
  current_state: string
  transitions: {
    name: string
    action: string
    action_label?: string
    to_state: string
    required_roles: string[]
    has_guards: boolean
  }[]
}

export function getTransitions(entity: string, id: string): Promise<TransitionsResponse> {
  return request(`${API_BASE}/${entity}/${id}/transitions`)
}

export function executeAction(
  entity: string,
  id: string,
  action: string,
  body?: { comment?: string; values?: Record<string, unknown> },
): Promise<{ success: boolean; from_state: string; to_state: string; action: string; executed_actions: string[]; error?: string }> {
  return request(`${API_BASE}/${entity}/${id}/${action}`, {
    method: 'POST',
    body: JSON.stringify(body ?? { action }),
  })
}

// ── Audit API ────────────────────────────────────────────────
export function getRecordHistory(entity: string, id: string): Promise<{ entity: string; id: string; history: AuditEntry[] }> {
  return request(`${API_BASE}/${entity}/${id}/history`)
}

export interface AuditEntry {
  id: string
  entityType: string
  entityId: string
  action: string
  oldData: unknown
  newData: unknown
  changedBy?: string
  changedAt: string
  sessionId?: string
  ipAddress?: string
  userAgent?: string
}

// ── Reports API ──────────────────────────────────────────────
export function getDashboardReport(): Promise<ReportResponse> {
  return request(`${API_BASE}/reports/dashboard`)
}

export function getEntityReport(entity: string): Promise<ReportResponse> {
  return request(`${API_BASE}/reports/${entity}`)
}

export interface ReportResponse {
  report_type: string
  generated_at: string
  data: Record<string, unknown>
}

// ── Admin API ────────────────────────────────────────────────
export function getAdminConfig(): Promise<{ entities: string[]; version: number }> {
  return request(`${ADMIN_BASE}/config`)
}

export function createEntity(definition: EntityDefinition): Promise<{ entity: string; table_created: boolean; workflow_loaded: boolean }> {
  return request(`${ADMIN_BASE}/schema`, {
    method: 'POST',
    body: JSON.stringify({ definition, create_table: true }),
  })
}

export function updateEntity(entity: string, definition: EntityDefinition): Promise<{ entity: string; updated: boolean }> {
  return request(`${ADMIN_BASE}/schema/${entity}`, {
    method: 'PUT',
    body: JSON.stringify({ definition }),
  })
}

export function deleteEntity(entity: string, dropTable = false): Promise<{ entity: string; table_dropped: boolean; definition_removed: boolean }> {
  return request(`${ADMIN_BASE}/schema/${entity}`, {
    method: 'DELETE',
    body: JSON.stringify({ drop_table: dropTable }),
  })
}

// ── Import / Export ──────────────────────────────────────────
export function exportData(entity: string): Promise<{ entity: string; count: number; data: Record<string, unknown>[] }> {
  return request(`${API_BASE}/export/${entity}`)
}

export function importData(entity: string, data: Record<string, unknown>[]): Promise<{ entity: string; imported: number; errors: string[] }> {
  return request(`${API_BASE}/import`, {
    method: 'POST',
    body: JSON.stringify({ entity, format: 'json', data, upsert: false }),
  })
}
