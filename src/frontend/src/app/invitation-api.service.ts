import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import type { CompanyRole, CurrentUser } from './auth-api.service';
import { environment } from '../environments/environment';

export type InvitationStatus = 'pending' | 'accepted' | 'expired' | 'cancelled';

export interface Invitation {
  id: string;
  companyId: string;
  projectId: string | null;
  invitedEmail: string;
  invitedRole: CompanyRole;
  invitedByUserId: string;
  status: InvitationStatus;
  expiresAt: string;
  acceptedAt: string | null;
  createdAt: string;
}

export interface CreateInvitationPayload {
  invitedEmail: string;
  invitedRole: CompanyRole;
  projectId?: string | null;
  message?: string | null;
}

export interface InvitationPreview {
  companyName: string;
  projectName: string | null;
  invitedRole: CompanyRole;
  invitedEmail: string;
  status: InvitationStatus;
  expiresAt: string;
  existingUserPresent: boolean;
}

export interface AcceptInvitationRegisterPayload {
  token: string;
  fullname: string;
  username: string;
  password: string;
  passwordConfirmation: string;
  timezone?: string | null;
}

@Injectable({
  providedIn: 'root',
})
export class InvitationApiService {
  private readonly http = inject(HttpClient);

  listInvitations(): Observable<Invitation[]> {
    const url = `${environment.apiBaseUrl}/api/v1/invitations`;
    return this.http.get<Invitation[]>(url);
  }

  createInvitation(payload: CreateInvitationPayload): Observable<Invitation> {
    const url = `${environment.apiBaseUrl}/api/v1/invitations`;
    return this.http.post<Invitation>(url, payload);
  }

  cancelInvitation(id: string): Observable<Invitation> {
    const url = `${environment.apiBaseUrl}/api/v1/invitations/${id}/cancel`;
    return this.http.post<Invitation>(url, {});
  }

  previewInvitation(token: string): Observable<InvitationPreview> {
    const url = `${environment.apiBaseUrl}/api/v1/invitations/accept/preview`;
    return this.http.get<InvitationPreview>(url, { params: { token } });
  }

  acceptInvitationRegister(payload: AcceptInvitationRegisterPayload): Observable<CurrentUser> {
    const url = `${environment.apiBaseUrl}/api/v1/invitations/accept/register`;
    return this.http.post<CurrentUser>(url, payload);
  }

  acceptInvitationConfirm(token: string): Observable<CurrentUser> {
    const url = `${environment.apiBaseUrl}/api/v1/invitations/accept/confirm`;
    return this.http.post<CurrentUser>(url, { token });
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string; invitationStatus?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Request failed (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}
