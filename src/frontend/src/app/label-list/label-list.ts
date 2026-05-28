import { CommonModule } from '@angular/common';
import { Component, OnInit, computed, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { AuthService } from '../auth.service';
import { Label, LabelsApiService } from '../labels-api.service';

@Component({
  selector: 'app-label-list',
  imports: [CommonModule, FormsModule],
  templateUrl: './label-list.html',
  styleUrl: './label-list.scss',
})
export class LabelList implements OnInit {
  private readonly labelsApi = inject(LabelsApiService);
  private readonly auth = inject(AuthService);

  protected readonly labels = signal<Label[]>([]);
  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly actionError = signal<string | null>(null);
  protected readonly saving = signal(false);
  protected readonly deletingId = signal<string | null>(null);
  protected readonly editingId = signal<string | null>(null);
  protected readonly editName = signal('');
  protected readonly editColor = signal('#3B82F6');

  protected readonly createName = signal('');
  protected readonly createColor = signal('#3B82F6');
  protected readonly canManageLabels = this.auth.canManageLabels;
  protected readonly sortedLabels = computed(() =>
    [...this.labels()].sort((a, b) => a.name.localeCompare(b.name)),
  );

  ngOnInit(): void {
    this.refresh();
  }

  protected refresh(): void {
    this.loadState.set('loading');
    this.loadError.set(null);
    this.labelsApi.listLabels().subscribe({
      next: (rows) => {
        this.labels.set(rows);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(LabelsApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }

  protected create(): void {
    if (!this.canManageLabels()) {
      return;
    }
    const name = this.createName().trim();
    const color = this.normalizeColor(this.createColor());
    if (!name || !color) {
      this.actionError.set('Name and color are required.');
      return;
    }
    this.actionError.set(null);
    this.saving.set(true);
    this.labelsApi.createLabel({ name, color }).subscribe({
      next: (label) => {
        this.labels.update((rows) => [...rows, label]);
        this.createName.set('');
        this.createColor.set('#3B82F6');
        this.saving.set(false);
      },
      error: (err: unknown) => {
        this.actionError.set(LabelsApiService.errorMessage(err));
        this.saving.set(false);
      },
    });
  }

  protected startEdit(label: Label): void {
    if (!this.canManageLabels()) {
      return;
    }
    this.editingId.set(label.id);
    this.editName.set(label.name);
    this.editColor.set(label.color);
    this.actionError.set(null);
  }

  protected cancelEdit(): void {
    this.editingId.set(null);
    this.editName.set('');
    this.editColor.set('#3B82F6');
  }

  protected saveEdit(label: Label): void {
    if (!this.canManageLabels()) {
      return;
    }
    const name = this.editName().trim();
    const color = this.normalizeColor(this.editColor());
    if (!name || !color) {
      this.actionError.set('Name and color are required.');
      return;
    }

    this.actionError.set(null);
    this.saving.set(true);
    this.labelsApi.updateLabel(label.id, { name, color }).subscribe({
      next: (updated) => {
        this.labels.update((rows) => rows.map((row) => (row.id === updated.id ? updated : row)));
        this.cancelEdit();
        this.saving.set(false);
      },
      error: (err: unknown) => {
        this.actionError.set(LabelsApiService.errorMessage(err));
        this.saving.set(false);
      },
    });
  }

  protected delete(label: Label): void {
    if (!this.canManageLabels()) {
      return;
    }
    if (!window.confirm(`Delete label "${label.name}"?`)) {
      return;
    }
    this.actionError.set(null);
    this.deletingId.set(label.id);
    this.labelsApi.deleteLabel(label.id).subscribe({
      next: () => {
        this.labels.update((rows) => rows.filter((row) => row.id !== label.id));
        this.deletingId.set(null);
      },
      error: (err: unknown) => {
        this.actionError.set(LabelsApiService.errorMessage(err));
        this.deletingId.set(null);
      },
    });
  }

  protected textColor(hex: string): string {
    const value = this.normalizeColor(hex);
    if (!value) {
      return '#111827';
    }
    const r = Number.parseInt(value.slice(1, 3), 16);
    const g = Number.parseInt(value.slice(3, 5), 16);
    const b = Number.parseInt(value.slice(5, 7), 16);
    const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    return luma > 165 ? '#111827' : '#ffffff';
  }

  private normalizeColor(raw: string): string | null {
    const value = raw.trim().toUpperCase();
    if (/^#[0-9A-F]{6}$/.test(value)) {
      return value;
    }
    return null;
  }
}
