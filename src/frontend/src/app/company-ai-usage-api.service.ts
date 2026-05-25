import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';
import {
  AiUsageByProviderModelRow,
  AiUsageDateRange,
  AiUsageListOptions,
  AiUsageTotals,
  CompanyAiUsageByProjectRow,
  CompanyAiUsageByUserRow,
  CompanyAiUsageFailureRow,
} from './ai-usage-types';

@Injectable({
  providedIn: 'root',
})
export class CompanyAiUsageApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.apiBaseUrl}/api/v1/company/ai-usage`;

  summary(range?: AiUsageDateRange): Observable<AiUsageTotals> {
    return this.http.get<AiUsageTotals>(`${this.base}/summary`, {
      params: this.dateParams(range),
    });
  }

  byUser(options?: AiUsageListOptions): Observable<CompanyAiUsageByUserRow[]> {
    let params = this.dateParams(options);
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<CompanyAiUsageByUserRow[]>(`${this.base}/by-user`, { params });
  }

  byProject(options?: AiUsageListOptions): Observable<CompanyAiUsageByProjectRow[]> {
    let params = this.dateParams(options);
    if (options?.limit != null) {
      params = params.set('limit', String(options.limit));
    }
    if (options?.offset != null) {
      params = params.set('offset', String(options.offset));
    }
    return this.http.get<CompanyAiUsageByProjectRow[]>(`${this.base}/by-project`, { params });
  }

  byProviderModel(options?: AiUsageDateRange): Observable<AiUsageByProviderModelRow[]> {
    return this.http.get<AiUsageByProviderModelRow[]>(`${this.base}/by-provider-model`, {
      params: this.dateParams(options),
    });
  }

  failures(
    options?: AiUsageListOptions & { userId?: string; provider?: string },
  ): Observable<CompanyAiUsageFailureRow[]> {
    let params = this.dateParams(options);
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
    return this.http.get<CompanyAiUsageFailureRow[]>(`${this.base}/failures`, { params });
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

  private dateParams(range?: AiUsageDateRange): HttpParams {
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
