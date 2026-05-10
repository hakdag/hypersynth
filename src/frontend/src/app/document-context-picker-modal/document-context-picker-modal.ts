import { CommonModule } from '@angular/common';
import { Component, inject, input, OnInit, output, signal } from '@angular/core';
import { finalize } from 'rxjs';

import {
  documentDisplayName,
  documentDisplaySize,
  documentDisplayType,
} from '../document-display.util';
import { ProjectApiService, type ProjectDocument } from '../project-api.service';

@Component({
  selector: 'app-document-context-picker-modal',
  imports: [CommonModule],
  templateUrl: './document-context-picker-modal.html',
  styleUrl: './document-context-picker-modal.scss',
})
export class DocumentContextPickerModal implements OnInit {
  private readonly projectApi = inject(ProjectApiService);

  readonly projectId = input.required<string>();
  readonly initialSelectedIds = input<string[]>([]);

  readonly confirmed = output<string[]>();
  readonly closed = output<void>();

  protected readonly documents = signal<ProjectDocument[]>([]);
  protected readonly loadError = signal<string | null>(null);
  protected readonly loading = signal(true);
  protected readonly selectedIds = signal<string[]>([]);

  protected readonly docName = documentDisplayName;
  protected readonly docType = documentDisplayType;
  protected readonly docSize = documentDisplaySize;

  ngOnInit(): void {
    const initial = this.initialSelectedIds();
    this.selectedIds.set([...initial]);
    this.projectApi
      .listProjectDocuments(this.projectId())
      .pipe(finalize(() => this.loading.set(false)))
      .subscribe({
        next: (rows) => {
          this.documents.set(rows);
          const valid = new Set(rows.map((d) => d.id));
          this.selectedIds.update((ids) => ids.filter((id) => valid.has(id)));
        },
        error: (err: unknown) => {
          this.loadError.set(ProjectApiService.loadDocumentsErrorMessage(err));
        },
      });
  }

  protected cancel(): void {
    this.closed.emit();
  }

  protected confirm(): void {
    this.confirmed.emit([...this.selectedIds()]);
  }

  protected isChecked(id: string): boolean {
    return this.selectedIds().includes(id);
  }

  protected toggle(id: string, checked: boolean): void {
    this.selectedIds.update((ids) => {
      const has = ids.includes(id);
      if (checked && !has) {
        return [...ids, id];
      }
      if (!checked && has) {
        return ids.filter((x) => x !== id);
      }
      return ids;
    });
  }
}
