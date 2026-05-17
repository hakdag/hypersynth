import { DatePipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { debounceTime, distinctUntilChanged } from 'rxjs';

import {
  AccountTypeValue,
  AdminUsersApiService,
  AdminUserSummary,
  UserStatusValue,
} from '../admin-users-api.service';

type LoadState = 'loading' | 'ok' | 'error';

@Component({
  selector: 'app-admin-users-list',
  imports: [DatePipe, ReactiveFormsModule, RouterLink],
  templateUrl: './admin-users-list.html',
  styleUrl: './admin-users-list.scss',
})
export class AdminUsersList implements OnInit {
  private readonly api = inject(AdminUsersApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly users = signal<AdminUserSummary[]>([]);

  protected readonly searchControl = new FormControl('', { nonNullable: true });
  protected readonly accountTypeControl = new FormControl<'' | AccountTypeValue>('', {
    nonNullable: true,
  });
  protected readonly statusControl = new FormControl<'' | UserStatusValue>('', {
    nonNullable: true,
  });

  ngOnInit(): void {
    this.load();
    this.searchControl.valueChanges
      .pipe(debounceTime(300), distinctUntilChanged())
      .subscribe(() => this.load());
    this.accountTypeControl.valueChanges.subscribe(() => this.load());
    this.statusControl.valueChanges.subscribe(() => this.load());
  }

  protected refresh(): void {
    this.load();
  }

  protected accountTypeLabel(type: AccountTypeValue): string {
    return type === 'company' ? 'Company' : 'Personal';
  }

  protected roleLabel(role: AdminUserSummary['role']): string {
    if (!role) {
      return '—';
    }
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

  protected statusLabel(status: UserStatusValue): string {
    switch (status) {
      case 'active':
        return 'Active';
      case 'disabled':
        return 'Disabled';
      case 'pending_invitation':
        return 'Pending invitation';
      default:
        return status;
    }
  }

  protected statusClass(status: UserStatusValue): string {
    if (status === 'active') {
      return 'au-badge au-badge-active';
    }
    if (status === 'disabled') {
      return 'au-badge au-badge-disabled';
    }
    return 'au-badge';
  }

  private load(): void {
    this.loadState.set('loading');
    this.loadError.set(null);
    const search = this.searchControl.value.trim();
    const accountType = this.accountTypeControl.value;
    const status = this.statusControl.value;
    this.api
      .list({
        search: search || undefined,
        accountType: accountType || undefined,
        status: status || undefined,
      })
      .subscribe({
        next: (rows) => {
          this.users.set(rows);
          this.loadState.set('ok');
        },
        error: (err: unknown) => {
          this.loadError.set(AdminUsersApiService.errorMessage(err));
          this.loadState.set('error');
        },
      });
  }
}
