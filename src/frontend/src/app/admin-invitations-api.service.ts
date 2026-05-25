import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import type { CompanyRole } from './auth-api.service';
import { environment } from '../environments/environment';

export type InvitationStatus = 'pending' | 'accepted' | 'expired' | 'cancelled';

export interface AdminInvitationSummary {
  id: string;
  companyId: string;
  companyName: string;
  invitedByUserId: string;
  inviterName: string;
  inviterEmail: string;
  invitedEmail: string;
  invitedRole: CompanyRole;
  status: InvitationStatus;
  expiresAt: string;
  createdAt: string;
}

export interface AdminInvitationsListResponse {
  items: AdminInvitationSummary[];
  total: number;
  limit: number;
  offset: number;
}

@Injectable({
  providedIn: 'root',
})
export class AdminInvitationsApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.apiBaseUrl}/api/v1/admin/invitations`;

  list(options?: {
    companyId?: string;
    status?: InvitationStatus;
    from?: string;
    to?: string;
    limit?: number;
    offset?: number;
  }): Observable<AdminInvitationsListResponse> {
    let params = new HttpParams();
    if (options?.companyId) {
      params = params.set('company_id', options.companyId);
    }
    if (options?.status) {
      params = params.set('status', options.status);
    }
    if (options?.from) {
      params = params.set('from', options.from);
    }
    if (options?.to) {
      params = params.set('to', options.to);
    }
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<AdminInvitationsListResponse>(this.base, { params });
  }

  cancelInvitation(id: string): Observable<AdminInvitationSummary> {
    return this.http.post<AdminInvitationSummary>(`${this.base}/${id}/cancel`, {});
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body?.message) {
        return body.message;
      }
      if (err.status === 403) {
        return 'You do not have permission to manage invitations.';
      }
      if (err.status === 404) {
        return 'Invitation not found or cannot be cancelled.';
      }
    }
    return 'Something went wrong. Please try again.';
  }
}
