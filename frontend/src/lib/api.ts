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
  const qs = dropTable ? '?drop_table=true' : ''
  return request(`${ADMIN_BASE}/schema/${entity}${qs}`, {
    method: 'DELETE',
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

// ══════════════════════════════════════════════════════════════════
// Oracle Fusion-Inspired Features
// ══════════════════════════════════════════════════════════════════

// ── Notifications (Bell Icon) ─────────────────────────────────

export interface Notification {
  id: string
  user_id: string
  notification_type: string
  priority: string
  title: string
  message: string | null
  entity_type: string | null
  entity_id: string | null
  action_url: string | null
  workflow_name: string | null
  from_state: string | null
  to_state: string | null
  action: string | null
  performed_by: string | null
  is_read: boolean
  is_dismissed: boolean
  channels: unknown
  metadata: unknown
  created_at: string
  expires_at: string | null
}

export function listNotifications(params?: { include_read?: boolean; limit?: number; offset?: number }): Promise<{ data: Notification[]; meta: { include_read: boolean; limit: number; offset: number } }> {
  const qs = new URLSearchParams()
  if (params?.include_read !== undefined) qs.set('include_read', String(params.include_read))
  if (params?.limit !== undefined) qs.set('limit', String(params.limit))
  if (params?.offset !== undefined) qs.set('offset', String(params.offset))
  const sep = qs.toString() ? '?' : ''
  return request(`${API_BASE}/notifications${sep}${qs}`)
}

export function getUnreadNotificationCount(): Promise<{ count: number }> {
  return request(`${API_BASE}/notifications/unread-count`)
}

export function markNotificationRead(id: string): Promise<void> {
  return request(`${API_BASE}/notifications/${id}/read`, { method: 'PUT' })
}

export function markAllNotificationsRead(): Promise<{ marked_read: number }> {
  return request(`${API_BASE}/notifications/read-all`, { method: 'PUT' })
}

export function dismissNotification(id: string): Promise<void> {
  return request(`${API_BASE}/notifications/${id}/dismiss`, { method: 'PUT' })
}

// ── Saved Searches (Personalized Views) ──────────────────────

export interface SavedSearch {
  id: string
  name: string
  description: string | null
  entity_type: string
  filters: unknown
  sort_by: string
  sort_direction: string
  columns: unknown
  columns_widths: unknown
  page_size: number
  is_shared: boolean
  is_default: boolean
  color: string | null
  icon: string | null
  created_at: string
  updated_at: string
}

export interface CreateSavedSearchRequest {
  name: string
  entity_type: string
  description?: string
  filters?: unknown
  sort_by?: string
  sort_direction?: string
  columns?: unknown
  columns_widths?: unknown
  page_size?: number
  is_shared?: boolean
  is_default?: boolean
  color?: string
  icon?: string
}

export function listSavedSearches(entityType?: string): Promise<{ data: SavedSearch[] }> {
  const qs = entityType ? `?entity=${entityType}` : ''
  return request(`${API_BASE}/saved-searches${qs}`)
}

export function createSavedSearch(data: CreateSavedSearchRequest): Promise<unknown> {
  return request(`${API_BASE}/saved-searches`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

export function deleteSavedSearch(id: string): Promise<void> {
  return request(`${API_BASE}/saved-searches/${id}`, { method: 'DELETE' })
}

// ── Approval Chains (Multi-Level Approval) ─────────────────────

export interface ApprovalChain {
  id: string
  name: string
  description: string | null
  entity_type: string
  condition_expression: string | null
  chain_definition: ApprovalLevel[]
  escalation_enabled: boolean
  escalation_hours: number
  escalation_to_roles: string[]
  allow_delegation: boolean
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface ApprovalLevel {
  level: number
  approver_type: string  // 'role', 'user', 'auto'
  roles: string[]
  user_ids?: string[]
  auto_approve_after_hours?: number
}

export interface ApprovalStep {
  id: string
  approval_request_id: string
  level: number
  approver_type: string
  approver_role: string | null
  approver_user_id: string | null
  is_delegated: boolean
  delegated_by: string | null
  delegated_to: string | null
  status: string
  action_at: string | null
  action_by: string | null
  comment: string | null
  auto_approve_after_hours: number | null
  created_at: string
  updated_at: string
}

export interface ApprovalRequest {
  id: string
  chain_id: string | null
  entity_type: string
  entity_id: string
  current_level: number
  total_levels: number
  status: string
  requested_by: string
  requested_at: string
  completed_at: string | null
  completed_by: string | null
  title: string | null
  description: string | null
  metadata: unknown
  steps: ApprovalStep[]
  created_at: string
  updated_at: string
}

export function listApprovalChains(entityType?: string): Promise<{ data: ApprovalChain[] }> {
  const qs = entityType ? `?entity_type=${entityType}` : ''
  return request(`${API_BASE}/approval-chains${qs}`)
}

export function createApprovalChain(data: {
  name: string
  entity_type: string
  description?: string
  condition_expression?: string
  chain_definition: ApprovalLevel[]
  escalation_enabled?: boolean
  escalation_hours?: number
  escalation_to_roles?: string[]
  allow_delegation?: boolean
}): Promise<unknown> {
  return request(`${API_BASE}/approval-chains`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

export function getPendingApprovals(): Promise<{ data: ApprovalStep[]; meta: { user_steps: number; role_steps: number } }> {
  return request(`${API_BASE}/approvals/pending`)
}

export function approveApprovalStep(stepId: string, comment?: string): Promise<unknown> {
  return request(`${API_BASE}/approvals/${stepId}/approve`, {
    method: 'POST',
    body: JSON.stringify({ comment }),
  })
}

export function rejectApprovalStep(stepId: string, comment?: string): Promise<unknown> {
  return request(`${API_BASE}/approvals/${stepId}/reject`, {
    method: 'POST',
    body: JSON.stringify({ comment }),
  })
}

export function delegateApprovalStep(stepId: string, delegatedTo: string): Promise<unknown> {
  return request(`${API_BASE}/approvals/${stepId}/delegate`, {
    method: 'POST',
    body: JSON.stringify({ delegated_to: delegatedTo }),
  })
}

// ── Duplicate Detection ───────────────────────────────────────

export interface DuplicateResult {
  has_duplicates: boolean
  duplicates: Array<{
    rule_id: string
    rule_name: string
    existing_record_id: string
    match_fields: string[]
    action: string
    existing_data: Record<string, unknown>
  }>
}

export function checkDuplicates(entityType: string, data: Record<string, unknown>): Promise<DuplicateResult> {
  return request(`${API_BASE}/duplicates/check`, {
    method: 'POST',
    body: JSON.stringify({ entity_type: entityType, data }),
  })
}

export function createDuplicateRule(data: {
  name: string
  entity_type: string
  description?: string
  match_criteria: Array<{ field: string; match_type: string; threshold?: number }>
  on_duplicate?: string
}): Promise<unknown> {
  return request(`${ADMIN_BASE}/duplicate-rules`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

// ══════════════════════════════════════════════════════════════════
// Advanced Oracle Fusion Features (Phase 2)
// ══════════════════════════════════════════════════════════════════

// ── Structured Filtering (Faceted Search) ──────────────────────

export type FilterExpression =
  | { type: 'condition'; field: string; operator: string; value: unknown }
  | { type: 'and'; conditions: FilterExpression[] }
  | { type: 'or'; conditions: FilterExpression[] }

export interface AdvancedListParams {
  search?: string
  offset?: number
  limit?: number
  sort?: string
  order?: string
  filter?: FilterExpression
}

export function listRecordsAdvanced(
  entity: string,
  params?: AdvancedListParams,
): Promise<PaginatedResponse<Record<string, unknown>>> {
  const qs = new URLSearchParams()
  if (params?.search) qs.set('search', params.search)
  if (params?.offset != null) qs.set('offset', String(params.offset))
  if (params?.limit != null) qs.set('limit', String(params.limit))
  if (params?.sort) qs.set('sort', params.sort)
  if (params?.order) qs.set('order', params.order)
  if (params?.filter) qs.set('filter', JSON.stringify(params.filter))
  const sep = qs.toString() ? '?' : ''
  return request(`${API_BASE}/${entity}/filtered${sep}${qs}`)
}

// ── Bulk Operations ──────────────────────────────────────────────

export interface BulkOperationRequest {
  entity_type: string
  operation: 'update' | 'delete' | 'workflow_action'
  filter?: FilterExpression
  record_ids?: string[]
  payload: Record<string, unknown>
  dry_run?: boolean
}

export interface BulkOperationResponse {
  id: string
  operation: string
  status: string
  total_records: number
  succeeded: number
  failed: number
  errors: Array<{ record_id: string; error: string }>
  is_dry_run: boolean
}

export function executeBulkOperation(data: BulkOperationRequest): Promise<BulkOperationResponse> {
  return request(`${API_BASE}/bulk`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

// ── Comments / Notes ─────────────────────────────────────────────

export interface Comment {
  id: string
  entity_type: string
  entity_id: string
  parent_id: string | null
  user_id: string
  user_name: string | null
  body: string
  body_format: string
  mentions: string[]
  is_pinned: boolean
  is_internal: boolean
  depth: number
  created_at: string
  updated_at: string
}

export function listComments(
  entity: string,
  id: string,
  params?: { limit?: number; offset?: number },
): Promise<{ data: Comment[]; meta: { total: number; offset: number; limit: number } }> {
  const qs = new URLSearchParams()
  if (params?.limit != null) qs.set('limit', String(params.limit))
  if (params?.offset != null) qs.set('offset', String(params.offset))
  const sep = qs.toString() ? '?' : ''
  return request(`${API_BASE}/${entity}/${id}/comments${sep}${qs}`)
}

export function createComment(
  entity: string,
  id: string,
  data: { body: string; parent_id?: string; is_internal?: boolean; mentions?: string[] },
): Promise<Comment> {
  return request(`${API_BASE}/${entity}/${id}/comments`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

export function deleteComment(entity: string, recordId: string, commentId: string): Promise<void> {
  return request(`${API_BASE}/${entity}/${recordId}/comments/${commentId}`, { method: 'DELETE' })
}

export function togglePinComment(entity: string, recordId: string, commentId: string): Promise<{ id: string; is_pinned: boolean }> {
  return request(`${API_BASE}/${entity}/${recordId}/comments/${commentId}/pin`, { method: 'PUT' })
}

// ── Favorites / Bookmarks ────────────────────────────────────────

export interface Favorite {
  id: string
  entity_type: string
  entity_id: string
  label: string | null
  notes: string | null
  display_order: number
  created_at: string
}

export function listFavorites(entityType?: string): Promise<{ data: Favorite[] }> {
  const qs = entityType ? `?entity_type=${entityType}` : ''
  return request(`${API_BASE}/favorites${qs}`)
}

export function addFavorite(
  entity: string,
  id: string,
  data?: { label?: string; notes?: string },
): Promise<Favorite> {
  return request(`${API_BASE}/${entity}/${id}/favorite`, {
    method: 'POST',
    body: JSON.stringify(data ?? {}),
  })
}

export function removeFavorite(entity: string, id: string): Promise<void> {
  return request(`${API_BASE}/${entity}/${id}/favorite`, { method: 'DELETE' })
}

export function checkFavorite(entity: string, id: string): Promise<{ is_favorite: boolean; favorite?: Favorite }> {
  return request(`${API_BASE}/${entity}/${id}/favorite`)
}

// ── CSV Export ────────────────────────────────────────────────────

export function exportCsv(
  entity: string,
  params?: { filter?: FilterExpression; fields?: string[] },
): Promise<Blob> {
  const qs = new URLSearchParams()
  if (params?.fields) qs.set('fields', params.fields.join(','))
  if (params?.filter) qs.set('filter', JSON.stringify(params.filter))
  const sep = qs.toString() ? '?' : ''
  const token = useAuth.getState().token
  // CSV export returns a blob, not JSON
  return fetch(`${API_BASE}/export/${entity}/csv${sep}${qs}`, {
    headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) },
  }).then((res) => {
    if (!res.ok) throw new Error(`Export failed: ${res.status}`)
    return res.blob()
  })
}

// ── CSV Import ────────────────────────────────────────────────────

export interface CsvImportRequest {
  entity: string
  csv_content: string
  field_mapping?: Record<string, string>
  upsert_mode?: boolean
  skip_validation?: boolean
  stop_on_error?: boolean
  delimiter?: string
}

export interface CsvImportResponse {
  entity: string
  total_rows: number
  imported: number
  failed: number
  skipped: number
  errors: Array<{ row: number; errors: string[] }>
  preview?: {
    columns: string[]
    mapped_fields: Array<[string, string]>
    sample_rows: string[][]
    unmapped_columns: string[]
  }
}

export function importCsv(data: CsvImportRequest): Promise<CsvImportResponse> {
  return request(`${API_BASE}/import/csv`, {
    method: 'POST',
    body: JSON.stringify({
      ...data,
      field_mapping: data.field_mapping ?? {},
      delimiter: data.delimiter ?? ',',
    }),
  })
}

// ── Related Records ──────────────────────────────────────────────

export function getRelatedRecords(
  entity: string,
  id: string,
  relatedEntity: string,
  params?: { limit?: number; offset?: number },
): Promise<{
  data: Record<string, unknown>[]
  meta: { parent_entity: string; parent_id: string; related_entity: string; foreign_key: string; total: number; offset: number; limit: number }
}> {
  const qs = new URLSearchParams()
  if (params?.limit != null) qs.set('limit', String(params.limit))
  if (params?.offset != null) qs.set('offset', String(params.offset))
  const sep = qs.toString() ? '?' : ''
  return request(`${API_BASE}/${entity}/${id}/related/${relatedEntity}${sep}${qs}`)
}

// ── Effective Dating ─────────────────────────────────────────────

export interface EffectiveVersion {
  id: string
  entity_type: string
  base_record_id: string
  effective_from: string
  effective_to: string | null
  data: Record<string, unknown>
  change_reason: string | null
  changed_by: string | null
  version: number
  is_current: boolean
  created_at: string
}

export function getEffectiveRecord(
  entity: string,
  id: string,
  params?: { as_of_date?: string; include_history?: boolean },
): Promise<EffectiveVersion | { entity_type: string; base_record_id: string; versions: EffectiveVersion[] }> {
  const qs = new URLSearchParams()
  if (params?.as_of_date) qs.set('as_of_date', params.as_of_date)
  if (params?.include_history) qs.set('include_history', 'true')
  const sep = qs.toString() ? '?' : ''
  return request(`${API_BASE}/${entity}/${id}/effective${sep}${qs}`)
}

export function createEffectiveVersion(
  entity: string,
  id: string,
  data: { effective_from: string; effective_to?: string; data: Record<string, unknown>; change_reason?: string },
): Promise<EffectiveVersion> {
  return request(`${API_BASE}/${entity}/${id}/effective`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}
