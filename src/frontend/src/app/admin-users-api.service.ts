import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export type UserStatusValue = 'active' | 'disabled' | 'pending_invitation';
export type AccountTypeValue = 'personal' | 'company';
export type CompanyRoleValue =
  | 'company_admin'
  | 'project_manager'
  | 'contributor'
  | 'viewer';

export interface AdminUserSummary {
  id: string;
  fullName: string;
  email: string;
  username: string | null;
  accountType: AccountTypeValue;
  role: CompanyRoleValue | null;
  status: UserStatusValue;
  companyId: string | null;
  companyName: string | null;
  createdAt: string;
}

export interface AdminUserDetail {
  id: string;
  fullName: string;
  displayName: string | null;
  email: string;
  username: string | null;
  accountType: AccountTypeValue;
  role: CompanyRoleValue | null;
  status: UserStatusValue;
  timezone: string | null;
  companyId: string | null;
  companyName: string | null;
  createdAt: string;
  updatedAt: string;
  activeSessionCount: number;
}

@Injectable({
  providedIn: 'root',
})
export class AdminUsersApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.apiBaseUrl}/api/v1/admin/users`;

  list(options?: {
    search?: string;
    accountType?: AccountTypeValue;
    status?: UserStatusValue;
    companyId?: string;
    limit?: number;
    offset?: number;
  }): Observable<AdminUserSummary[]> {
    let params = new HttpParams();
    if (options?.search?.trim()) {
      params = params.set('search', options.search.trim());
    }
    if (options?.accountType) {
      params = params.set('account_type', options.accountType);
    }
    if (options?.status) {
      params = params.set('status', options.status);
    }
    if (options?.companyId) {
      params = params.set('company_id', options.companyId);
    }
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<AdminUserSummary[]>(this.base, { params });
  }

  get(id: string): Observable<AdminUserDetail> {
    return this.http.get<AdminUserDetail>(`${this.base}/${id}`);
  }

  setStatus(id: string, status: 'active' | 'disabled'): Observable<AdminUserDetail> {
    return this.http.post<AdminUserDetail>(`${this.base}/${id}/status`, { status });
  }

  resetAccess(id: string): Observable<AdminUserDetail> {
    return this.http.post<AdminUserDetail>(`${this.base}/${id}/reset-access`, {});
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Request failed (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}
