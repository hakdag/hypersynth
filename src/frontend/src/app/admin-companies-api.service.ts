import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export type CompanyStatusValue = 'active' | 'disabled' | 'pending_verification';

export interface AdminCompanySummary {
  id: string;
  name: string;
  companyEmail: string;
  status: CompanyStatusValue;
  userCount: number;
  projectCount: number;
  documentCount: number;
  createdAt: string;
}

export interface AdminAiUsageSummary {
  totalRequests: number;
  totalTokens: number;
  estimatedCost: number;
}

export interface AdminCompanyDetail {
  id: string;
  name: string;
  companyEmail: string;
  country: string;
  timezone: string;
  legalName: string | null;
  website: string | null;
  industry: string | null;
  companySize: string | null;
  phone: string | null;
  billingEmail: string | null;
  address: string | null;
  taxVatNumber: string | null;
  status: CompanyStatusValue;
  createdAt: string;
  updatedAt: string;
  userCount: number;
  projectCount: number;
  documentCount: number;
  aiUsage: AdminAiUsageSummary | null;
}

@Injectable({
  providedIn: 'root',
})
export class AdminCompaniesApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.apiBaseUrl}/api/v1/admin/companies`;

  list(options?: { search?: string; limit?: number; offset?: number }): Observable<AdminCompanySummary[]> {
    let params = new HttpParams();
    if (options?.search?.trim()) {
      params = params.set('search', options.search.trim());
    }
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<AdminCompanySummary[]>(this.base, { params });
  }

  get(id: string): Observable<AdminCompanyDetail> {
    return this.http.get<AdminCompanyDetail>(`${this.base}/${id}`);
  }

  setStatus(id: string, status: 'active' | 'disabled'): Observable<AdminCompanyDetail> {
    return this.http.post<AdminCompanyDetail>(`${this.base}/${id}/status`, { status });
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
