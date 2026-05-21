import { DatePipe, DecimalPipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { forkJoin } from 'rxjs';

import {
  AiUsageByProviderModelRow,
  AiUsageTotals,
  CompanyAiUsageByProjectRow,
  CompanyAiUsageByUserRow,
  CompanyAiUsageFailureRow,
} from '../ai-usage-types';
import { CompanyAiUsageApiService } from '../company-ai-usage-api.service';

type LoadState = 'loading' | 'ok' | 'error';
type ActiveSection = 'users' | 'projects' | 'providers' | 'failures';

@Component({
  selector: 'app-company-ai-usage',
  imports: [DatePipe, DecimalPipe, ReactiveFormsModule, RouterLink],
  templateUrl: './company-ai-usage.html',
  styleUrl: '../admin-ai-usage/admin-ai-usage.scss',
})
export class CompanyAiUsage implements OnInit {
  private readonly api = inject(CompanyAiUsageApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly activeSection = signal<ActiveSection>('users');

  protected readonly totals = signal<AiUsageTotals | null>(null);
  protected readonly byUser = signal<CompanyAiUsageByUserRow[]>([]);
  protected readonly byProject = signal<CompanyAiUsageByProjectRow[]>([]);
  protected readonly byProviderModel = signal<AiUsageByProviderModelRow[]>([]);
  protected readonly failures = signal<CompanyAiUsageFailureRow[]>([]);

  protected readonly rangeForm = new FormGroup({
    from: new FormControl(this.defaultFromLocal(), { nonNullable: true }),
    to: new FormControl(this.defaultToLocal(), { nonNullable: true }),
  });

  protected readonly failureFilters = new FormGroup({
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

  protected formatCost(value: number): string {
    return value.toFixed(4);
  }

  protected operationLabel(op: string): string {
    return op.replaceAll('_', ' ');
  }

  protected projectLabel(row: CompanyAiUsageByProjectRow): string {
    return row.projectName ?? 'Unattributed';
  }

  private load(): void {
    this.loadState.set('loading');
    this.loadError.set(null);

    const range = this.currentRange();

    forkJoin({
      totals: this.api.summary(range),
      users: this.api.byUser(range),
      projects: this.api.byProject(range),
      providers: this.api.byProviderModel(range),
      failures: this.api.failures({
        ...range,
        userId: this.failureFilters.controls.userId.value.trim() || undefined,
        provider: this.failureFilters.controls.provider.value.trim() || undefined,
      }),
    }).subscribe({
      next: (data) => {
        this.totals.set(data.totals);
        this.byUser.set(data.users);
        this.byProject.set(data.projects);
        this.byProviderModel.set(data.providers);
        this.failures.set(data.failures);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(CompanyAiUsageApiService.errorMessage(err));
        this.loadState.set('error');
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
    return new Date(local).toISOString();
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
