import { DatePipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';

import {
  AdminUserDetail as AdminUserDetailData,
  AdminUsersApiService,
  UserStatusValue,
} from '../admin-users-api.service';

type LoadState = 'loading' | 'ok' | 'error';

@Component({
  selector: 'app-admin-user-detail',
  imports: [DatePipe, RouterLink],
  templateUrl: './admin-user-detail.html',
  styleUrl: './admin-user-detail.scss',
})
export class AdminUserDetail implements OnInit {
  private readonly api = inject(AdminUsersApiService);
  private readonly route = inject(ActivatedRoute);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly user = signal<AdminUserDetailData | null>(null);
  protected readonly actionError = signal<string | null>(null);
  protected readonly statusUpdating = signal(false);
  protected readonly resetUpdating = signal(false);

  ngOnInit(): void {
    this.route.paramMap.subscribe((params) => {
      const id = params.get('userId');
      if (!id) {
        this.loadError.set('Invalid user.');
        this.loadState.set('error');
        return;
      }
      this.load(id);
    });
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

  protected accountTypeLabel(type: AdminUserDetailData['accountType']): string {
    return type === 'company' ? 'Company' : 'Personal';
  }

  protected roleLabel(role: AdminUserDetailData['role']): string {
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

  protected canActivate(): boolean {
    return this.user()?.status === 'disabled';
  }

  protected canDisable(): boolean {
    return this.user()?.status === 'active';
  }

  protected setStatus(next: 'active' | 'disabled'): void {
    const u = this.user();
    if (!u || this.statusUpdating()) {
      return;
    }
    const confirmed = window.confirm(
      next === 'disabled'
        ? `Disable "${u.fullName}"? They will be signed out and cannot sign in until re-enabled.`
        : `Activate "${u.fullName}"? They will be able to sign in again.`,
    );
    if (!confirmed) {
      return;
    }

    this.actionError.set(null);
    this.statusUpdating.set(true);
    this.api.setStatus(u.id, next).subscribe({
      next: (updated) => {
        this.user.set(updated);
        this.statusUpdating.set(false);
      },
      error: (err: unknown) => {
        this.actionError.set(AdminUsersApiService.errorMessage(err));
        this.statusUpdating.set(false);
      },
    });
  }

  protected resetAccess(): void {
    const u = this.user();
    if (!u || this.resetUpdating()) {
      return;
    }
    const confirmed = window.confirm(
      'This will sign the user out of all devices. They must sign in again to continue. Continue?',
    );
    if (!confirmed) {
      return;
    }

    this.actionError.set(null);
    this.resetUpdating.set(true);
    this.api.resetAccess(u.id).subscribe({
      next: (updated) => {
        this.user.set(updated);
        this.resetUpdating.set(false);
      },
      error: (err: unknown) => {
        this.actionError.set(AdminUsersApiService.errorMessage(err));
        this.resetUpdating.set(false);
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
        this.user.set(detail);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(AdminUsersApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }
}
