import type { ProjectDocument } from './project-api.service';

function metadataString(document: ProjectDocument, key: string): string {
  const value = document.metadata[key];
  return typeof value === 'string' ? value.trim() : '';
}

/** Raw trimmed `contentType` from metadata (may be empty). */
export function documentContentType(document: ProjectDocument): string {
  return metadataString(document, 'contentType');
}

/** Display label for upload metadata `originalFilename`, or fallback. */
export function documentDisplayName(document: ProjectDocument): string {
  const name = metadataString(document, 'originalFilename');
  return name.length > 0 ? name : 'Untitled document';
}

/** Coarse category for picker rows (aligned with Project Detail wording). */
export function documentDisplayType(document: ProjectDocument): string {
  const contentType = documentContentType(document).toLowerCase();
  const name = documentDisplayName(document);
  const extension = name.split('.').pop()?.trim().toLowerCase();

  if (
    contentType.startsWith('image/') ||
    ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(extension ?? '')
  ) {
    return 'Image';
  }

  if (
    contentType.startsWith('text/') ||
    ['txt', 'md', 'csv'].includes(extension ?? '')
  ) {
    return 'Text';
  }

  return 'Document';
}

/** Human-readable size from metadata `size`. */
export function documentDisplaySize(document: ProjectDocument): string {
  const rawSize = document.metadata['size'];
  const size = typeof rawSize === 'number' ? rawSize : Number(rawSize);
  if (!Number.isFinite(size) || size < 0) return 'Unknown';
  if (size < 1024) return `${size} B`;

  const units = ['KB', 'MB', 'GB'];
  let value = size / 1024;
  for (const unit of units) {
    if (value < 1024) return `${value.toFixed(value >= 10 ? 0 : 1)} ${unit}`;
    value /= 1024;
  }
  return `${value.toFixed(1)} TB`;
}
