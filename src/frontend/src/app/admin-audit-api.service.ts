import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface AdminAuditLogEntry {
  id: string;
  createdAt: string;
  companyId: string | null;
  userId: string | null;
  systemAdminEmail: string | null;
  actionType: string;
  entityType: string;
  entityId: string | null;
  metadata: Record<string, unknown>;
  ipAddress: string | null;
  userAgent: string | null;
}

export interface AdminAuditLogsListResponse {
  items: AdminAuditLogEntry[];
  total: number;
  limit: number;
  offset: number;
}

@Injectable({
  providedIn: 'root',
})
export class AdminAuditApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.apiBaseUrl}/api/v1/admin/audit-logs`;

  list(options?: {
    companyId?: string;
    userId?: string;
    actionType?: string;
    from?: string;
    to?: string;
    limit?: number;
    offset?: number;
  }): Observable<AdminAuditLogsListResponse> {
    let params = new HttpParams();
    if (options?.companyId) {
      params = params.set('company_id', options.companyId);
    }
    if (options?.userId?.trim()) {
      params = params.set('user_id', options.userId.trim());
    }
    if (options?.actionType) {
      params = params.set('action_type', options.actionType);
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
    return this.http.get<AdminAuditLogsListResponse>(this.base, { params });
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body?.message) {
        return body.message;
      }
      if (err.status === 403) {
        return 'You do not have permission to view audit logs.';
      }
    }
    return 'Failed to load audit logs. Please try again.';
  }
}
