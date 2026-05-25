import { DatePipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { debounceTime, distinctUntilChanged } from 'rxjs';

import {
  AdminAuditApiService,
  AdminAuditLogEntry,
} from '../admin-audit-api.service';
import {
  buildAuditMetadataDisplay,
  hasExpandableMetadata,
  type AuditMetadataDisplay,
} from '../admin-audit-metadata';
import {
  AdminCompaniesApiService,
  AdminCompanySummary,
} from '../admin-companies-api.service';

type LoadState = 'loading' | 'ok' | 'error';

const ACTION_TYPE_OPTIONS: { value: string; label: string }[] = [
  { value: 'system_admin_login_success', label: 'System admin login success' },
  { value: 'system_admin_login_failure', label: 'System admin login failure' },
  {
    value: 'ai_enhance_project_requirements_requested',
    label: 'AI enhance project requirements',
  },
  {
    value: 'ai_enhance_feature_requirements_requested',
    label: 'AI enhance feature requirements',
  },
  { value: 'ai_generate_tasks_requested', label: 'AI generate tasks' },
  { value: 'companies_created', label: 'Company created' },
  { value: 'companies_updated', label: 'Company updated' },
  { value: 'users_created', label: 'User created' },
  { value: 'users_updated', label: 'User updated' },
  { value: 'projects_created', label: 'Project created' },
  { value: 'projects_updated', label: 'Project updated' },
  { value: 'projects_deleted', label: 'Project deleted' },
  { value: 'features_created', label: 'Feature created' },
  { value: 'features_updated', label: 'Feature updated' },
  { value: 'tasks_created', label: 'Task created' },
  { value: 'tasks_updated', label: 'Task updated' },
  { value: 'invitations_created', label: 'Invitation created' },
  { value: 'invitations_updated', label: 'Invitation updated' },
  { value: 'project_documents_created', label: 'Document uploaded' },
  { value: 'project_documents_deleted', label: 'Document deleted' },
  { value: 'project_ai_settings_updated', label: 'AI settings changed' },
];

@Component({
  selector: 'app-admin-audit-logs',
  imports: [DatePipe, ReactiveFormsModule, RouterLink],
  templateUrl: './admin-audit-logs.html',
  styleUrl: './admin-audit-logs.scss',
})
export class AdminAuditLogs implements OnInit {
  private readonly api = inject(AdminAuditApiService);
  private readonly companiesApi = inject(AdminCompaniesApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly items = signal<AdminAuditLogEntry[]>([]);
  protected readonly total = signal(0);
  protected readonly limit = signal(50);
  protected readonly offset = signal(0);
  protected readonly companies = signal<AdminCompanySummary[]>([]);
  protected readonly actionTypeOptions = ACTION_TYPE_OPTIONS;
  protected readonly expandedEntryId = signal<string | null>(null);

  protected readonly tableColumnCount = 8;

  protected readonly filterForm = new FormGroup({
    companyId: new FormControl('', { nonNullable: true }),
    userId: new FormControl('', { nonNullable: true }),
    actionType: new FormControl('', { nonNullable: true }),
    from: new FormControl(this.defaultFromLocal(), { nonNullable: true }),
    to: new FormControl(this.defaultToLocal(), { nonNullable: true }),
  });

  ngOnInit(): void {
    this.loadCompanies();
    this.load();

    this.filterForm.controls.companyId.valueChanges.subscribe(() => this.resetAndLoad());
    this.filterForm.controls.actionType.valueChanges.subscribe(() => this.resetAndLoad());
    this.filterForm.controls.from.valueChanges.subscribe(() => this.resetAndLoad());
    this.filterForm.controls.to.valueChanges.subscribe(() => this.resetAndLoad());
    this.filterForm.controls.userId.valueChanges
      .pipe(debounceTime(300), distinctUntilChanged())
      .subscribe(() => this.resetAndLoad());
  }

  protected refresh(): void {
    this.load();
  }

  protected previousPage(): void {
    const next = Math.max(0, this.offset() - this.limit());
    if (next === this.offset()) {
      return;
    }
    this.offset.set(next);
    this.load();
  }

  protected nextPage(): void {
    const next = this.offset() + this.limit();
    if (next >= this.total()) {
      return;
    }
    this.offset.set(next);
    this.load();
  }

  protected pageRangeLabel(): string {
    const count = this.items().length;
    if (count === 0) {
      return 'No records';
    }
    const start = this.offset() + 1;
    const end = this.offset() + count;
    return `Showing ${start}–${end} of ${this.total()}`;
  }

  protected canGoPrevious(): boolean {
    return this.offset() > 0;
  }

  protected canGoNext(): boolean {
    return this.offset() + this.limit() < this.total();
  }

  protected actionLabel(actionType: string): string {
    const match = ACTION_TYPE_OPTIONS.find((o) => o.value === actionType);
    if (match) {
      return match.label;
    }
    return actionType.replaceAll('_', ' ');
  }

  protected actorLabel(entry: AdminAuditLogEntry): string {
    if (entry.systemAdminEmail) {
      return entry.systemAdminEmail;
    }
    if (entry.userId) {
      return entry.userId;
    }
    return '—';
  }

  protected companyLabel(companyId: string): string {
    const match = this.companies().find((c) => c.id === companyId);
    return match?.name ?? companyId;
  }

  protected truncate(value: string | null, max = 32): string {
    if (!value) {
      return '—';
    }
    if (value.length <= max) {
      return value;
    }
    return `${value.slice(0, max)}…`;
  }

  protected isExpanded(entry: AdminAuditLogEntry): boolean {
    return this.expandedEntryId() === entry.id;
  }

  protected canExpand(entry: AdminAuditLogEntry): boolean {
    return hasExpandableMetadata(entry);
  }

  protected toggleExpand(entry: AdminAuditLogEntry): void {
    if (!this.canExpand(entry)) {
      return;
    }
    this.expandedEntryId.update((current) => (current === entry.id ? null : entry.id));
  }

  protected metadataDisplay(entry: AdminAuditLogEntry): AuditMetadataDisplay {
    return buildAuditMetadataDisplay(entry);
  }

  protected expandToggleLabel(entry: AdminAuditLogEntry): string {
    return this.isExpanded(entry) ? 'Hide' : 'View';
  }

  private resetAndLoad(): void {
    this.expandedEntryId.set(null);
    this.offset.set(0);
    this.load();
  }

  private loadCompanies(): void {
    this.companiesApi.list().subscribe({
      next: (rows) => this.companies.set(rows),
      error: () => this.companies.set([]),
    });
  }

  private load(): void {
    this.loadState.set('loading');
    this.loadError.set(null);

    const f = this.filterForm.controls;
    this.api
      .list({
        companyId: f.companyId.value || undefined,
        userId: f.userId.value || undefined,
        actionType: f.actionType.value || undefined,
        from: this.localToIso(f.from.value),
        to: this.localToIso(f.to.value),
        limit: this.limit(),
        offset: this.offset(),
      })
      .subscribe({
        next: (response) => {
          this.expandedEntryId.set(null);
          this.items.set(response.items);
          this.total.set(response.total);
          this.limit.set(response.limit);
          this.offset.set(response.offset);
          this.loadState.set('ok');
        },
        error: (err: unknown) => {
          this.loadError.set(AdminAuditApiService.errorMessage(err));
          this.loadState.set('error');
        },
      });
  }

  private localToIso(local: string): string {
    return new Date(local).toISOString();
  }

  private defaultFromLocal(): string {
    const d = new Date();
    d.setDate(d.getDate() - 7);
    return this.toLocalInputValue(d);
  }

  private defaultToLocal(): string {
    return this.toLocalInputValue(new Date());
  }

  private toLocalInputValue(date: Date): string {
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
  }
}
