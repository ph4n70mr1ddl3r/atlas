import { useForm, Controller } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import type { EntityDefinition, FieldDefinition } from '@/lib/api'
import { buildZodSchema, getDefaultValues, getInputType, isSelectField, isTextareaField, getEditableFields } from '@/lib/schema-utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { cn } from '@/lib/utils'
import { Loader2 } from 'lucide-react'

export interface DynamicFormProps {
  schema: EntityDefinition
  initialData?: Record<string, unknown>
  onSubmit: (values: Record<string, unknown>) => Promise<void>
  onCancel?: () => void
  submitLabel?: string
  cancelLabel?: string
  /** Show only a subset of fields */
  visibleFields?: string[]
  /** Disable specific fields */
  disabledFields?: string[]
}

/**
 * Schema-driven dynamic form generator.
 * Uses React Hook Form + Zod for type-safe validation.
 * Renders appropriate inputs based on field type definitions.
 */
export function DynamicForm({
  schema,
  initialData,
  onSubmit,
  onCancel,
  submitLabel = 'Save',
  cancelLabel = 'Cancel',
  visibleFields,
  disabledFields = [],
}: DynamicFormProps) {
  const zodSchema = buildZodSchema(schema)
  const defaults = initialData ? { ...getDefaultValues(schema), ...initialData } : getDefaultValues(schema)
  const editableFields = getEditableFields(schema)
  const fields = visibleFields
    ? editableFields.filter((f) => visibleFields.includes(f.name))
    : editableFields

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm({
    resolver: zodResolver(zodSchema),
    defaultValues: defaults,
  })

  const processSubmit = async (data: Record<string, unknown>) => {
    // Clean up empty strings for optional number fields
    const cleaned: Record<string, unknown> = {}
    for (const field of editableFields) {
      const val = data[field.name]
      if (val === '' || val === undefined) {
        if (!field.isRequired) {
          cleaned[field.name] = null
        } else {
          cleaned[field.name] = val
        }
      } else {
        cleaned[field.name] = val
      }
    }
    await onSubmit(cleaned)
  }

  return (
    <form onSubmit={handleSubmit(processSubmit)} className="space-y-4 max-h-[65vh] overflow-y-auto pr-2">
      {fields.map((field) => (
        <FormField
          key={field.name}
          field={field}
          control={control}
          error={errors[field.name]}
          disabled={disabledFields.includes(field.name)}
        />
      ))}
      <div className="flex justify-end gap-2 pt-4 border-t">
        {onCancel && (
          <Button type="button" variant="outline" onClick={onCancel} disabled={isSubmitting}>
            {cancelLabel}
          </Button>
        )}
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {submitLabel}
        </Button>
      </div>
    </form>
  )
}

interface FormFieldProps {
  field: FieldDefinition
  control: any
  error?: any
  disabled?: boolean
}

function FormField({ field, control, error, disabled }: FormFieldProps) {
  const ft = field.fieldType

  // Skip computed / attachment / json fields in forms
  if (ft.type === 'computed' || ft.type === 'attachment' || ft.type === 'json') {
    return null
  }

  return (
    <div className="space-y-1.5">
      <Label htmlFor={field.name} className="text-sm font-medium">
        {field.label || field.name}
        {field.isRequired && <span className="text-destructive ml-1">*</span>}
      </Label>

      {field.helpText && (
        <p className="text-xs text-muted-foreground">{field.helpText}</p>
      )}

      <Controller
        name={field.name}
        control={control}
        render={({ field: formField }) => {
          if (isSelectField(ft)) {
            return (
              <select
                id={field.name}
                className={cn(
                  'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
                  error && 'border-destructive',
                  disabled && 'opacity-50 cursor-not-allowed',
                )}
                value={String(formField.value ?? '')}
                onChange={formField.onChange}
                onBlur={formField.onBlur}
                disabled={disabled}
              >
                <option value="">Select…</option>
                {ft.type === 'enum' && ft.values.map((v) => (
                  <option key={v} value={v}>{v.replace(/_/g, ' ')}</option>
                ))}
              </select>
            )
          }

          if (isTextareaField(ft)) {
            return (
              <Textarea
                id={field.name}
                className={cn(error && 'border-destructive')}
                placeholder={field.placeholder}
                value={String(formField.value ?? '')}
                onChange={formField.onChange}
                onBlur={formField.onBlur}
                disabled={disabled}
                rows={4}
              />
            )
          }

          if (ft.type === 'boolean') {
            return (
              <div className="flex items-center gap-2">
                <input
                  id={field.name}
                  type="checkbox"
                  checked={!!formField.value}
                  onChange={formField.onChange}
                  onBlur={formField.onBlur}
                  disabled={disabled}
                  className="h-4 w-4 rounded border-input"
                />
                <span className="text-sm text-muted-foreground">{field.placeholder || 'Enabled'}</span>
              </div>
            )
          }

          return (
            <Input
              id={field.name}
              type={getInputType(ft)}
              className={cn(error && 'border-destructive')}
              placeholder={field.placeholder}
              value={String(formField.value ?? '')}
              onChange={(e) => {
                if (ft.type === 'integer' || ft.type === 'decimal' || ft.type === 'currency') {
                  formField.onChange(e.target.value === '' ? '' : Number(e.target.value))
                } else {
                  formField.onChange(e.target.value)
                }
              }}
              onBlur={formField.onBlur}
              disabled={disabled}
              step={ft.type === 'decimal' ? '0.01' : ft.type === 'currency' ? '0.01' : undefined}
              min={ft.type === 'integer' ? (ft as any).min : ft.type === 'decimal' ? (ft as any).min : undefined}
              max={ft.type === 'integer' ? (ft as any).max : ft.type === 'decimal' ? (ft as any).max : undefined}
            />
          )
        }}
      />

      {error && (
        <p className="text-xs text-destructive">
          {error.message ?? 'This field is invalid'}
        </p>
      )}
    </div>
  )
}
