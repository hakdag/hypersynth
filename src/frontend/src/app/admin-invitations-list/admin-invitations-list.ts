import { DatePipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { finalize } from 'rxjs';

import {
  AdminCompaniesApiService,
  AdminCompanySummary,
} from '../admin-companies-api.service';
import {
  AdminInvitationSummary,
  AdminInvitationsApiService,
  InvitationStatus,
} from '../admin-invitations-api.service';

type LoadState = 'loading' | 'ok' | 'error';

const STATUS_OPTIONS: { value: '' | InvitationStatus; label: string }[] = [
  { value: '', label: 'Pending & expired (default)' },
  { value: 'pending', label: 'Pending' },
  { value: 'expired', label: 'Expired' },
  { value: 'accepted', label: 'Accepted' },
  { value: 'cancelled', label: 'Cancelled' },
];

@Component({
  selector: 'app-admin-invitations-list',
  imports: [DatePipe, ReactiveFormsModule, RouterLink],
  templateUrl: './admin-invitations-list.html',
  styleUrl: './admin-invitations-list.scss',
})
export class AdminInvitationsList implements OnInit {
  private readonly api = inject(AdminInvitationsApiService);
  private readonly companiesApi = inject(AdminCompaniesApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly items = signal<AdminInvitationSummary[]>([]);
  protected readonly total = signal(0);
  protected readonly limit = signal(50);
  protected readonly offset = signal(0);
  protected readonly companies = signal<AdminCompanySummary[]>([]);
  protected readonly statusOptions = STATUS_OPTIONS;
  protected readonly actionError = signal<string | null>(null);
  protected readonly cancellingId = signal<string | null>(null);

  protected readonly filterForm = new FormGroup({
    companyId: new FormControl('', { nonNullable: true }),
    status: new FormControl<'' | InvitationStatus>('', { nonNullable: true }),
    from: new FormControl(this.defaultFromLocal(), { nonNullable: true }),
    to: new FormControl(this.defaultToLocal(), { nonNullable: true }),
  });

  ngOnInit(): void {
    this.loadCompanies();
    this.load();

    this.filterForm.controls.companyId.valueChanges.subscribe(() => this.resetAndLoad());
    this.filterForm.controls.status.valueChanges.subscribe(() => this.resetAndLoad());
    this.filterForm.controls.from.valueChanges.subscribe(() => this.resetAndLoad());
    this.filterForm.controls.to.valueChanges.subscribe(() => this.resetAndLoad());
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

  protected inviterLabel(inv: AdminInvitationSummary): string {
    const name = inv.inviterName?.trim();
    if (name) {
      return name;
    }
    return inv.inviterEmail;
  }

  protected statusLabel(status: InvitationStatus): string {
    switch (status) {
      case 'pending':
        return 'Pending';
      case 'accepted':
        return 'Accepted';
      case 'expired':
        return 'Expired';
      case 'cancelled':
        return 'Cancelled';
      default:
        return status;
    }
  }

  protected statusClass(status: InvitationStatus): string {
    if (status === 'pending') {
      return 'au-badge au-badge-active';
    }
    if (status === 'expired') {
      return 'au-badge';
    }
    if (status === 'cancelled') {
      return 'au-badge au-badge-disabled';
    }
    return 'au-badge';
  }

  protected dayLabel(dateOnly: string): string {
    const date = this.parseLocalDate(dateOnly);
    return date.toLocaleDateString(undefined, {
      weekday: 'long',
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }

  protected roleLabel(role: AdminInvitationSummary['invitedRole']): string {
    switch (role) {
      case 'company_admin':
        return 'Company Admin';
      case 'project_manager':
        return 'Project Manager';
      case 'contributor':
        return 'Contributor';
      case 'viewer':
        return 'Viewer';
      default:
        return role;
    }
  }

  protected cancel(inv: AdminInvitationSummary): void {
    if (inv.status !== 'pending') {
      return;
    }
    if (!window.confirm(`Cancel invitation to ${inv.invitedEmail}?`)) {
      return;
    }
    this.actionError.set(null);
    this.cancellingId.set(inv.id);
    this.api
      .cancelInvitation(inv.id)
      .pipe(finalize(() => this.cancellingId.set(null)))
      .subscribe({
        next: (updated) => {
          this.items.update((list) =>
            list.map((row) => (row.id === updated.id ? updated : row)),
          );
        },
        error: (err: unknown) => {
          this.actionError.set(AdminInvitationsApiService.errorMessage(err));
        },
      });
  }

  private resetAndLoad(): void {
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
        status: f.status.value || undefined,
        from: this.dateToRangeStart(f.from.value),
        to: this.dateToRangeEnd(f.to.value),
        limit: this.limit(),
        offset: this.offset(),
      })
      .subscribe({
        next: (response) => {
          this.items.set(response.items);
          this.total.set(response.total);
          this.limit.set(response.limit);
          this.offset.set(response.offset);
          this.loadState.set('ok');
        },
        error: (err: unknown) => {
          this.loadError.set(AdminInvitationsApiService.errorMessage(err));
          this.loadState.set('error');
        },
      });
  }

  private dateToRangeStart(dateOnly: string): string {
    const date = this.parseLocalDate(dateOnly);
    date.setHours(0, 0, 0, 0);
    return date.toISOString();
  }

  private dateToRangeEnd(dateOnly: string): string {
    const date = this.parseLocalDate(dateOnly);
    date.setHours(23, 59, 59, 999);
    return date.toISOString();
  }

  private parseLocalDate(dateOnly: string): Date {
    const [year, month, day] = dateOnly.split('-').map((part) => Number(part));
    return new Date(year, month - 1, day);
  }

  private defaultFromLocal(): string {
    const d = new Date();
    d.setDate(d.getDate() - 7);
    return this.toDateInputValue(d);
  }

  private defaultToLocal(): string {
    return this.toDateInputValue(new Date());
  }

  private toDateInputValue(date: Date): string {
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  }
}
