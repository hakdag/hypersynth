import { DatePipe, DecimalPipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { forkJoin } from 'rxjs';

import {
  AdminAiUsageApiService,
  AdminAiUsageByCompanyRow,
  AdminAiUsageByProviderModelRow,
  AdminAiUsageByUserRow,
  AdminAiUsageFailureRow,
  AdminAiUsageSort,
  AdminAiUsageTotals,
} from '../admin-ai-usage-api.service';

type LoadState = 'loading' | 'ok' | 'error';
type ActiveSection = 'companies' | 'users' | 'providers' | 'failures';

@Component({
  selector: 'app-admin-ai-usage',
  imports: [DatePipe, DecimalPipe, ReactiveFormsModule, RouterLink],
  templateUrl: './admin-ai-usage.html',
  styleUrl: './admin-ai-usage.scss',
})
export class AdminAiUsage implements OnInit {
  private readonly api = inject(AdminAiUsageApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly activeSection = signal<ActiveSection>('companies');
  protected readonly companySort = signal<AdminAiUsageSort>('tokens');

  protected readonly totals = signal<AdminAiUsageTotals | null>(null);
  protected readonly byCompany = signal<AdminAiUsageByCompanyRow[]>([]);
  protected readonly byUser = signal<AdminAiUsageByUserRow[]>([]);
  protected readonly byProviderModel = signal<AdminAiUsageByProviderModelRow[]>([]);
  protected readonly failures = signal<AdminAiUsageFailureRow[]>([]);

  protected readonly rangeForm = new FormGroup({
    from: new FormControl(this.defaultFromLocal(), { nonNullable: true }),
    to: new FormControl(this.defaultToLocal(), { nonNullable: true }),
  });

  protected readonly failureFilters = new FormGroup({
    companyId: new FormControl('', { nonNullable: true }),
    userId: new FormControl('', { nonNullable: true }),
    provider: new FormControl('', { nonNullable: true }),
  });

  ngOnInit(): void {
    this.load();
  }

  protected refresh(): void {
    this.load();
  }

  protected setSection(section: ActiveSection): void {
    this.activeSection.set(section);
  }

  protected setCompanySort(sort: AdminAiUsageSort): void {
    this.companySort.set(sort);
    this.loadByCompany();
  }

  protected companyLabel(row: AdminAiUsageByCompanyRow): string {
    return row.companyName ?? 'Personal (no company)';
  }

  protected formatCost(value: number): string {
    return value.toFixed(4);
  }

  protected operationLabel(op: string): string {
    return op.replaceAll('_', ' ');
  }

  private load(): void {
    this.loadState.set('loading');
    this.loadError.set(null);

    const range = this.currentRange();

    forkJoin({
      totals: this.api.summary(range),
      companies: this.api.byCompany({ ...range, sort: this.companySort() }),
      users: this.api.byUser(range),
      providers: this.api.byProviderModel(range),
      failures: this.api.failures({
        ...range,
        companyId: this.failureFilters.controls.companyId.value.trim() || undefined,
        userId: this.failureFilters.controls.userId.value.trim() || undefined,
        provider: this.failureFilters.controls.provider.value.trim() || undefined,
      }),
    }).subscribe({
      next: (data) => {
        this.totals.set(data.totals);
        this.byCompany.set(data.companies);
        this.byUser.set(data.users);
        this.byProviderModel.set(data.providers);
        this.failures.set(data.failures);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(AdminAiUsageApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }

  private loadByCompany(): void {
    const range = this.currentRange();
    this.api.byCompany({ ...range, sort: this.companySort() }).subscribe({
      next: (rows) => this.byCompany.set(rows),
      error: (err: unknown) => {
        this.loadError.set(AdminAiUsageApiService.errorMessage(err));
      },
    });
  }

  private currentRange(): { from: string; to: string } {
    return {
      from: this.localToIso(this.rangeForm.controls.from.value),
      to: this.localToIso(this.rangeForm.controls.to.value),
    };
  }

  private localToIso(local: string): string {
    const parsed = new Date(local);
    return parsed.toISOString();
  }

  private defaultFromLocal(): string {
    const d = new Date();
    d.setDate(d.getDate() - 30);
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
