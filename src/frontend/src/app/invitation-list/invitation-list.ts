import { CommonModule } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { finalize } from 'rxjs';

import { InvitationApiService, type Invitation } from '../invitation-api.service';

@Component({
  selector: 'app-invitation-list',
  imports: [CommonModule, RouterLink],
  templateUrl: './invitation-list.html',
  styleUrl: './invitation-list.scss',
})
export class InvitationList implements OnInit {
  private readonly invitationApi = inject(InvitationApiService);

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly invitations = signal<Invitation[]>([]);
  protected readonly actionError = signal<string | null>(null);
  protected readonly cancellingId = signal<string | null>(null);

  ngOnInit(): void {
    this.refresh();
  }

  protected refresh(): void {
    this.loadState.set('loading');
    this.loadError.set(null);
    this.invitationApi.listInvitations().subscribe({
      next: (rows) => {
        this.invitations.set(rows);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(InvitationApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }

  protected cancel(inv: Invitation): void {
    if (inv.status !== 'pending') {
      return;
    }
    if (!window.confirm(`Cancel invitation to ${inv.invitedEmail}?`)) {
      return;
    }
    this.actionError.set(null);
    this.cancellingId.set(inv.id);
    this.invitationApi
      .cancelInvitation(inv.id)
      .pipe(finalize(() => this.cancellingId.set(null)))
      .subscribe({
        next: (updated) => {
          this.invitations.update((list) =>
            list.map((row) => (row.id === updated.id ? updated : row)),
          );
        },
        error: (err: unknown) => {
          this.actionError.set(InvitationApiService.errorMessage(err));
        },
      });
  }

  protected statusLabel(status: Invitation['status']): string {
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

  protected roleLabel(role: Invitation['invitedRole']): string {
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
}
