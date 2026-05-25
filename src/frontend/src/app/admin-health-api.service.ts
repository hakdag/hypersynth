import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export type HealthIndicatorStatus =
  | 'healthy'
  | 'degraded'
  | 'unavailable'
  | 'not_configured';

export interface HealthIndicator {
  status: HealthIndicatorStatus;
  summary: string;
  detail?: string | null;
}

export interface AdminSystemHealthResponse {
  application: HealthIndicator;
  database: HealthIndicator;
  backgroundJobs: HealthIndicator;
  aiProviderErrorRate: HealthIndicator;
  emailDelivery: HealthIndicator;
  storage: HealthIndicator;
}

@Injectable({
  providedIn: 'root',
})
export class AdminHealthApiService {
  private readonly http = inject(HttpClient);
  private readonly url = `${environment.apiBaseUrl}/api/v1/admin/health`;

  getHealth(): Observable<AdminSystemHealthResponse> {
    return this.http.get<AdminSystemHealthResponse>(this.url);
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body?.message) {
        return body.message;
      }
      return err.statusText || 'Request failed.';
    }
    return 'Could not reach the server.';
  }
}
