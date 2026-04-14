import { z } from 'zod'
import type { EntityDefinition, FieldDefinition, FieldTypeUnion } from './api'

/**
 * Generate a Zod schema from an Atlas EntityDefinition.
 * Used by the dynamic form generator for type-safe validation.
 */
export function buildZodSchema(entity: EntityDefinition): z.ZodObject<any> {
  const shape: Record<string, z.ZodTypeAny> = {}

  for (const field of entity.fields) {
    if (['id', 'created_at', 'updated_at', 'deleted_at', 'organization_id'].includes(field.name)) continue
    if (field.isReadOnly && field.fieldType.type !== 'computed') continue

    let fieldSchema = fieldTypeToZod(field.fieldType, field.name)

    if (field.isRequired) {
      fieldSchema = fieldSchema.describe(field.label || field.name)
    } else {
      fieldSchema = fieldSchema.optional().or(z.literal(''))
    }

    shape[field.name] = fieldSchema
  }

  return z.object(shape)
}

function fieldTypeToZod(ft: FieldTypeUnion, name: string): z.ZodTypeAny {
  switch (ft.type) {
    case 'string':
    case 'email':
    case 'url':
    case 'phone':
    case 'fixed_string': {
      let s = z.string()
      if (ft.type === 'email') s = z.string().email()
      if (ft.type === 'url') s = z.string().url()
      if (ft.type === 'string' && 'maxLength' in ft && ft.maxLength) s = s.max(ft.maxLength)
      if (ft.type === 'fixed_string') s = s.length(ft.length)
      if (ft.type === 'string' && 'pattern' in ft && ft.pattern) {
        s = s.regex(new RegExp(ft.pattern), `Invalid format for ${name}`)
      }
      return s
    }
    case 'integer': {
      let s = z.coerce.number().int()
      if (ft.min != null) s = s.min(ft.min as number)
      if (ft.max != null) s = s.max(ft.max as number)
      return s
    }
    case 'decimal': {
      let s = z.coerce.number()
      if ('min' in ft && ft.min != null) s = s.min(ft.min as number)
      if ('max' in ft && ft.max != null) s = s.max(ft.max as number)
      return s
    }
    case 'boolean':
      return z.coerce.boolean()
    case 'date':
      return z.string().min(1)
    case 'date_time':
      return z.string().min(1)
    case 'enum':
      if (ft.values.length === 0) return z.string()
      return z.enum(ft.values as [string, ...string[]])
    case 'reference':
      return z.string().uuid().or(z.string().min(1))
    case 'currency':
      return z.coerce.number()
    case 'rich_text':
      return z.string()
    case 'json':
      return z.any()
    case 'computed':
      // computed fields are read-only, allow any for display
      return z.any().optional()
    case 'one_to_many':
    case 'one_to_one':
      return z.any().optional()
    case 'attachment':
      return z.any().optional()
    default:
      return z.string()
  }
}

/**
 * Get default values for a form from the entity schema.
 */
export function getDefaultValues(entity: EntityDefinition): Record<string, unknown> {
  const defaults: Record<string, unknown> = {}
  for (const field of entity.fields) {
    if (['id', 'created_at', 'updated_at', 'deleted_at', 'organization_id'].includes(field.name)) continue
    if (field.isReadOnly) continue
    defaults[field.name] = field.defaultValue ?? getFieldTypeDefault(field.fieldType)
  }
  return defaults
}

function getFieldTypeDefault(ft: FieldTypeUnion): unknown {
  switch (ft.type) {
    case 'boolean': return false
    case 'integer':
    case 'decimal':
    case 'currency': return ''
    case 'enum': return ''
    case 'date':
    case 'date_time': return ''
    default: return ''
  }
}

/**
 * Get the HTML input type for a field.
 */
export function getInputType(ft: FieldTypeUnion): string {
  switch (ft.type) {
    case 'integer':
    case 'decimal':
    case 'currency': return 'number'
    case 'date': return 'date'
    case 'date_time': return 'datetime-local'
    case 'boolean': return 'checkbox'
    case 'email': return 'email'
    case 'url': return 'url'
    case 'phone': return 'tel'
    default: return 'text'
  }
}

/**
 * Check if a field should be rendered as a select dropdown.
 */
export function isSelectField(ft: FieldTypeUnion): boolean {
  return ft.type === 'enum'
}

/**
 * Check if a field should be rendered as a textarea.
 */
export function isTextareaField(ft: FieldTypeUnion): boolean {
  return ft.type === 'rich_text' || (ft.type === 'string' && 'maxLength' in ft && (ft as any).maxLength != null && (ft as any).maxLength > 200)
}

/**
 * Get editable fields from an entity definition.
 */
export function getEditableFields(entity: EntityDefinition): FieldDefinition[] {
  return entity.fields.filter(
    (f) => !f.isReadOnly && !['id', 'created_at', 'updated_at', 'deleted_at', 'organization_id'].includes(f.name)
  )
}

/**
 * Get displayable fields from an entity definition.
 */
export function getDisplayFields(entity: EntityDefinition): FieldDefinition[] {
  return entity.fields.filter(
    (f) => !['id', 'organization_id'].includes(f.name)
  )
}
