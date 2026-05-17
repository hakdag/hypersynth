import { DatePipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { debounceTime, distinctUntilChanged } from 'rxjs';

import {
  AdminCompaniesApiService,
  AdminCompanySummary,
  CompanyStatusValue,
} from '../admin-companies-api.service';

type LoadState = 'loading' | 'ok' | 'error';

@Component({
  selector: 'app-admin-companies-list',
  imports: [DatePipe, ReactiveFormsModule, RouterLink],
  templateUrl: './admin-companies-list.html',
  styleUrl: './admin-companies-list.scss',
})
export class AdminCompaniesList implements OnInit {
  private readonly api = inject(AdminCompaniesApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly companies = signal<AdminCompanySummary[]>([]);

  protected readonly searchControl = new FormControl('', { nonNullable: true });

  ngOnInit(): void {
    this.load();
    this.searchControl.valueChanges
      .pipe(debounceTime(300), distinctUntilChanged())
      .subscribe(() => this.load());
  }

  protected refresh(): void {
    this.load();
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

  protected statusClass(status: CompanyStatusValue): string {
    if (status === 'active') {
      return 'ac-badge ac-badge-active';
    }
    if (status === 'disabled') {
      return 'ac-badge ac-badge-disabled';
    }
    return 'ac-badge';
  }

  private load(): void {
    this.loadState.set('loading');
    this.loadError.set(null);
    const search = this.searchControl.value.trim();
    this.api.list({ search: search || undefined }).subscribe({
      next: (rows) => {
        this.companies.set(rows);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(AdminCompaniesApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }
}
