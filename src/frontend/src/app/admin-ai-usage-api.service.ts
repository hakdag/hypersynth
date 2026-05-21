import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export type AdminAiUsageSort = 'tokens' | 'cost';

export interface AdminAiUsageTotals {
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface AdminAiUsageByCompanyRow {
  companyId: string | null;
  companyName: string | null;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface AdminAiUsageByUserRow {
  userId: string;
  userEmail: string;
  userFullName: string;
  companyId: string | null;
  companyName: string | null;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface AdminAiUsageByProviderModelRow {
  provider: string;
  model: string;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface AdminAiUsageFailureRow {
  id: string;
  companyId: string | null;
  companyName: string | null;
  userId: string;
  userEmail: string;
  provider: string;
  model: string;
  operationType: string;
  errorCode: string | null;
  createdAt: string;
}

export interface AdminAiUsageDateRange {
  from?: string;
  to?: string;
}

export interface AdminAiUsageListOptions extends AdminAiUsageDateRange {
  limit?: number;
  offset?: number;
}

@Injectable({
  providedIn: 'root',
})
export class AdminAiUsageApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.apiBaseUrl}/api/v1/admin/ai-usage`;

  summary(range?: AdminAiUsageDateRange): Observable<AdminAiUsageTotals> {
    return this.http.get<AdminAiUsageTotals>(`${this.base}/summary`, {
      params: this.dateParams(range),
    });
  }

  byCompany(
    options?: AdminAiUsageListOptions & { sort?: AdminAiUsageSort },
  ): Observable<AdminAiUsageByCompanyRow[]> {
    let params = this.dateParams(options);
    if (options?.sort) {
      params = params.set('sort', options.sort);
    }
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<AdminAiUsageByCompanyRow[]>(`${this.base}/by-company`, { params });
  }

  byUser(
    options?: AdminAiUsageListOptions & { companyId?: string },
  ): Observable<AdminAiUsageByUserRow[]> {
    let params = this.dateParams(options);
    if (options?.companyId) {
      params = params.set('companyId', options.companyId);
    }
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<AdminAiUsageByUserRow[]>(`${this.base}/by-user`, { params });
  }

  byProviderModel(
    options?: AdminAiUsageDateRange & { companyId?: string },
  ): Observable<AdminAiUsageByProviderModelRow[]> {
    let params = this.dateParams(options);
    if (options?.companyId) {
      params = params.set('companyId', options.companyId);
    }
    return this.http.get<AdminAiUsageByProviderModelRow[]>(`${this.base}/by-provider-model`, {
      params,
    });
  }

  failures(
    options?: AdminAiUsageListOptions & {
      companyId?: string;
      userId?: string;
      provider?: string;
    },
  ): Observable<AdminAiUsageFailureRow[]> {
    let params = this.dateParams(options);
    if (options?.companyId) {
      params = params.set('companyId', options.companyId);
    }
    if (options?.userId) {
      params = params.set('userId', options.userId);
    }
    if (options?.provider?.trim()) {
      params = params.set('provider', options.provider.trim());
    }
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<AdminAiUsageFailureRow[]>(`${this.base}/failures`, { params });
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

  private dateParams(range?: AdminAiUsageDateRange): HttpParams {
    let params = new HttpParams();
    if (range?.from) {
      params = params.set('from', range.from);
    }
    if (range?.to) {
      params = params.set('to', range.to);
    }
    return params;
  }
}
