import { DatePipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';

import {
  AdminCompaniesApiService,
  AdminCompanyDetail as AdminCompanyDetailData,
  CompanyStatusValue,
} from '../admin-companies-api.service';

type LoadState = 'loading' | 'ok' | 'error';

@Component({
  selector: 'app-admin-company-detail',
  imports: [DatePipe, RouterLink],
  templateUrl: './admin-company-detail.html',
  styleUrl: './admin-company-detail.scss',
})
export class AdminCompanyDetail implements OnInit {
  private readonly api = inject(AdminCompaniesApiService);
  private readonly route = inject(ActivatedRoute);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly company = signal<AdminCompanyDetailData | null>(null);
  protected readonly actionError = signal<string | null>(null);
  protected readonly statusUpdating = signal(false);

  ngOnInit(): void {
    this.route.paramMap.subscribe((params) => {
      const id = params.get('companyId');
      if (!id) {
        this.loadError.set('Invalid company.');
        this.loadState.set('error');
        return;
      }
      this.load(id);
    });
  }

  protected statusLabel(status: CompanyStatusValue): string {
    switch (status) {
      case 'active':
        return 'Active';
      case 'disabled':
        return 'Disabled';
      case 'pending_verification':
        return 'Pending verification';
      default:
        return status;
    }
  }

  protected canActivate(): boolean {
    return this.company()?.status === 'disabled';
  }

  protected canDisable(): boolean {
    return this.company()?.status === 'active';
  }

  protected setStatus(next: 'active' | 'disabled'): void {
    const c = this.company();
    if (!c || this.statusUpdating()) {
      return;
    }
    const confirmed = window.confirm(
      next === 'disabled'
        ? `Disable "${c.name}"? All company users will lose access immediately.`
        : `Activate "${c.name}"? Company users will be able to sign in again.`,
    );
    if (!confirmed) {
      return;
    }

    this.actionError.set(null);
    this.statusUpdating.set(true);
    this.api.setStatus(c.id, next).subscribe({
      next: (updated) => {
        this.company.set(updated);
        this.statusUpdating.set(false);
      },
      error: (err: unknown) => {
        this.actionError.set(AdminCompaniesApiService.errorMessage(err));
        this.statusUpdating.set(false);
      },
    });
  }

  protected display(value: string | null | undefined): string {
    return value?.trim() ? value : '—';
  }

  private load(id: string): void {
    this.loadState.set('loading');
    this.loadError.set(null);
    this.api.get(id).subscribe({
      next: (detail) => {
        this.company.set(detail);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(AdminCompaniesApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }
}
