import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface PlatformConfig {
  allowedAiProviders: string[];
  defaultMonthlyTokenLimit: number | null;
  platformAnnouncement: string | null;
  featureFlags: Record<string, boolean>;
  updatedAt: string;
}

export interface UpdatePlatformConfigRequest {
  allowedAiProviders?: string[];
  defaultMonthlyTokenLimit?: number | null;
  platformAnnouncement?: string | null;
  featureFlags?: Record<string, boolean>;
}

@Injectable({
  providedIn: 'root',
})
export class AdminConfigApiService {
  private readonly http = inject(HttpClient);
  private readonly url = `${environment.apiBaseUrl}/api/v1/admin/platform-config`;

  get(): Observable<PlatformConfig> {
    return this.http.get<PlatformConfig>(this.url);
  }

  patch(body: UpdatePlatformConfigRequest): Observable<PlatformConfig> {
    return this.http.patch<PlatformConfig>(this.url, body);
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
