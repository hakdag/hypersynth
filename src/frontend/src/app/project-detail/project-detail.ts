import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { catchError, forkJoin, map, of, Subscription, switchMap } from 'rxjs';

import {
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
  CreatedFeature,
  ProjectDocument,
} from '../project-api.service';

type FeatureLoadResult =
  | { kind: 'ok'; features: CreatedFeature[] }
  | { kind: 'error'; message: string };

type DocumentLoadResult =
  | { kind: 'ok'; documents: ProjectDocument[] }
  | { kind: 'error'; message: string };

type DocumentPreviewKind = 'text' | 'image';

interface DocumentPreviewState {
  document: ProjectDocument;
  kind: DocumentPreviewKind;
  status: 'loading' | 'ready' | 'error';
  text: string | null;
  objectUrl: string | null;
  error: string | null;
}

type DetailResult =
  | { kind: 'invalid' }
  | {
      kind: 'ok';
      row: ProjectDetailModel;
      features: CreatedFeature[];
      documents: ProjectDocument[];
      featuresMessage: string | null;
      documentsMessage: string | null;
    }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-project-detail',
  imports: [RouterLink],
  templateUrl: './project-detail.html',
  styleUrl: './project-detail.scss',
})
export class ProjectDetail implements OnInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly projectApi = inject(ProjectApiService);
  private sub: Subscription | null = null;
  private uploadSub: Subscription | null = null;
  private documentsSub: Subscription | null = null;
  private downloadSub: Subscription | null = null;
  private viewSub: Subscription | null = null;
  private documentPreviewObjectUrl: string | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly detailError = signal<string | null>(null);
  protected readonly features = signal<CreatedFeature[]>([]);
  protected readonly featuresLoadError = signal<string | null>(null);
  protected readonly documents = signal<ProjectDocument[]>([]);
  protected readonly documentsLoadError = signal<string | null>(null);
  protected readonly requirementsExpanded = signal(false);
  protected readonly documentUploadModalOpen = signal(false);
  protected readonly selectedDocumentFiles = signal<File[]>([]);
  protected readonly documentUploadState = signal<'idle' | 'uploading'>('idle');
  protected readonly documentUploadError = signal<string | null>(null);
  protected readonly documentUploadSuccess = signal<string | null>(null);
  protected readonly documentDownloadError = signal<string | null>(null);
  protected readonly downloadingDocumentId = signal<string | null>(null);
  protected readonly documentPreviewModalOpen = signal(false);
  protected readonly documentPreview = signal<DocumentPreviewState | null>(null);

  private readonly rtf = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
  protected readonly acceptedDocumentTypes =
    '.md,.txt,.csv,.xls,.xlsx,.doc,.docx,.jpg,.jpeg,.png,.gif,.webp,.bmp,.svg';

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const id = params.get('projectId') ?? '';
          if (id.length === 0) {
            return of<DetailResult>({ kind: 'invalid' });
          }
          this.loadState.set('loading');
          this.detailError.set(null);
          this.featuresLoadError.set(null);
          this.documentsLoadError.set(null);
          this.documentDownloadError.set(null);
          this.downloadingDocumentId.set(null);
          this.closeDocumentViewModal();
          return this.projectApi.getProject(id).pipe(
            switchMap((row) =>
              forkJoin({
                featuresResult: this.projectApi.listFeatures(row.id).pipe(
                  map((features): FeatureLoadResult => ({ kind: 'ok', features })),
                  catchError((err: unknown) =>
                    of<FeatureLoadResult>({
                      kind: 'error',
                      message: ProjectApiService.listFeaturesErrorMessage(err),
                    }),
                  ),
                ),
                documentsResult: this.projectApi.listProjectDocuments(row.id).pipe(
                  map((documents): DocumentLoadResult => ({ kind: 'ok', documents })),
                  catchError((err: unknown) =>
                    of<DocumentLoadResult>({
                      kind: 'error',
                      message: ProjectApiService.listDocumentsErrorMessage(err),
                    }),
                  ),
                ),
              }).pipe(
                map(({ featuresResult, documentsResult }): DetailResult => ({
                  kind: 'ok',
                  row,
                  features: featuresResult.kind === 'ok' ? featuresResult.features : [],
                  documents: documentsResult.kind === 'ok' ? documentsResult.documents : [],
                  featuresMessage: featuresResult.kind === 'error' ? featuresResult.message : null,
                  documentsMessage: documentsResult.kind === 'error' ? documentsResult.message : null,
                })),
              ),
            ),
            catchError((err: unknown) =>
              of<DetailResult>({
                kind: 'error',
                message: ProjectApiService.detailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.detailError.set('No project identifier provided.');
          this.project.set(null);
          this.features.set([]);
          this.featuresLoadError.set(null);
          this.documents.set([]);
          this.documentsLoadError.set(null);
          this.documentDownloadError.set(null);
          this.downloadingDocumentId.set(null);
          this.requirementsExpanded.set(false);
          this.closeDocumentUploadModal();
          this.closeDocumentViewModal();
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.detailError.set(res.message);
          this.project.set(null);
          this.features.set([]);
          this.featuresLoadError.set(null);
          this.documents.set([]);
          this.documentsLoadError.set(null);
          this.documentDownloadError.set(null);
          this.downloadingDocumentId.set(null);
          this.requirementsExpanded.set(false);
          this.closeDocumentUploadModal();
          this.closeDocumentViewModal();
          this.loadState.set('error');
          return;
        }
        this.project.set(res.row);
        this.features.set(res.features);
        this.documents.set(res.documents);
        this.featuresLoadError.set(res.featuresMessage);
        this.documentsLoadError.set(res.documentsMessage);
        this.detailError.set(null);
        this.requirementsExpanded.set(false);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
    this.uploadSub?.unsubscribe();
    this.documentsSub?.unsubscribe();
    this.downloadSub?.unsubscribe();
    this.viewSub?.unsubscribe();
    this.revokeDocumentPreviewObjectUrl();
  }

  protected completionPercent(status: string): number {
    switch (status) {
      case 'In Progress':
        return 50;
      case 'Done':
        return 100;
      default:
        return 0;
    }
  }

  protected priorityLabel(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'Elevated';
      case 'Done':
        return 'Complete';
      default:
        return 'Standard';
    }
  }

  protected priorityIcon(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'priority_high';
      case 'Done':
        return 'check_circle';
      default:
        return 'flag';
    }
  }

  protected priorityIconClass(status: string): string {
    const base = 'material-symbols-outlined pd-priority__icon';
    switch (status) {
      case 'In Progress':
        return `${base} pd-priority__icon--high`;
      case 'Done':
        return `${base} pd-priority__icon--done`;
      default:
        return `${base} pd-priority__icon--std`;
    }
  }

  protected toggleRequirementsExpanded(): void {
    this.requirementsExpanded.update((v) => !v);
  }

  protected openDocumentUploadModal(): void {
    this.documentUploadModalOpen.set(true);
    this.documentUploadError.set(null);
    this.documentUploadSuccess.set(null);
  }

  protected closeDocumentUploadModal(): void {
    if (this.documentUploadState() === 'uploading') return;
    this.documentUploadModalOpen.set(false);
    this.selectedDocumentFiles.set([]);
    this.documentUploadError.set(null);
  }

  protected onDocumentFilesSelected(event: Event): void {
    const input = event.target as HTMLInputElement;
    const files = input.files ? Array.from(input.files) : [];
    this.selectedDocumentFiles.set(files);
    this.documentUploadError.set(null);
    this.documentUploadSuccess.set(null);
  }

  protected uploadSelectedDocuments(): void {
    const p = this.project();
    if (!p) return;

    const files = this.selectedDocumentFiles();
    if (files.length === 0) {
      this.documentUploadError.set('Select at least one file to upload.');
      return;
    }
    const unsupported = files.find((file) => !this.isAllowedDocumentFile(file.name));
    if (unsupported) {
      this.documentUploadError.set(
        `${unsupported.name} is not supported. Upload Markdown, text, Excel, Word, or common image files.`,
      );
      return;
    }

    this.uploadSub?.unsubscribe();
    this.documentUploadState.set('uploading');
    this.documentUploadError.set(null);
    this.documentUploadSuccess.set(null);
    this.uploadSub = this.projectApi.uploadProjectDocuments(p.id, files).subscribe({
      next: (rows) => {
        this.documentUploadSuccess.set(
          rows.length === 1 ? 'Document uploaded.' : `${rows.length} documents uploaded.`,
        );
        this.selectedDocumentFiles.set([]);
        this.documentUploadModalOpen.set(false);
        this.documentUploadState.set('idle');
        this.refreshProjectDocuments(p.id);
      },
      error: (err: unknown) => {
        this.documentUploadError.set(ProjectApiService.uploadDocumentsErrorMessage(err));
        this.documentUploadState.set('idle');
      },
    });
  }

  protected downloadProjectDocument(document: ProjectDocument): void {
    const p = this.project();
    if (!p) return;

    this.downloadSub?.unsubscribe();
    this.downloadingDocumentId.set(document.id);
    this.documentDownloadError.set(null);
    this.downloadSub = this.projectApi.downloadProjectDocument(p.id, document.id).subscribe({
      next: (blob) => {
        this.saveDownloadedDocument(blob, this.documentName(document));
        this.downloadingDocumentId.set(null);
      },
      error: (err: unknown) => {
        this.documentDownloadError.set(ProjectApiService.downloadDocumentErrorMessage(err));
        this.downloadingDocumentId.set(null);
      },
    });
  }

  protected openDocumentViewModal(projectDocument: ProjectDocument): void {
    const p = this.project();
    const previewKind = this.documentPreviewKind(projectDocument);
    if (!p || !previewKind) return;

    this.viewSub?.unsubscribe();
    this.revokeDocumentPreviewObjectUrl();
    this.documentPreviewModalOpen.set(true);
    this.documentPreview.set({
      document: projectDocument,
      kind: previewKind,
      status: 'loading',
      text: null,
      objectUrl: null,
      error: null,
    });

    const documentId = projectDocument.id;
    this.viewSub = this.projectApi.viewProjectDocument(p.id, documentId).subscribe({
      next: (blob) => {
        if (previewKind === 'image') {
          const objectUrl = URL.createObjectURL(blob);
          this.documentPreviewObjectUrl = objectUrl;
          this.documentPreview.set({
            document: projectDocument,
            kind: previewKind,
            status: 'ready',
            text: null,
            objectUrl,
            error: null,
          });
          return;
        }

        void blob
          .text()
          .then((text) => {
            const activePreview = this.documentPreview();
            if (!this.documentPreviewModalOpen() || activePreview?.document.id !== documentId) return;
            this.documentPreview.set({
              document: projectDocument,
              kind: previewKind,
              status: 'ready',
              text,
              objectUrl: null,
              error: null,
            });
          })
          .catch(() => {
            this.setDocumentPreviewError(projectDocument, previewKind, 'Could not read document preview.');
          });
      },
      error: (err: unknown) => {
        this.setDocumentPreviewError(
          projectDocument,
          previewKind,
          ProjectApiService.viewDocumentErrorMessage(err),
        );
      },
    });
  }

  protected closeDocumentViewModal(): void {
    this.viewSub?.unsubscribe();
    this.viewSub = null;
    this.documentPreviewModalOpen.set(false);
    this.documentPreview.set(null);
    this.revokeDocumentPreviewObjectUrl();
  }

  protected projectRequirementsText(requirements: string | null): string {
    const t = requirements?.trim();
    return t && t.length > 0 ? t : '';
  }

  protected hasExpandableProjectRequirements(requirements: string | null): boolean {
    return this.projectRequirementsText(requirements).length > 0;
  }

  protected tagline(requirements: string | null): string {
    const t = requirements?.trim();
    if (t && t.length > 0) {
      if (t.length <= 200) return t;
      return `${t.slice(0, 197)}…`;
    }
    return 'Add requirements when creating or editing the project.';
  }

  protected relativeCreatedLabel(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return '—';
    const diffSec = Math.round((d.getTime() - Date.now()) / 1000);
    const abs = Math.abs(diffSec);
    if (abs < 45) return 'Created just now';
    const divisions: { unit: Intl.RelativeTimeFormatUnit; secs: number }[] = [
      { unit: 'year', secs: 31536000 },
      { unit: 'month', secs: 2592000 },
      { unit: 'week', secs: 604800 },
      { unit: 'day', secs: 86400 },
      { unit: 'hour', secs: 3600 },
      { unit: 'minute', secs: 60 },
    ];
    for (const { unit, secs } of divisions) {
      if (abs >= secs) {
        const delta = Math.trunc(diffSec / secs);
        const rel = this.rtf.format(delta, unit);
        return `Created ${rel}`;
      }
    }
    return `Created ${d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })}`;
  }

  protected statusBadgeClass(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'pd-status pd-status--progress';
      case 'Done':
        return 'pd-status pd-status--done';
      default:
        return 'pd-status pd-status--pending';
    }
  }

  protected statusForDisplay(status: string): string {
    return status.toUpperCase();
  }

  protected documentName(document: ProjectDocument): string {
    const name = this.documentMetadataString(document, 'originalFilename');
    return name.length > 0 ? name : 'Untitled document';
  }

  protected documentType(document: ProjectDocument): string {
    const contentType = this.documentMetadataString(document, 'contentType').toLowerCase();
    const name = this.documentName(document);
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

  protected documentSize(document: ProjectDocument): string {
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

  protected documentAddedLabel(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return 'Unknown';
    return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
  }

  protected canViewProjectDocument(document: ProjectDocument): boolean {
    return this.documentPreviewKind(document) !== null;
  }

  protected documentViewLabel(document: ProjectDocument): string {
    if (this.canViewProjectDocument(document)) {
      return `View ${this.documentName(document)}`;
    }
    return `${this.documentName(document)} file not supported for viewing`;
  }

  protected documentViewTitle(document: ProjectDocument): string {
    return this.canViewProjectDocument(document) ? 'View document' : 'File not supported for viewing';
  }

  protected isViewingDocument(document: ProjectDocument): boolean {
    const preview = this.documentPreview();
    return preview?.document.id === document.id && preview.status === 'loading';
  }

  private isAllowedDocumentFile(name: string): boolean {
    const extension = name.split('.').pop()?.toLowerCase() ?? '';
    return [
      'md',
      'txt',
      'csv',
      'xls',
      'xlsx',
      'doc',
      'docx',
      'jpg',
      'jpeg',
      'png',
      'gif',
      'webp',
      'bmp',
      'svg',
    ].includes(extension);
  }

  private refreshProjectDocuments(projectId: string): void {
    this.documentsSub?.unsubscribe();
    this.documentsLoadError.set(null);
    this.documentsSub = this.projectApi.listProjectDocuments(projectId).subscribe({
      next: (documents) => {
        this.documents.set(documents);
        this.documentsLoadError.set(null);
      },
      error: (err: unknown) => {
        this.documents.set([]);
        this.documentsLoadError.set(ProjectApiService.listDocumentsErrorMessage(err));
      },
    });
  }

  private documentMetadataString(document: ProjectDocument, key: string): string {
    const value = document.metadata[key];
    return typeof value === 'string' ? value.trim() : '';
  }

  private documentPreviewKind(document: ProjectDocument): DocumentPreviewKind | null {
    const contentType = this.documentMetadataString(document, 'contentType').toLowerCase();
    const name = this.documentName(document);
    const extension = name.split('.').pop()?.trim().toLowerCase();

    if (
      contentType.startsWith('image/') ||
      ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(extension ?? '')
    ) {
      return 'image';
    }

    if (
      contentType.startsWith('text/') ||
      ['txt', 'md', 'csv'].includes(extension ?? '')
    ) {
      return 'text';
    }

    return null;
  }

  private setDocumentPreviewError(
    document: ProjectDocument,
    kind: DocumentPreviewKind,
    message: string,
  ): void {
    this.documentPreview.set({
      document,
      kind,
      status: 'error',
      text: null,
      objectUrl: null,
      error: message,
    });
  }

  private revokeDocumentPreviewObjectUrl(): void {
    if (!this.documentPreviewObjectUrl) return;
    URL.revokeObjectURL(this.documentPreviewObjectUrl);
    this.documentPreviewObjectUrl = null;
  }

  private saveDownloadedDocument(blob: Blob, fileName: string): void {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 0);
  }
}
