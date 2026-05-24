import { AdminAuditLogEntry } from './admin-audit-api.service';

export interface AuditFieldRow {
  key: string;
  label: string;
  value: string;
}

export interface AuditRowChangeDisplay {
  kind: 'row_change';
  op: 'c' | 'u' | 'd';
  previous: AuditFieldRow[];
  next: AuditFieldRow[];
  showPrevious: boolean;
  showNext: boolean;
}

export interface AuditEventDisplay {
  kind: 'event';
  details: AuditFieldRow[];
}

export type AuditMetadataDisplay = AuditRowChangeDisplay | AuditEventDisplay;

const ROW_CHANGE_OPS = new Set(['c', 'u', 'd']);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function formatFieldLabel(key: string): string {
  return key
    .split('_')
    .filter((part) => part.length > 0)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function formatFieldValue(value: unknown): string {
  if (value === null || value === undefined) {
    return '—';
  }
  if (typeof value === 'string') {
    return value.trim() === '' ? '—' : value;
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value);
  }
  if (Array.isArray(value)) {
    if (value.length === 0) {
      return '—';
    }
    return value.map((item) => formatFieldValue(item)).join(', ');
  }
  if (isRecord(value)) {
    const keys = Object.keys(value);
    if (keys.length === 0) {
      return '—';
    }
    return keys.map((k) => `${formatFieldLabel(k)}: ${formatFieldValue(value[k])}`).join('; ');
  }
  return String(value);
}

function sortedKeys(...objects: Array<Record<string, unknown> | null>): string[] {
  const keys = new Set<string>();
  for (const obj of objects) {
    if (!obj) {
      continue;
    }
    for (const key of Object.keys(obj)) {
      keys.add(key);
    }
  }
  return [...keys].sort((a, b) => a.localeCompare(b));
}

function valuesEqual(a: unknown, b: unknown): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

function objectToFieldRows(obj: Record<string, unknown>, keys: string[]): AuditFieldRow[] {
  return keys.map((key) => ({
    key,
    label: formatFieldLabel(key),
    value: formatFieldValue(obj[key]),
  }));
}

function changedKeys(before: Record<string, unknown>, after: Record<string, unknown>): string[] {
  return sortedKeys(before, after).filter((key) => !valuesEqual(before[key], after[key]));
}

function parseRowChange(metadata: Record<string, unknown>): AuditRowChangeDisplay | null {
  const op = metadata['op'];
  if (typeof op !== 'string' || !ROW_CHANGE_OPS.has(op)) {
    return null;
  }

  const before = isRecord(metadata['before']) ? metadata['before'] : null;
  const after = isRecord(metadata['after']) ? metadata['after'] : null;

  if (op === 'c') {
    const keys = after ? sortedKeys(after) : [];
    return {
      kind: 'row_change',
      op: 'c',
      previous: [],
      next: after ? objectToFieldRows(after, keys) : [],
      showPrevious: false,
      showNext: keys.length > 0,
    };
  }

  if (op === 'd') {
    const keys = before ? sortedKeys(before) : [];
    return {
      kind: 'row_change',
      op: 'd',
      previous: before ? objectToFieldRows(before, keys) : [],
      next: [],
      showPrevious: keys.length > 0,
      showNext: false,
    };
  }

  if (!before && !after) {
    return {
      kind: 'row_change',
      op: 'u',
      previous: [],
      next: [],
      showPrevious: false,
      showNext: false,
    };
  }

  const keys = before && after ? changedKeys(before, after) : sortedKeys(before, after);
  return {
    kind: 'row_change',
    op: 'u',
    previous: before ? objectToFieldRows(before, keys) : [],
    next: after ? objectToFieldRows(after, keys) : [],
    showPrevious: before !== null && keys.length > 0,
    showNext: after !== null && keys.length > 0,
  };
}

function parseEventDetails(metadata: Record<string, unknown>): AuditEventDisplay {
  const keys = sortedKeys(metadata).filter((key) => key !== 'entity_type' && key !== 'entity_id');
  return {
    kind: 'event',
    details: objectToFieldRows(metadata, keys),
  };
}

export function buildAuditMetadataDisplay(entry: AdminAuditLogEntry): AuditMetadataDisplay {
  const metadata = entry.metadata;
  if (!isRecord(metadata)) {
    return { kind: 'event', details: [] };
  }

  const rowChange = parseRowChange(metadata);
  if (rowChange) {
    return rowChange;
  }

  return parseEventDetails(metadata);
}

export function hasExpandableMetadata(entry: AdminAuditLogEntry): boolean {
  const display = buildAuditMetadataDisplay(entry);
  if (display.kind === 'event') {
    return display.details.length > 0;
  }
  return display.showPrevious || display.showNext;
}
